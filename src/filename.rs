#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilenameValidationError {
    Empty,
    InvalidChars,
}

pub fn validate_filename(file_name: &str) -> Result<(), FilenameValidationError> {
    if file_name.is_empty() {
        return Err(FilenameValidationError::Empty);
    }

    if file_name.contains('/') || file_name.contains('\\') || file_name.contains("..") {
        return Err(FilenameValidationError::InvalidChars);
    }

    Ok(())
}
