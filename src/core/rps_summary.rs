use std::time::{Duration, Instant};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RpsSummaryError {
    #[error("RpsSummary is not started")]
    NotStarted,
    #[error("Request count is empty")]
    EmptyRequestCount,
}

type Result<T> = std::result::Result<T, RpsSummaryError>;

#[derive(Debug, Default)]
pub struct RpsSummary {
    request_counts: Vec<usize>,
    window_size: Duration,
    start_time: Option<Instant>,
}

impl RpsSummary {
    pub fn new(window_size: Duration) -> Self {
        Self {
            request_counts: Vec::new(),
            window_size,
            start_time: None,
        }
    }

    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    pub fn increment_request_count(&mut self) -> Result<()> {
        if self.start_time.is_none() {
            return Err(RpsSummaryError::NotStarted);
        }
        let start_time = self.start_time.unwrap();

        let elapsed = start_time.elapsed();
        let window_index = (elapsed.as_nanos() / self.window_size.as_nanos()) as usize;

        if window_index >= self.request_counts.len() {
            self.request_counts.resize(window_index + 1, 0);
        }

        self.request_counts[window_index] += 1;

        Ok(())
    }

    pub fn get_current_rps(&self) -> Result<Option<f64>> {
        if self.start_time.is_none() {
            return Err(RpsSummaryError::NotStarted);
        }

        if self.request_counts.is_empty() {
            return Err(RpsSummaryError::EmptyRequestCount);
        }

        let start_time = self.start_time.unwrap();

        let elapsed = start_time.elapsed();
        let window_index = (elapsed.as_nanos() / self.window_size.as_nanos()) as usize;

        if window_index < self.request_counts.len() {
            Ok(Some(
                self.request_counts[window_index] as f64 / self.window_size.as_secs_f64(),
            ))
        } else {
            Ok(None)
        }
    }

    pub fn get_average_rps(&self) -> Result<Option<f64>> {
        if self.start_time.is_none() {
            return Err(RpsSummaryError::NotStarted);
        }

        if self.request_counts.is_empty() {
            return Err(RpsSummaryError::EmptyRequestCount);
        }

        let total_requests: usize = self.request_counts.iter().sum();
        let elapsed = self.start_time.unwrap().elapsed();

        Ok(Some(total_requests as f64 / elapsed.as_secs_f64()))
    }

    pub fn get_all_rps(&self) -> Result<Vec<f64>> {
        if self.start_time.is_none() {
            return Err(RpsSummaryError::NotStarted);
        }

        if self.request_counts.is_empty() {
            return Err(RpsSummaryError::EmptyRequestCount);
        }

        let rps_vec = self
            .request_counts
            .iter()
            .map(|&count| count as f64 / self.window_size.as_secs_f64())
            .collect();

        Ok(rps_vec)
    }

    pub fn reset(&mut self) {
        self.request_counts.clear();
        self.start_time = None;
    }
}
