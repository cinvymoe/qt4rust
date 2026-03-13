// Qt Threading Utils Library

pub mod timer;
pub mod collector;

pub mod prelude {
    pub use crate::timer::*;
    pub use crate::collector::*;
}
