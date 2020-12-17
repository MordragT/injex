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
    pub use crate::memory::MemoryManipulation;
    pub use crate::process::Process;
}
