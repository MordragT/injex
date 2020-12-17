use procfs::ProcError;
use std::fmt;

pub type ProcessResult<T> = Result<T, ProcessError>;

#[derive(Debug)]
pub enum ProcessError {
    InternalError(String),
    IoError(std::io::Error),
    ProcessNotFound(i32),
    ProcFsError(ProcError),
}

impl fmt::Display for ProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessError::InternalError(s) => f.write_str(&format!("INTERNAL ERROR: {}", s)),
            ProcessError::IoError(e) => f.write_str(&e.to_string()),
            ProcessError::ProcessNotFound(pid) => {
                f.write_str(&format!("Process not found: {}", pid))
            }
            ProcessError::ProcFsError(e) => f.write_str(&e.to_string()),
        }
    }
}

impl std::error::Error for ProcessError {}

impl From<std::io::Error> for ProcessError {
    fn from(e: std::io::Error) -> Self {
        ProcessError::IoError(e)
    }
}

impl From<ProcError> for ProcessError {
    fn from(e: ProcError) -> Self {
        ProcessError::ProcFsError(e)
    }
}
