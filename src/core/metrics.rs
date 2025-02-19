use std::collections::HashMap;
use std::time::Duration;

use super::rps_summary::RpsSummary;
use super::summary::Summary;

#[derive(Debug, Default)]
pub struct Metrics {
    pub total_latency: Summary,
    pub tcp_connect_time: Summary,
    pub tls_handshake_time: Summary,
    pub http_request_time: Summary,
    pub rps_summary: RpsSummary,
    pub total_errors: usize,
    pub error_rates_per_sec: Summary,
    pub status_code_counts: HashMap<u16, usize>,
    pub other_errors: Vec<String>,
}

impl Metrics {
    pub fn new(rps_window_size: Duration) -> Self {
        Self {
            rps_summary: RpsSummary::new(rps_window_size),
            ..Default::default()
        }
    }
}
