//! This library helps injecting dynamic libraries into processes
//! and manipulating the memory of these processes.
//! Currently only Linux is supported but support for other platforms (freebsd, windows) is coming.
//!
//! ```rust
//! use injex::prelude::*;
//!
//! # use std::error::Error;
//! #
//! # fn main() -> Result<(), Box<dyn Error>> {
//!
//! let anon = AnonManipulator::new("game_name")?;
//! inject(&anon, anon.pid(), "path/to/dynamic_library")?;
//! # Ok(())
//! # }
//! ```
//! ```rust
//! // Dynamic Library in its own crate
//!
//! use std::thread;
//! use injex::prelude::*;
//!
//! #[link_section = ".init_array"]
//! static INITIALIZE: fn() = init;
//!
//! fn init() {
//!    thread::spawn(move || -> thread::Result<()> {
//!        let manipulator = InternalManipulator {}
//!        println!("{:?}", manipulator.memory_maps());
//!        let address = manipulator.find(0, 1024, &[0, 3, 10, 32, 1]).unwrap();
//!        loop {
//!            manipulator.write(address, &[255, 255, 255, 255]).unwrap();
//!        }
//!    });
//! }
//!
//! ```

pub mod error;
pub mod external;
pub mod injection;
pub mod internal;
pub mod memory;
pub mod process;

pub mod prelude {
    pub use crate::external::AnonManipulator;
    pub use crate::external::ExternalManipulator;
    pub use crate::injection::inject;
    pub use crate::internal::InternalManipulator;
    pub use crate::memory::DefaultInit;
    pub use crate::memory::MemoryManipulation;
    pub use crate::process::Process;
}
