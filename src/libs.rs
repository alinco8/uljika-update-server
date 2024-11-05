use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct CustomError {
    pub message: String,
}
impl CustomError {
    pub fn new(error: &str) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}
impl Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for CustomError {}
