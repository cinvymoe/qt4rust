use byteorder::BigEndian;
use byteorder::ByteOrder;
use std::net::SocketAddr;
use std::sync::Mutex;

use tokio_modbus::client::Context;
use tokio_modbus::prelude::*;

use crate::error::{SensorError, SensorResult};
use crate::real::{ByteOrder as SensorByteOrder, ModbusDataType, ModbusTcpConfig};
use crate::traits::SensorProvider;

pub struct ModbusTcpSensor {
    config: ModbusTcpConfig,
    connected: bool,
    client: Mutex<Option<Context>>,
    runtime: Mutex<Option<tokio::runtime::Runtime>>,
    name: String,
}

impl ModbusTcpSensor {
    pub fn new(config: ModbusTcpConfig, name: impl Into<String>) -> Self {
        Self {
            config,
            connected: false,
            client: Mutex::new(None),
            runtime: Mutex::new(None),
            name: name.into(),
        }
    }

    pub fn connect(&mut self) -> SensorResult<()> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| SensorError::InitError(format!("Failed to create runtime: {}", e)))?;

        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port)
            .parse()
            .map_err(|e| SensorError::ConfigError(format!("Invalid address: {}", e)))?;

        use tokio_modbus::slave::Slave;
        let slave = Slave::from(self.config.slave_id);
        let client = runtime
            .block_on(tcp::connect_slave(addr, slave))
            .map_err(|e| {
                SensorError::IoError(std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    e.to_string(),
                ))
            })?;

        *self.client.lock().unwrap() = Some(client);
        *self.runtime.lock().unwrap() = Some(runtime);
        self.connected = true;
        Ok(())
    }

    fn read_holding_registers_sync(&self) -> SensorResult<f64> {
        let registers = {
            let mut client_guard = self.client.lock().unwrap();
            let client = client_guard.as_mut().ok_or(SensorError::NotConnected)?;
            let runtime_guard = self.runtime.lock().unwrap();
            let runtime = runtime_guard.as_ref().ok_or(SensorError::NotConnected)?;

            runtime
                .block_on(client.read_holding_registers(
                    self.config.register_address,
                    self.config.register_count.into(),
                ))
                .map_err(|e| SensorError::ReadError(format!("Modbus read error: {}", e)))
                .and_then(|result: Result<Vec<u16>, tokio_modbus::ExceptionCode>| {
                    result.map_err(|e| {
                        SensorError::ReadError(format!("Modbus protocol error: {}", e))
                    })
                })?
        };

        self.convert_registers_to_f64(&registers)
    }

    fn convert_registers_to_f64(&self, registers: &[u16]) -> SensorResult<f64> {
        match self.config.data_type {
            ModbusDataType::UInt16 => {
                let value = registers
                    .first()
                    .ok_or_else(|| SensorError::ReadError("No register data".to_string()))?;
                Ok(*value as f64)
            }
            ModbusDataType::Int16 => {
                let value = registers
                    .first()
                    .ok_or_else(|| SensorError::ReadError("No register data".to_string()))?;
                Ok(*value as i16 as f64)
            }
            ModbusDataType::Float32 => {
                if registers.len() < 2 {
                    return Err(SensorError::ReadError(
                        "Float32 requires 2 registers".to_string(),
                    ));
                }
                let bytes = match self.config.byte_order {
                    SensorByteOrder::BigEndian => {
                        let mut bytes = [0u8; 4];
                        BigEndian::write_u16(&mut bytes[0..2], registers[0]);
                        BigEndian::write_u16(&mut bytes[2..4], registers[1]);
                        bytes
                    }
                    SensorByteOrder::LittleEndian => {
                        let mut bytes = [0u8; 4];
                        BigEndian::write_u16(&mut bytes[0..2], registers[1]);
                        BigEndian::write_u16(&mut bytes[2..4], registers[0]);
                        bytes
                    }
                };
                let value = BigEndian::read_f32(&bytes);
                Ok(value as f64)
            }
        }
    }

    pub fn disconnect(&mut self) {
        let client = self.client.lock().unwrap().take();
        let runtime = self.runtime.lock().unwrap().take();
        if let (Some(mut client), Some(runtime)) = (client, runtime.as_ref()) {
            let _ = runtime.block_on(client.disconnect());
        }
        self.connected = false;
    }
}

