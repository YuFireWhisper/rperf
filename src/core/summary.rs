#[derive(Debug, Clone, Default)]
pub struct Summary {
    min: f64,
    max: f64,
    sum: f64,
    count: usize,
}

impl Summary {
    pub fn new() -> Self {
        Summary {
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
