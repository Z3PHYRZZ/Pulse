use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct History {
    capacity: usize,
    values: VecDeque<f32>,
}

impl History {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(2),
            values: VecDeque::with_capacity(capacity.max(2)),
        }
    }

    pub fn push(&mut self, value: f32) {
        if self.values.len() == self.capacity {
            self.values.pop_front();
        }
        self.values.push_back(value.clamp(0.0, 100.0));
    }

    pub fn resize(&mut self, capacity: usize) {
        self.capacity = capacity.max(2);
        while self.values.len() > self.capacity {
            self.values.pop_front();
        }
    }

    pub fn values(&self) -> impl Iterator<Item = f32> + '_ {
        self.values.iter().copied()
    }

    #[allow(dead_code)]
    pub fn latest(&self) -> f32 {
        self.values.back().copied().unwrap_or(0.0)
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}
