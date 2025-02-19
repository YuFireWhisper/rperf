use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest;
use tokio::sync::{watch, Mutex};
use tokio::task::JoinHandle;

use super::metrics::Metrics;

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
        Self {
            url: url.to_string(),
            metrics: Arc::new(Mutex::new(Metrics::new(rps_window_size))),
            client: reqwest::Client::new(),
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
                    _ = &mut handle => {
                    },
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
