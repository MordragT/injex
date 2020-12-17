use crate::memory::MemoryManipulation;
use error::{InjectionError, InjectionResult};
use std::{fs::OpenOptions, path::Path};

#[cfg(target_os = "linux")]
use {
    dynasm::dynasm,
    dynasmrt::x64::X64Relocation,
    dynasmrt::{DynasmApi, DynasmLabelApi, VecAssembler},
    goblin::{elf::sym::Sym, Object},
    nix::{
        sys::signal::{self, Signal},
        unistd::Pid,
    },
    procfs::process::{MMapPath, MemoryMap, Process as LinuxProcess},
    regex::Regex,
    std::{fs::File, io::prelude::*},
};
pub mod error;

#[cfg(target_os = "linux")]
fn find_memory_region(regex: &str, process: &LinuxProcess) -> Option<MemoryMap> {
    let re = match Regex::new(regex) {
        Ok(re) => re,
        Err(_) => return None,
    };
    let mapped_memory_regions = match process.maps() {
        Ok(m) => m,
        Err(_) => return None,
    };
    mapped_memory_regions.into_iter().find(|x| {
        if let MMapPath::Path(path_buf) = &x.pathname {
            if re.is_match(path_buf.to_str().unwrap()) {
                return true;
            }
        }
        return false;
    })
}

