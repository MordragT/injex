#[cfg(target_os = "linux")]
use procfs::process::Process as LinuxProcess;

#[cfg(target_os = "linux")]
pub use procfs::process::{MMapPath, MemoryMap};

use error::ProcessResult;

pub mod error;

pub trait Process {
    /// Returns the process id
    fn pid(&self) -> i32;
    /// Returns the Memory Maps
    #[cfg(target_os = "linux")]
    fn memory_maps(&self) -> ProcessResult<Vec<MemoryMap>> {
        let process = LinuxProcess::new(self.pid())?;
        let maps = process.maps()?;
        Ok(maps)
    }
}
