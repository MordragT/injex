use crate::memory::{error::MemoryResult, MemoryManipulation};
use crate::process::Process;

pub struct InternalManipulator;

impl MemoryManipulation for InternalManipulator {
    fn read(&self, address: usize, buf: &mut [u8]) -> MemoryResult<usize> {
        let pointer = unsafe { std::mem::transmute::<usize, *const u8>(address) };
        //let slice_ptr = std::ptr::slice_from_raw_parts(pointer, buf.len());
        unsafe { std::ptr::copy(pointer, buf[0] as *mut u8, buf.len()) };
        Ok(buf.len())
    }
    fn write(&self, address: usize, payload: &[u8]) -> MemoryResult<usize> {
        let pointer = unsafe { std::mem::transmute::<usize, *mut u8>(address) };
        //let slice_ptr = std::ptr::slice_from_raw_parts_mut(pointer, payload.len());
        unsafe { std::ptr::copy(payload[0] as *const u8, pointer, payload.len()) };
        Ok(payload.len())
    }
}

impl Process for InternalManipulator {
    fn pid(&self) -> i32 {
        std::process::id() as i32
    }
}
