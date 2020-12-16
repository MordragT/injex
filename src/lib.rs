pub mod error;
pub mod external;
pub mod injection;
pub mod internal;
pub mod memory;

pub mod prelude {
    pub use crate::external::AnonManipulator;
    pub use crate::external::ExternalManipulator;
    pub use crate::injection::Injection;
    pub use crate::memory::MemoryManipulation;
}
