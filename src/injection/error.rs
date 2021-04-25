use crate::memory::error::MemoryError;
use std::fmt;
use std::io::Error as IoError;
use std::num::ParseIntError;

#[cfg(target_os = "linux")]
use dynasmrt::DynasmError;

pub type InjectionResult<T> = Result<T, InjectionError>;

/// Injection Error Type
#[derive(Debug)]
pub enum InjectionError {
    InternalError(String),
    ProcessNotFound(i32),
    LibraryNotFound(String),
    SymbolNotFound(String),
    IoError(IoError),
    ParseIntError(ParseIntError),
    #[cfg(target_os = "linux")]
    DynasmError(DynasmError),
    MemoryError(MemoryError),
}

impl fmt::Display for InjectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InjectionError::InternalError(s) => f.write_str(&format!("INTERNAL ERROR: {}", s)),
            InjectionError::ProcessNotFound(pid) => {
                f.write_str(&format!("Process was not found, pid: {}", pid))
            }
            InjectionError::LibraryNotFound(lib) => {
                f.write_str(&format!("Library was not found: {}", lib))
            }
            InjectionError::SymbolNotFound(sym) => {
                f.write_str(&format!("Symbol was not found: {}", sym))
            }
            InjectionError::IoError(e) => f.write_str(&e.to_string()),
            InjectionError::ParseIntError(e) => f.write_str(&e.to_string()),
            #[cfg(target_os = "linux")]
            InjectionError::DynasmError(e) => f.write_str(&e.to_string()),
            InjectionError::MemoryError(e) => f.write_str(&e.to_string()),
        }
    }
}

impl std::error::Error for InjectionError {}

impl From<IoError> for InjectionError {
    fn from(e: IoError) -> Self {
        InjectionError::IoError(e)
    }
}

impl From<ParseIntError> for InjectionError {
    fn from(e: ParseIntError) -> Self {
        InjectionError::ParseIntError(e)
    }
}

#[cfg(target_os = "linux")]
impl From<DynasmError> for InjectionError {
    fn from(e: DynasmError) -> Self {
        InjectionError::DynasmError(e)
    }
}

impl From<MemoryError> for InjectionError {
    fn from(e: MemoryError) -> Self {
        InjectionError::MemoryError(e)
    }
}
