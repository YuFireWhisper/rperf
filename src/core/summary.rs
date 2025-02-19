#[derive(Debug, Clone, Default)]
pub struct Summary {
    pub min: f64,
    pub max: f64,
    pub sum: f64,
    pub count: usize,
}

impl Summary {
    pub fn new() -> Self {
        Self {
            min: f64::MAX,
            max: f64::MIN,
            sum: 0.0,
            count: 0,
        }
    }

    pub fn update(&mut self, value: f64) {
        self.min = self.min.min(value);
        self.max = self.max.max(value);
        self.sum += value;
        self.count += 1;
    }

    pub fn update_optional(&mut self, value: Option<f64>) {
        if let Some(v) = value {
            self.min = self.min.min(v);
            self.max = self.max.max(v);
            self.sum += v;
            self.count += 1;
        }
    }

    pub fn average(&self) -> Option<f64> {
        if self.count > 0 {
            Some(self.sum / self.count as f64)
        } else {
            None
        }
    }

    pub fn min(&self) -> Option<f64> {
        if self.count > 0 {
            Some(self.min)
        } else {
            None
        }
    }

    pub fn max(&self) -> Option<f64> {
        if self.count > 0 {
            Some(self.max)
        } else {
            None
        }
    }

    pub fn sum(&self) -> f64 {
        self.sum
    }

    pub fn count(&self) -> usize {
        self.count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_summary() {
        let summary = Summary::new();
        assert_eq!(summary.count(), 0);
        assert_eq!(summary.sum(), 0.0);
        assert_eq!(summary.min(), None);
        assert_eq!(summary.max(), None);
        assert_eq!(summary.average(), None);
    }

    #[test]
    fn test_update() {
        let mut summary = Summary::new();
        summary.update(10.0);
        summary.update(5.0);
        summary.update(20.0);

        assert_eq!(summary.count(), 3);
        assert_eq!(summary.sum(), 35.0);
        assert_eq!(summary.min(), Some(5.0));
        assert_eq!(summary.max(), Some(20.0));
        assert_eq!(summary.average(), Some(35.0 / 3.0));
    }

    #[test]
    fn test_update_optional() {
        let mut summary = Summary::new();
        summary.update_optional(Some(15.0));
        summary.update_optional(None);
        summary.update_optional(Some(5.0));

        assert_eq!(summary.count(), 2);
        assert_eq!(summary.sum(), 20.0);
        assert_eq!(summary.min(), Some(5.0));
        assert_eq!(summary.max(), Some(15.0));
    }
}

