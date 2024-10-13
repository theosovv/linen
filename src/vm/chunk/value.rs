#[derive(Clone)]
pub struct Value {
    pub values: Vec<f64>,
}

impl Default for Value {
    fn default() -> Self {
        Self::new()
    }
}

impl Value {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn write(&mut self, value: f64) {
        self.values.push(value);
    }

    pub fn free_value(&mut self) {
        self.values.clear();
    }
}
