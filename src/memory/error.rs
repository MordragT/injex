use std::fmt;

pub type MemoryResult<T> = Result<T, MemoryError>;

/// Memory Error type
#[derive(Debug)]
pub enum MemoryError {
    InternalError(String),
    ReadError(String),
    WriteError(String),
    WildcardError(String),
    IoError(std::io::Error),
}

impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryError::InternalError(s) => f.write_str(&format!("INTERNAL ERROR: {}", s)),
            MemoryError::ReadError(s) => f.write_str(&format!("Read Error: {}", s)),
            MemoryError::WriteError(s) => f.write_str(&format!("Write Error: {}", s)),
            MemoryError::WildcardError(s) => f.write_str(&format!("Wildcard Error: {}", s)),
            MemoryError::IoError(e) => f.write_str(&e.to_string()),
        }
    }
}

impl std::error::Error for MemoryError {}

impl From<std::io::Error> for MemoryError {
    fn from(e: std::io::Error) -> Self {
        MemoryError::IoError(e)
    }
}
