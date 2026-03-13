// Data Validator

use crate::error::DataResult;

/// Validator trait - data validator
pub trait Validator<T> {
    fn validate(&self, data: &T) -> DataResult<()>;
}

/// ValidationResult - validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: vec![],
        }
    }

    pub fn invalid(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            errors,
        }
    }
}
