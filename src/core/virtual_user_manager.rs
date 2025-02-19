use std::time::{Duration, Instant};
use tokio::time::sleep;

use crate::core::metrics::Metrics;
use crate::core::summary::Summary;
use crate::core::virtual_user::VirtualUser;

#[derive(Debug, Clone)]
pub struct VirtualUserConfig {
    pub url: String,
    pub rps_window_size: Duration,
    pub graceful_shutdown: Duration,
}

impl VirtualUserConfig {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            rps_window_size: Duration::from_secs(1),
            graceful_shutdown: Duration::from_secs(0),
        }
    }

    pub fn rps_window_size(mut self, window_size: Duration) -> Self {
        self.rps_window_size = window_size;
        self
    }

    pub fn graceful_shutdown(mut self, shutdown: Duration) -> Self {
        self.graceful_shutdown = shutdown;
        self
    }
}

#[derive(Debug, Clone)]
pub struct PlanSegment {
    pub duration: Duration,
    pub target: usize,
}

impl PlanSegment {
    pub fn new(duration: Duration, target: usize) -> Self {
        Self { duration, target }
    }
}

pub struct VirtualUserManager {
    config: VirtualUserConfig,
    plans: Vec<PlanSegment>,
    running_vus: Vec<VirtualUser>,
    overall_metrics: Metrics,
}

impl VirtualUserManager {
    pub fn new(config: VirtualUserConfig) -> Self {
        let overall_metrics = Metrics::new(config.rps_window_size);
        Self {
            config,
            plans: Vec::new(),
            running_vus: Vec::new(),
            overall_metrics,
        }
    }

    pub fn add_plan(&mut self, duration: Duration, target: usize) {
        self.plans.push(PlanSegment::new(duration, target));
    }

    pub async fn run(&mut self) {
        let tick_interval = Duration::from_millis(100);
        let mut current_count = self.running_vus.len();

        for plan in &self.plans {
            let segment_start_count = current_count;
            let target_count = plan.target;
            let change = target_count as isize - segment_start_count as isize;
            let segment_duration = plan.duration;
            let start_time = Instant::now();

            while start_time.elapsed() < segment_duration {
                let elapsed = start_time.elapsed();
                let ratio = elapsed.as_secs_f64() / segment_duration.as_secs_f64();
                let ideal_count = segment_start_count as f64 + (change as f64 * ratio);
                let diff = ideal_count - current_count as f64;
                let delta_int: isize = if diff >= 1.0 {
                    diff.floor() as isize
                } else if diff <= -1.0 {
                    diff.ceil() as isize
                } else {
                    0
                };

                use std::cmp::Ordering;

                match delta_int.cmp(&0) {
                    Ordering::Greater => {
                        for _ in 0..delta_int {
                            let mut vu =
                                VirtualUser::new(&self.config.url, self.config.rps_window_size)
                                    .set_graceful_shutdown(self.config.graceful_shutdown);
                            vu.start();
                            self.running_vus.push(vu);
                            current_count += 1;
                        }
                    }
                    Ordering::Less => {
                        let num_to_remove = (-delta_int) as usize;
                        for _ in 0..num_to_remove {
                            if let Some(mut vu) = self.running_vus.pop() {
                                vu.stop().await;
                                let metrics = vu.metrics();
                                let m = metrics.lock().await;
                                Self::merge_metrics(&mut self.overall_metrics, &m);
                                current_count -= 1;
                            }
                        }
                    }
                    Ordering::Equal => {}
                }

                sleep(tick_interval).await;
            }

            while current_count < target_count {
                let mut vu = VirtualUser::new(&self.config.url, self.config.rps_window_size)
                    .set_graceful_shutdown(self.config.graceful_shutdown);
                vu.start();
                self.running_vus.push(vu);
                current_count += 1;
            }
            while current_count > target_count {
                if let Some(mut vu) = self.running_vus.pop() {
                    vu.stop().await;
                    let metrics = vu.metrics();
                    let m = metrics.lock().await;
                    Self::merge_metrics(&mut self.overall_metrics, &m);
                    current_count -= 1;
                }
            }
        }

        while let Some(mut vu) = self.running_vus.pop() {
            vu.stop().await;
            let metrics = vu.metrics();
            let m = metrics.lock().await;
            Self::merge_metrics(&mut self.overall_metrics, &m);
        }
    }

    pub fn get_overall_metrics(&self) -> &Metrics {
        &self.overall_metrics
    }

    fn merge_metrics(dest: &mut Metrics, src: &Metrics) {
        Self::merge_summary(&mut dest.total_latency, &src.total_latency);
        Self::merge_summary(&mut dest.tcp_connect_time, &src.tcp_connect_time);
        Self::merge_summary(&mut dest.tls_handshake_time, &src.tls_handshake_time);
        Self::merge_summary(&mut dest.http_request_time, &src.http_request_time);
        dest.total_errors += src.total_errors;
        Self::merge_summary(&mut dest.error_rates_per_sec, &src.error_rates_per_sec);
        for (code, count) in &src.status_code_counts {
            *dest.status_code_counts.entry(*code).or_insert(0) += count;
        }
        dest.other_errors.extend(src.other_errors.iter().cloned());
    }

    fn merge_summary(dest: &mut Summary, src: &Summary) {
        dest.min = dest.min.min(src.min);
        dest.max = dest.max.max(src.max);
        dest.sum += src.sum;
        dest.count += src.count;
    }
}