#[cfg(target_os = "linux")]
fn find_elf_symbol_addr(mem_map: &MemoryMap, sym_name: &str) -> Option<u64> {
    fn find_elf_symbol(path: &MMapPath, sym_name: &str) -> Option<Sym> {
        let buffer = match path {
            MMapPath::Path(p) => match std::fs::read(p) {
                Ok(buf) => buf,
                Err(_) => return None,
            },
            _ => return None,
        };
        let elf_option = match Object::parse(&buffer) {
            Ok(Object::Elf(elf)) => Some(elf),
            _ => None,
        };
        if let Some(elf) = elf_option {
            match elf.syms.iter().find(|sym| {
                if let Some(Ok(name)) = elf.strtab.get(sym.st_name) {
                    name == sym_name
                } else {
                    false
                }
            }) {
                Some(sym) => return Some(sym),
                None => (),
            }
            elf.dynsyms.iter().find(|sym| {
                if let Some(Ok(name)) = elf.dynstrtab.get(sym.st_name) {
                    name == sym_name
                } else {
                    false
                }
            })
        } else {
            None
        }
    }
    match find_elf_symbol(&mem_map.pathname, sym_name) {
        Some(elf_symbol) => Some(mem_map.address.0 + elf_symbol.st_value),
        None => None,
    }
}
#[cfg(target_os = "linux")]
pub fn inject<P, T>(manipulator: &T, pid: i32, library: P) -> InjectionResult<()>
where
    P: AsRef<Path>,
    T: MemoryManipulation,
{
    let library_path = match library.as_ref().to_str() {
        Some(s) => {
            println!("Injection library: {}", s);
            format!("{}\0", s)
        }
        None => {
            return Err(InjectionError::LibraryNotFound(
                "user-defined library".to_owned(),
            ))
        }
    };
    const STACK_BACKUP_SIZE: usize = 8 * 16;
    const STAGE_TWO_SIZE: u32 = 0x8000;

    let process = match LinuxProcess::new(pid) {
        Ok(proc) => proc,
        Err(_) => return Err(InjectionError::ProcessNotFound(pid)),
    };
    let ld = match find_memory_region(r".*/ld-.*\.so", &process) {
        Some(mem_map) => mem_map,
        None => return Err(InjectionError::LibraryNotFound("ld.so".to_owned())),
    };
    // let librt = match find_memory_region(r".*/librt-.*\.so", &process) {
    //     Some(mem_map) => mem_map,
    //     None => return Err(InjectionError::LibraryNotFound("librt.so".to_owned())),
    // };
    let dl_open_address = match find_elf_symbol_addr(&ld, "_dl_open") {
        Some(addr) => addr,
        None => return Err(InjectionError::SymbolNotFound("_dl_open".to_owned())),
    };
    // let shm_open_address = match find_elf_symbol_addr(&librt, "shm_open") {
    //     Some(addr) => addr,
    //     None => return Err(InjectionError::SymbolNotFound("shm_open".to_owned())),
    // };
    // let shm_unlink_address = match find_elf_symbol_addr(&librt, "shm_unlink") {
    //     Some(addr) => addr,
    //     None => return Err(InjectionError::SymbolNotFound("shm_unlink".to_owned())),
    // };

    match signal::kill(Pid::from_raw(pid), Signal::SIGSTOP) {
        Ok(_) => (),
        Err(e) => return Err(InjectionError::InternalError(e.to_string())),
    }

    // wait for process to stop
    while process.status().unwrap().state != "T (stopped)"
        && process.status().unwrap().state != "t (tracing stop)"
    {}

    let mut syscall_file = File::open(format!("/proc/{}/syscall", pid))?;
    let mut syscall_buffer = String::new();
    syscall_file.read_to_string(&mut syscall_buffer)?;
    syscall_buffer.pop();
    let syscall_buffer: Vec<&str> = syscall_buffer.rsplit(" ").collect();

    let current_rip = usize::from_str_radix(syscall_buffer[0].trim_start_matches("0x"), 16)?;
    let current_rsp = usize::from_str_radix(syscall_buffer[1].trim_start_matches("0x"), 16)?;

    //println!("Instruction Pointer: {:x}", current_rip);
    let mut ops: VecAssembler<X64Relocation> = VecAssembler::new(0x00);

    dynasm!(ops
        ; pushf
        ; push rax
        ; push rbx
        ; push rcx
        ; push rdx
        ; push rbp
        ; push rsi
        ; push rdi
        ; push r8
        ; push r9
        ; push r10
        ; push r11
        ; push r12
        ; push r13
        ; push r14
        ; push r15

        // Open shared memory object: stage two
        // funzt nicht gibt ffffffff
        // ; lea rdi, [>shared_object]
        // ; mov rsi, 2
        // ; mov rdx, 0400 // sollte egal sein
        // ; mov rax, QWORD shm_open_address as i64
        // ; call rax
        // ; mov r14, rax

        ; mov rax, 2 // SYS_OPEN
        ; lea rdi, [>shared_object]
        ; xor rsi, rsi // O_RDONLY
        ; xor rdx, rdx // Mode sollte egal sein bei 0_RDONLY
        ; syscall
        ; mov r14, rax // Save the fd

        // mmap it
        ; mov rax, 9 // SYS_MMAP
        ; xor rdi, rdi // addr
        ; mov rsi, STAGE_TWO_SIZE as i32 // len
        ; mov rdx, 0x7 // prot (rwx)
        ; mov r10, 0x2 // flags (MAP_PRIVATE)
        ; mov r8, r14 // fd
        ; xor r9, r9 // off
        ; syscall
        ; mov r15, rax // save mmap addr

        // close the file
        ; mov rax, 3 // SYS_CLOSE
        ; mov rdi, r14 // fd
        ; syscall

        // // Unlink shared memory object
        // ; lea rdi, [>shared_object]
        // ; mov rax, QWORD shm_unlink_address as i64
        // ; call rax

        // Delete the file
        ; mov rax, 87 // SYS_UNLINK
        ; lea rdi, [>shared_object]
        ; syscall

        // Jump to Stage two
        ; jmp r15

        ; shared_object:
        ; .bytes "/tmp/stage_two.bin\0".as_bytes()
    );

    let shell_code_buf = ops.finalize()?;

    let mut code_backup = vec![0_u8; shell_code_buf.len()];
    manipulator.read(current_rip, &mut code_backup)?;
    let mut stack_backup = [0_u8; STACK_BACKUP_SIZE];
    manipulator.read(current_rsp - STACK_BACKUP_SIZE, &mut stack_backup)?;

    let mut ops: VecAssembler<X64Relocation> = VecAssembler::new(0x00);

    dynasm!(ops
        ; cld
        ; fxsave [>moar_regs]

        // Open /proc/self/mem
        ; mov rax, 2 // SYS_OPEM
        ; lea rdi, [>proc_self_mem]
        ; mov rsi, 2 // flags (O_RDWR)
        ; xor rdx, rdx
        ; syscall
        ; mov r15, rax // save the fd

        // seek to code
        ; mov rax, 8 // SYS_LSEEK
        ; mov rdi, r15 // fd
        ; mov rsi, QWORD current_rip as i64 // offset
        ; xor rdx, rdx // whence (LEEK_SET)
        ; syscall

        // restore code
        ; mov rax, 1 // SYS_WRITE
        ; mov rdi, r15 // fd
        ; lea rsi, [>old_code] // backup buffer
        ; mov rdx, code_backup.len() as i32 // count
        ; syscall

        // close /proc/self/mem
        ; mov rax, 3 // SYS_CLOSE
        ; mov rdi, r15 // fd
        ; syscall

        // move pushed regs to our new stack
        ; lea rdi, [>new_stack_base - (STACK_BACKUP_SIZE as isize)]
        ; mov rsi, QWORD (current_rsp - STACK_BACKUP_SIZE) as i64
        ; mov rcx, STACK_BACKUP_SIZE as i32
        ; rep movsb

        // restore original stack
        ; mov rdi, QWORD (current_rsp - STACK_BACKUP_SIZE) as i64
        ; lea rsi, [>old_stack]
        ; mov rcx, STACK_BACKUP_SIZE as _
        ; rep movsb

        ; lea rsp, [>new_stack_base - (STACK_BACKUP_SIZE as isize)]

        // call _dl_open
        ; lea rdi, [>lib_path]
        ; mov rsi, 2
        ; xor rcx, rcx
        ; mov rax, QWORD dl_open_address as i64
        ; call rax

        ; fxrstor [>moar_regs]
        ; pop r15
        ; pop r14
        ; pop r13
        ; pop r12
        ; pop r11
        ; pop r10
        ; pop r9
        ; pop r8
        ; pop rdi
        ; pop rsi
        ; pop rbp
        ; pop rdx
        ; pop rcx
        ; pop rdx
        ; pop rax
        ; popf
        ; mov rsp, QWORD current_rsp as i64
        ; jmp QWORD [>old_rip]

        ; old_rip:
        ; .qword current_rip as i64

        ; old_code:
        ; .bytes code_backup.as_slice()

        ; old_stack:
        ; .bytes &stack_backup
        ; .align 16

        ; moar_regs:
        ; .bytes &[0_u8; 512]
        //; .space 512

        ; lib_path:
        ; .bytes library_path.as_bytes()

        ; proc_self_mem:
        ; .bytes "/proc/self/mem\0".as_bytes()

        ; new_stack:
        ; .align 0x8000

        ; new_stack_base:
    );

    let mut injection_buf = ops.finalize()?;

    // let shared_fd = mman::shm_open(
    //     "/stage_two",
    //     OFlag::O_CREAT | OFlag::O_RDWR,
    //     Mode::S_IRWXG | Mode::S_IRWXU | Mode::S_IRWXO,
    // )
    // .unwrap();
    // unistd::ftruncate(shared_fd, injection_buf.len() as i64).unwrap();
    // let shared_data = unsafe {
    //     mman::mmap(
    //         0 as *mut std::ffi::c_void,
    //         injection_buf.len(),
    //         ProtFlags::PROT_WRITE,
    //         MapFlags::MAP_SHARED,
    //         shared_fd,
    //         0,
    //     )
    //     .unwrap()
    // };
    // unsafe {
    //     ptr::copy_nonoverlapping(
    //         injection_buf.as_ptr(),
    //         shared_data as *mut u8,
    //         injection_buf.len(),
    //     );
    //     mman::munmap(shared_data, injection_buf.len()).unwrap();
    //     unistd::close(shared_fd).unwrap();
    // }

    let mut stage_two = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("/tmp/stage_two.bin")?;

    let mut perms = stage_two.metadata()?.permissions();
    perms.set_readonly(false);
    stage_two.set_permissions(perms)?;
    stage_two.write_all(&mut injection_buf)?;

    manipulator.write(current_rip, shell_code_buf.as_slice())?;

    match signal::kill(Pid::from_raw(pid), Signal::SIGCONT) {
        Ok(_) => (),
        Err(e) => return Err(InjectionError::InternalError(e.to_string())),
    }
    Ok(())
}
