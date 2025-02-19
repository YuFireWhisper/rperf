use std::collections::HashMap;
use std::time::Duration;

use super::rps_summary::RpsSummary;
use super::summary::Summary;

#[derive(Debug, Default)]
pub struct Metrics {
    total_latency: Summary,
    tcp_connect_time: Summary,
    tls_handshake_time: Summary,
    http_request_time: Summary,
    rps_summary: RpsSummary,
    total_errors: usize,
    error_rates_per_sec: Summary,
    status_code_counts: HashMap<u16, usize>,
    other_errors: Vec<String>,
}

impl Metrics {
    pub fn new(rps_window_size: Duration) -> Self {
        Self {
            rps_summary: RpsSummary::new(rps_window_size),
            ..Default::default()
        }
    }
}
