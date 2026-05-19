use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct DiagramSource(String);

impl DiagramSource {
    pub fn parse(value: String) -> Result<Self, DiagramError> {
        if value.trim().is_empty() {
            return Err(DiagramError::EmptySource);
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DiagramValidation {
    pub line_count: usize,
    pub character_count: usize,
}

#[derive(Debug, Error)]
pub enum DiagramError {
    #[error("diagram source is empty")]
    EmptySource,
}
