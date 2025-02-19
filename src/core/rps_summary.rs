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

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_increment_without_start() {
        let mut rps = RpsSummary::new(Duration::from_secs(1));
        assert!(rps.increment_request_count().is_err());
        assert!(rps.get_current_rps().is_err());
        assert!(rps.get_average_rps().is_err());
        assert!(rps.get_all_rps().is_err());
    }

    #[test]
    fn test_increment_and_get_current_rps() {
        let window = Duration::from_secs(1);
        let mut rps = RpsSummary::new(window);
        rps.start();
        assert!(rps.increment_request_count().is_ok());

        let current_rps = rps.get_current_rps().unwrap().unwrap();
        assert!((current_rps - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_increment_in_multiple_windows() {
        let window = Duration::from_millis(10);
        let mut rps = RpsSummary::new(window);
        rps.start();

        assert!(rps.increment_request_count().is_ok());

        sleep(Duration::from_millis(15));

        assert!(rps.increment_request_count().is_ok());

        let rps_vec = rps.get_all_rps().unwrap();

        assert!(rps_vec.len() >= 2);
        assert!(rps_vec[0] > 0.0);
        assert!(rps_vec[1] > 0.0);
    }

    #[test]
    fn test_reset() {
        let window = Duration::from_secs(1);
        let mut rps = RpsSummary::new(window);
        rps.start();
        assert!(rps.increment_request_count().is_ok());
        rps.reset();
        assert!(rps.request_counts.is_empty());
        assert!(rps.start_time.is_none());
    }
}
