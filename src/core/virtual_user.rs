use std::sync::Arc;
use std::time::{Duration, Instant};

use once_cell::sync::Lazy;
use reqwest;
use tokio::sync::{watch, Mutex};
use tokio::task::JoinHandle;

use super::metrics::Metrics;

static GLOBAL_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .build()
        .expect("failed to build client")
});

pub struct VirtualUser {
    url: String,
    metrics: Arc<Mutex<Metrics>>,
    client: reqwest::Client,
    graceful_shutdown: Duration,
    shutdown_tx: Option<watch::Sender<bool>>,
    join_handle: Option<JoinHandle<()>>,
}

impl VirtualUser {
    pub fn new(url: &str, rps_window_size: Duration) -> Self {
        if rps_window_size.as_secs() == 0 {
            panic!("rps_window_size must be greater than 0");
        }

        Self {
            url: url.to_string(),
            metrics: Arc::new(Metrics::new(rps_window_size).into()),
            client: GLOBAL_CLIENT.clone(),
            graceful_shutdown: Duration::from_secs(0),
            shutdown_tx: None,
            join_handle: None,
        }
    }

    pub fn set_graceful_shutdown(self, graceful_shutdown: Duration) -> Self {
        Self {
            graceful_shutdown,
            ..self
        }
    }

    pub fn start(&mut self) {
        let (tx, rx) = watch::channel(false);
        self.shutdown_tx = Some(tx);

        let url = self.url.clone();
        let client = self.client.clone();
        let metrics = self.metrics.clone();

        let handle = tokio::spawn(async move {
            let _ = client.get(&url).send().await;

            {
                let mut m = metrics.lock().await;
                m.rps_summary.start();
            }
            loop {
                if *rx.borrow() {
                    break;
                }

                let req_start = Instant::now();
                let response_result = client.get(&url).send().await;
                let latency = req_start.elapsed().as_secs_f64();

                {
                    let mut m = metrics.lock().await;
                    m.total_latency.update(latency);
                    m.http_request_time.update(latency);
                    let _ = m.rps_summary.increment_request_count();
                }

                match response_result {
                    Ok(resp) => {
                        let status = resp.status().as_u16();
                        let mut m = metrics.lock().await;
                        *m.status_code_counts.entry(status).or_insert(0) += 1;
                    }
                    Err(e) => {
                        let mut m = metrics.lock().await;
                        m.total_errors += 1;
                        m.other_errors.push(e.to_string());
                    }
                }
            }
        });

        self.join_handle = Some(handle);
    }

    pub async fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(true);
        }

        if let Some(mut handle) = self.join_handle.take() {
            if self.graceful_shutdown > Duration::from_secs(0) {
                tokio::select! {
                    _ = &mut handle => {},
                    _ = tokio::time::sleep(self.graceful_shutdown) => {
                        handle.abort();
                    },
                }
            } else {
                handle.abort();
            }
        }
    }

    pub fn metrics(&self) -> Arc<Mutex<Metrics>> {
        self.metrics.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    #[should_panic]
    fn test_create_new_virtual_user_with_zero_second() {
        let url = "http://test.com";
        let rps_window_size = Duration::from_secs(0);
        VirtualUser::new(url, rps_window_size);
    }

    #[test]
    fn test_create_new_virtual_user() {
        let url = "http://test.com";
        let rps_window_size = Duration::from_secs(1);
        let vu = VirtualUser::new(url, rps_window_size);
        assert_eq!(vu.url, url);
        assert_eq!(vu.graceful_shutdown, Duration::from_secs(0));
    }

    #[tokio::test]
    async fn test_virtual_user_success() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let url = mock_server.uri();
        let mut vu = VirtualUser::new(&url, Duration::from_secs(1))
            .set_graceful_shutdown(Duration::from_millis(50));
        vu.start();

        sleep(Duration::from_millis(200)).await;
        vu.stop().await;

        let metrics = vu.metrics();
        let m = metrics.lock().await;
        assert!(m.http_request_time.count() > 0);
        assert!(m.status_code_counts.contains_key(&200));
        assert_eq!(m.total_errors, 0);
    }

    #[tokio::test]
    async fn test_virtual_user_failure() {
        let invalid_url = "http://127.0.0.1:12345";
        let mut vu = VirtualUser::new(invalid_url, Duration::from_secs(1))
            .set_graceful_shutdown(Duration::from_millis(50));
        vu.start();

        sleep(Duration::from_millis(200)).await;
        vu.stop().await;

        let metrics = vu.metrics();
        let m = metrics.lock().await;
        assert!(m.total_errors > 0);
        assert!(m.status_code_counts.is_empty());
    }
}
