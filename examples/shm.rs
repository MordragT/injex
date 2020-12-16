use nix::{
    fcntl::OFlag,
    sys::{
        mman::{self, MapFlags, ProtFlags},
        signal::{self, Signal},
        stat::Mode,
    },
    unistd::{self, Pid},
};

fn main() {
    mman::shm_open("/stage_two", OFlag::O_RDONLY, Mode::S_IROTH).unwrap();
}