impl SensorProvider for ModbusTcpSensor {
    fn read(&self) -> SensorResult<f64> {
        if !self.connected {
            return Err(SensorError::NotConnected);
        }
        self.read_holding_registers_sync()
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl Drop for ModbusTcpSensor {
    fn drop(&mut self) {
        self.disconnect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modbus_tcp_config_default() {
        let config = ModbusTcpConfig::default();
        assert_eq!(config.host, "192.168.1.100");
        assert_eq!(config.port, 502);
        assert_eq!(config.slave_id, 1);
        assert_eq!(config.register_address, 0);
        assert_eq!(config.register_count, 2);
        assert_eq!(config.data_type, ModbusDataType::Float32);
        assert_eq!(config.timeout_ms, 1000);
        assert_eq!(config.byte_order, SensorByteOrder::BigEndian);
    }

    #[test]
    fn test_modbus_tcp_sensor_creation() {
        let config = ModbusTcpConfig::default();
        let sensor = ModbusTcpSensor::new(config, "Test Sensor");
        assert_eq!(sensor.name(), "Test Sensor");
        assert!(!sensor.is_connected());
    }

    #[test]
    fn test_modbus_tcp_not_connected() {
        let config = ModbusTcpConfig::default();
        let sensor = ModbusTcpSensor::new(config, "Test Sensor");
        let result = sensor.read();
        assert!(result.is_err());
        match result {
            Err(SensorError::NotConnected) => {}
            _ => panic!("Expected NotConnected error"),
        }
    }

    #[test]
    fn test_uint16_conversion() {
        let config = ModbusTcpConfig {
            data_type: ModbusDataType::UInt16,
            ..Default::default()
        };
        let sensor = ModbusTcpSensor::new(config, "Test");
        let registers = [1000u16];
        let result = sensor.convert_registers_to_f64(&registers);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1000.0);
    }

    #[test]
    fn test_int16_conversion() {
        let config = ModbusTcpConfig {
            data_type: ModbusDataType::Int16,
            ..Default::default()
        };
        let sensor = ModbusTcpSensor::new(config, "Test");
        let registers = [0xFFF6u16];
        let result = sensor.convert_registers_to_f64(&registers);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), -10.0);
    }

    #[test]
    fn test_float32_big_endian_conversion() {
        let config = ModbusTcpConfig {
            data_type: ModbusDataType::Float32,
            byte_order: SensorByteOrder::BigEndian,
            ..Default::default()
        };
        let sensor = ModbusTcpSensor::new(config, "Test");
        let registers = [0x4148, 0x0000];
        let result = sensor.convert_registers_to_f64(&registers);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!((value - 12.5).abs() < 0.001);
    }

    #[test]
    fn test_float32_little_endian_conversion() {
        let config = ModbusTcpConfig {
            data_type: ModbusDataType::Float32,
            byte_order: SensorByteOrder::LittleEndian,
            ..Default::default()
        };
        let sensor = ModbusTcpSensor::new(config, "Test");
        let registers = [0x0000, 0x4148];
        let result = sensor.convert_registers_to_f64(&registers);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!((value - 12.5).abs() < 0.001);
    }

    #[test]
    fn test_float32_requires_two_registers() {
        let config = ModbusTcpConfig {
            data_type: ModbusDataType::Float32,
            ..Default::default()
        };
        let sensor = ModbusTcpSensor::new(config, "Test");
        let registers = [1000u16];
        let result = sensor.convert_registers_to_f64(&registers);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_registers_error() {
        let config = ModbusTcpConfig::default();
        let sensor = ModbusTcpSensor::new(config, "Test");
        let result = sensor.convert_registers_to_f64(&[]);
        assert!(result.is_err());
    }
}
