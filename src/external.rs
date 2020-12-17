use crate::error::{Error, Result};
use crate::memory::{error::MemoryError, error::MemoryResult, MemoryManipulation};
use crate::process::Process;
use sysinfo::{ProcessExt, System, SystemExt};

#[cfg(target_os = "linux")]
use {
    nix::{
        sys::uio::{self, IoVec, RemoteIoVec},
        unistd::Pid,
    },
    std::{
        fs::OpenOptions,
        io::{prelude::*, SeekFrom},
    },
};

pub struct ExternalManipulator {
    pid: i32,
}

impl ExternalManipulator {
    pub fn new_unchecked(pid: i32) -> ExternalManipulator {
        ExternalManipulator { pid }
    }
    pub fn new(name: &str) -> Result<ExternalManipulator> {
        let system = System::new_all();
        let process_list = system.get_process_by_name(name);
        if !process_list.is_empty() {
            let pid = process_list[0].pid();
            return Ok(ExternalManipulator { pid });
        }
        Err(Error::ProcessNotFound(name.to_owned()))
    }
}

#[cfg(target_os = "linux")]
impl MemoryManipulation for ExternalManipulator {
    fn read(&self, address: usize, buf: &mut [u8]) -> MemoryResult<usize> {
        let remote = [RemoteIoVec {
            base: address,
            len: std::mem::size_of_val(buf),
        }];
        let local = [IoVec::from_mut_slice(buf)];
        match uio::process_vm_readv(Pid::from_raw(self.pid), &local, &remote) {
            Ok(x) if x > 0 => Ok(x),
            Err(e) => Err(MemoryError::ReadError(e.to_string())),
            _ => Err(MemoryError::ReadError("No bytes read".to_owned())),
        }
    }
    fn write(&self, address: usize, payload: &[u8]) -> MemoryResult<usize> {
        let remote = [RemoteIoVec {
            base: address,
            len: std::mem::size_of_val(payload),
        }];
        let local = [IoVec::from_slice(payload)];
        match uio::process_vm_writev(Pid::from_raw(self.pid), &local, &remote) {
            Ok(x) if x > 0 => Ok(x),
            Err(e) => Err(MemoryError::WriteError(e.to_string())),
            _ => Err(MemoryError::WriteError("No bytes written".to_owned())),
        }
    }
}

impl Process for ExternalManipulator {
    fn pid(&self) -> i32 {
        self.pid
    }
}
#[cfg(target_os = "linux")]
pub struct AnonManipulator {
    pid: i32,
}

impl AnonManipulator {
    pub fn new(name: &str) -> Result<AnonManipulator> {
        let system = System::new_all();
        let process_list = system.get_process_by_name(name);
        if !process_list.is_empty() {
            let pid = process_list[0].pid();
            return Ok(AnonManipulator { pid });
        }
        Err(Error::ProcessNotFound(name.to_owned()))
    }
}

#[cfg(target_os = "linux")]
impl MemoryManipulation for AnonManipulator {
    fn read(&self, address: usize, buf: &mut [u8]) -> MemoryResult<usize> {
        let mut mem_file = OpenOptions::new()
            .read(true)
            .open(format!("/proc/{}/mem", self.pid))?;
        mem_file.seek(SeekFrom::Start(address as u64))?;
        mem_file.read_exact(buf)?;
        Ok(buf.len())
    }
    fn write(&self, address: usize, payload: &[u8]) -> MemoryResult<usize> {
        let mut mem_file = OpenOptions::new()
            .write(true)
            .append(false)
            .open(format!("/proc/{}/mem", self.pid))?;
        mem_file.seek(SeekFrom::Start(address as u64))?;
        mem_file.write_all(payload)?;
        Ok(payload.len())
    }
}

impl Process for AnonManipulator {
    fn pid(&self) -> i32 {
        self.pid
    }
}
