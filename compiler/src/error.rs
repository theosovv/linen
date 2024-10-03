pub struct LinenError {
    message: String,
    line: usize,
    source: String,
}
impl LinenError {
    pub fn new(message: String, line: usize, source: String) -> Self {
        Self {
            message,
            line,
            source,
        }
    }
    pub fn report(&self) {
        println!(
            "[line {}] Error at {}: {}",
            self.line, self.source, self.message
        );
    }
}
