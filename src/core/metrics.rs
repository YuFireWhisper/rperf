// metrics.rs
use std::collections::HashMap;
use std::time::Duration;

use super::rps_summary::RpsSummary;
use super::summary::Summary;

#[derive(Debug)]
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

impl Default for Metrics {
    fn default() -> Self {
        Self {
            total_latency: Summary::new(),
            tcp_connect_time: Summary { min: 0.0, max: 0.0, sum: 0.0, count: 0 },
            tls_handshake_time: Summary { min: 0.0, max: 0.0, sum: 0.0, count: 0 },
            http_request_time: Summary::new(),
            rps_summary: RpsSummary::default(),
            total_errors: 0,
            error_rates_per_sec: Summary::new(),
            status_code_counts: HashMap::new(),
            other_errors: Vec::new(),
        }
    }
}

impl Metrics {
    pub fn new(rps_window_size: Duration) -> Self {
        Self {
            total_latency: Summary::new(),
            tcp_connect_time: Summary { min: 0.0, max: 0.0, sum: 0.0, count: 0 },
            tls_handshake_time: Summary { min: 0.0, max: 0.0, sum: 0.0, count: 0 },
            http_request_time: Summary::new(),
            rps_summary: RpsSummary::new(rps_window_size),
            total_errors: 0,
            error_rates_per_sec: Summary::new(),
            status_code_counts: HashMap::new(),
            other_errors: Vec::new(),
        }
    }
}

