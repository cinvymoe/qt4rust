pub mod analog_simulator;
pub mod combined;
pub mod digital;
pub mod digital_simulator;
pub mod factory;
pub mod registry;

pub use analog_simulator::SimulatedAnalogSource;
pub use combined::CombinedSensorSource;
pub use digital::{
    DigitalInputConfig, DigitalInputSourceFactory, GpioConfig, ModbusDigitalConfig, SpiConfig,
};
pub use digital_simulator::{SimulatedDigitalInput, SimulatedDigitalInputFactory};
pub use factory::{init_builtin_sources, SensorSourceFactory};
pub use registry::DigitalInputRegistry;
