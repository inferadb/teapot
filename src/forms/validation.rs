//! Validation utilities for form fields.

use std::sync::Arc;

/// A validation error message.
pub type ValidationError = String;

/// A validation result.
pub type ValidationResult = Result<(), ValidationError>;

/// A validator function type.
pub type ValidatorFn<T> = Arc<dyn Fn(&T) -> ValidationResult + Send + Sync>;

/// Wrapper for validator functions.
#[derive(Clone)]
pub struct Validator<T> {
    func: ValidatorFn<T>,
}

impl<T> Validator<T> {
    /// Create a new validator from a function.
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&T) -> ValidationResult + Send + Sync + 'static,
    {
        Self { func: Arc::new(f) }
    }

    /// Validate a value.
    pub fn validate(&self, value: &T) -> ValidationResult {
        (self.func)(value)
    }
}

impl<T> std::fmt::Debug for Validator<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Validator")
    }
}

/// Built-in validators for common cases.
pub mod validators {
    use super::*;

    /// Validate that a string is not empty.
    pub fn required() -> Validator<String> {
        Validator::new(|s: &String| {
            if s.trim().is_empty() {
                Err("This field is required".to_string())
            } else {
                Ok(())
            }
        })
    }

    /// Validate minimum length.
    pub fn min_length(min: usize) -> Validator<String> {
        Validator::new(move |s: &String| {
            if s.len() < min {
                Err(format!("Must be at least {} characters", min))
            } else {
                Ok(())
            }
        })
    }

    /// Validate maximum length.
    pub fn max_length(max: usize) -> Validator<String> {
        Validator::new(move |s: &String| {
            if s.len() > max {
                Err(format!("Must be at most {} characters", max))
            } else {
                Ok(())
            }
        })
    }

    /// Validate email format (simple check).
    pub fn email() -> Validator<String> {
        Validator::new(|s: &String| {
            if s.contains('@') && s.contains('.') {
                Ok(())
            } else {
                Err("Invalid email address".to_string())
            }
        })
    }

    /// Validate URL format (simple check).
    pub fn url() -> Validator<String> {
        Validator::new(|s: &String| {
            if s.starts_with("http://") || s.starts_with("https://") {
                Ok(())
            } else {
                Err("Must be a valid URL".to_string())
            }
        })
    }

    /// Validate that value matches a pattern.
    pub fn pattern(regex: &str, message: &str) -> Validator<String> {
        let pattern = regex.to_string();
        let msg = message.to_string();
        Validator::new(move |s: &String| {
            // Simple pattern matching without regex crate
            // Just checks if the pattern is contained
            if s.contains(&pattern) {
                Ok(())
            } else {
                Err(msg.clone())
            }
        })
    }

    /// Compose multiple validators.
    pub fn all<T: Clone + Send + Sync + 'static>(validators: Vec<Validator<T>>) -> Validator<T> {
        Validator::new(move |value: &T| {
            for v in &validators {
                v.validate(value)?;
            }
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::validators::*;

    #[test]
    fn test_required() {
        let v = required();
        assert!(v.validate(&"hello".to_string()).is_ok());
        assert!(v.validate(&"".to_string()).is_err());
        assert!(v.validate(&"   ".to_string()).is_err());
    }

    #[test]
    fn test_min_length() {
        let v = min_length(3);
        assert!(v.validate(&"abc".to_string()).is_ok());
        assert!(v.validate(&"ab".to_string()).is_err());
    }

    #[test]
    fn test_email() {
        let v = email();
        assert!(v.validate(&"test@example.com".to_string()).is_ok());
        assert!(v.validate(&"invalid".to_string()).is_err());
    }

    #[test]
    fn test_composed() {
        let v = all(vec![required(), min_length(5)]);
        assert!(v.validate(&"hello".to_string()).is_ok());
        assert!(v.validate(&"hi".to_string()).is_err());
        assert!(v.validate(&"".to_string()).is_err());
    }
}
