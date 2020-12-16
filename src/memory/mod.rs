pub mod error;

use error::MemoryResult;
use regex::Regex;

fn is_sub(mut haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.len() == 0 {
        return Some(0);
    }
    let mut offset = 0;
    while !haystack.is_empty() {
        if haystack.starts_with(needle) {
            return Some(offset);
        }
        haystack = &haystack[1..];
        offset += 1;
    }
    None
}

fn wildcard_is_sub(mut haystack: &[u8], needle: &[Option<u8>]) -> Option<usize> {
    if needle.len() == 0 {
        return Some(0);
    }
    let mut offset = 0;
    while !haystack.is_empty() {
        let starts_with = needle
            .iter()
            .zip(haystack.iter())
            .map(|(n, h)| {
                if let Some(val) = n {
                    match val == h {
                        true => Some(()),
                        false => None,
                    }
                } else {
                    Some(())
                }
            })
            .collect::<Option<()>>();
        if starts_with == Some(()) && haystack.len() >= needle.len() {
            return Some(offset);
        }
        haystack = &haystack[1..];
        offset += 1;
    }
    None
}

pub trait MemoryManipulation {
    fn read(&self, address: usize, buf: &mut [u8]) -> MemoryResult<usize>;
    fn write(&self, address: usize, payload: &[u8]) -> MemoryResult<usize>;
    fn read_structure<T: Clone>(&self, address: usize) -> MemoryResult<T> {
        let mut buf = vec![0_u8; std::mem::size_of::<T>()];
        self.read(address, &mut buf)?;
        Ok(unsafe { std::mem::transmute::<*const u8, &T>(&buf[0] as *const u8) }.clone())
    }
    fn write_structure<T>(&self, address: usize, payload: T) -> MemoryResult<usize> {
        let payload = unsafe { std::mem::transmute::<&T, *const u8>(&payload) };
        self.write(address, unsafe {
            std::slice::from_raw_parts(payload, std::mem::size_of::<T>())
        })
    }
    // fn find_len(
    //     &self,
    //     start: usize,
    //     current: usize,
    //     end: usize,
    //     buffer_len: usize,
    //     signature: &[u8],
    // ) -> Option<usize> {
    //     if buffer_len > (end - start) / 4 {
    //         return None;
    //     }
    //     if current + buffer_len > end {
    //         return self.find_len(start, current, end, buffer_len * 2, signature);
    //         return None;
    //     }
    //     let mut buffer = vec![0_u8; buffer_len];
    //     match self.read(current, &mut buffer) {
    //         Ok(_) => (),
    //         Err(_) => return None,
    //     }
    //     match is_sub(&mut buffer, signature) {
    //         Some(addr) => Some(current + addr),
    //         None => self.find_len(current, current + buffer_len, end, buffer_len, signature),
    //     }
    // }
    // fn find(&self, start: usize, end: usize, signature: &[u8]) -> Option<usize> {
    //     self.find_len(
    //         start,
    //         start,
    //         end,
    //         std::mem::size_of_val(signature) * 4,
    //         signature,
    //     )
    // }
    /// Not guranteed to find signature, until Rust supports Tail Call Optimization cause i am lazy
    fn find(&self, start: usize, end: usize, signature: &[u8]) -> Option<usize> {
        (start..end)
            .step_by(std::mem::size_of_val(signature) * 4) // here i cannot gurantee that i do not slice into the searched signature
            .into_iter()
            .find_map(|addr| {
                let mut buffer = vec![0_u8; std::mem::size_of_val(signature) * 4];
                match self.read(addr, &mut buffer) {
                    Ok(_) => (),
                    Err(_) => return None,
                }
                match is_sub(&mut buffer, signature) {
                    Some(inner) => Some(addr + inner),
                    None => None,
                }
            })
    }
    /// Not guranteed to find signature, until Rust supports Tail Call Optimization cause i am lazy
    fn find_wildcard(&self, start: usize, end: usize, signature: &str) -> Option<usize> {
        let re = Regex::new(r"[[:xdigit:]][[:xdigit:]]|::").unwrap();
        let signature = re
            .captures_iter(signature)
            .into_iter()
            .map(|caps| {
                caps.get(0)
                    .map(|m| match u8::from_str_radix(m.as_str(), 16) {
                        Ok(num) => Some(num),
                        Err(_) => None,
                    })
                    .unwrap_or(None)
            })
            .collect::<Vec<Option<u8>>>();
        (start..end)
            .step_by(signature.len() * 4) // here i cannot gurantee that i do not slice into the searched signature
            .into_iter()
            .find_map(|addr| {
                let mut buffer = vec![0_u8; signature.len() * 4];
                match self.read(addr, &mut buffer) {
                    Ok(_) => (),
                    Err(_) => return None,
                }
                match wildcard_is_sub(&mut buffer, &signature) {
                    Some(inner) => Some(addr + inner),
                    None => None,
                }
            })
    }
}
