// Unified Sensor Library
//
// 提供统一的传感器接口，支持模拟传感器和真实传感器

pub mod config;
pub mod error;
pub mod traits;
pub mod simulated;
pub mod real;

// 保留旧的 simulator 模块以保持向后兼容
pub mod simulator;

pub mod prelude {
    pub use crate::config::*;
    pub use crate::error::*;
    pub use crate::traits::*;
    pub use crate::simulated::*;
    pub use crate::real::*;
    
    // 向后兼容
    pub use crate::simulator::SineSimulator;
}
