use byteorder::BigEndian;
use byteorder::ByteOrder;
use std::net::SocketAddr;
use std::sync::Mutex;

use tokio_modbus::client::Context;
use tokio_modbus::prelude::*;

use crate::config::{ModbusByteOrder, ModbusDataType, ModbusTcpConfig};
use crate::error::{ModbusError, ModbusResult};

pub trait ModbusProvider {
    fn read(&self) -> ModbusResult<f64>;
    fn is_connected(&self) -> bool;
}

pub struct ModbusTcpClient {
    config: ModbusTcpConfig,
    connected: bool,
    client: Mutex<Option<Context>>,
    runtime: Mutex<Option<tokio::runtime::Runtime>>,
    #[allow(dead_code)]
    name: String,
}

impl ModbusTcpClient {
    pub fn new(config: ModbusTcpConfig, name: impl Into<String>) -> Self {
        Self {
            config,
            connected: false,
            client: Mutex::new(None),
            runtime: Mutex::new(None),
            name: name.into(),
        }
    }

    pub fn connect(&mut self) -> ModbusResult<()> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| ModbusError::InitError(format!("Failed to create runtime: {}", e)))?;

        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port)
            .parse()
            .map_err(|e| ModbusError::ConfigError(format!("Invalid address: {}", e)))?;

        use tokio_modbus::slave::Slave;
        let slave = Slave::from(self.config.slave_id);
        let client = runtime
            .block_on(tcp::connect_slave(addr, slave))
            .map_err(|e| {
                ModbusError::IoError(std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    e.to_string(),
                ))
            })?;

        *self.client.lock().unwrap() = Some(client);
        *self.runtime.lock().unwrap() = Some(runtime);
        self.connected = true;
        Ok(())
    }

    fn read_holding_registers_sync(&self) -> ModbusResult<f64> {
        let registers = {
            let mut client_guard = self.client.lock().unwrap();
            let client = client_guard.as_mut().ok_or(ModbusError::NotConnected)?;
            let runtime_guard = self.runtime.lock().unwrap();
            let runtime = runtime_guard.as_ref().ok_or(ModbusError::NotConnected)?;

            runtime
                .block_on(client.read_holding_registers(
                    self.config.register_address,
                    self.config.register_count.into(),
                ))
                .map_err(|e| ModbusError::ReadError(format!("Modbus read error: {}", e)))
                .and_then(|result: Result<Vec<u16>, tokio_modbus::ExceptionCode>| {
                    result.map_err(|e| ModbusError::ProtocolError(e.to_string()))
                })?
        };

        self.convert_registers_to_f64(&registers)
    }

    fn convert_registers_to_f64(&self, registers: &[u16]) -> ModbusResult<f64> {
        match self.config.data_type {
            ModbusDataType::UInt16 => {
                let value = registers
                    .first()
                    .ok_or_else(|| ModbusError::ReadError("No register data".to_string()))?;
                Ok(*value as f64)
            }
            ModbusDataType::Int16 => {
                let value = registers
                    .first()
                    .ok_or_else(|| ModbusError::ReadError("No register data".to_string()))?;
                Ok(*value as i16 as f64)
            }
            ModbusDataType::Float32 => {
                if registers.len() < 2 {
                    return Err(ModbusError::ReadError(
                        "Float32 requires 2 registers".to_string(),
                    ));
                }
                let bytes = match self.config.byte_order {
                    ModbusByteOrder::BigEndian => {
                        let mut bytes = [0u8; 4];
                        BigEndian::write_u16(&mut bytes[0..2], registers[0]);
                        BigEndian::write_u16(&mut bytes[2..4], registers[1]);
                        bytes
                    }
                    ModbusByteOrder::LittleEndian => {
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

impl ModbusProvider for ModbusTcpClient {
    fn read(&self) -> ModbusResult<f64> {
        if !self.connected {
            return Err(ModbusError::NotConnected);
        }
        self.read_holding_registers_sync()
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

impl Drop for ModbusTcpClient {
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
        assert_eq!(config.byte_order, ModbusByteOrder::BigEndian);
    }

    #[test]
    fn test_modbus_tcp_client_creation() {
        let config = ModbusTcpConfig::default();
        let client = ModbusTcpClient::new(config, "Test Client");
        assert!(!client.is_connected());
    }

    #[test]
    fn test_uint16_conversion() {
        let config = ModbusTcpConfig {
            data_type: ModbusDataType::UInt16,
            ..Default::default()
        };
        let client = ModbusTcpClient::new(config, "Test");
        let registers = [1000u16];
        let result = client.convert_registers_to_f64(&registers);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1000.0);
    }

    #[test]
    fn test_int16_conversion() {
        let config = ModbusTcpConfig {
            data_type: ModbusDataType::Int16,
            ..Default::default()
        };
        let client = ModbusTcpClient::new(config, "Test");
        let registers = [0xFFF6u16];
        let result = client.convert_registers_to_f64(&registers);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), -10.0);
    }

    #[test]
    fn test_float32_big_endian_conversion() {
        let config = ModbusTcpConfig {
            data_type: ModbusDataType::Float32,
            byte_order: ModbusByteOrder::BigEndian,
            ..Default::default()
        };
        let client = ModbusTcpClient::new(config, "Test");
        let registers = [0x4148, 0x0000];
        let result = client.convert_registers_to_f64(&registers);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!((value - 12.5).abs() < 0.001);
    }

    #[test]
    fn test_float32_little_endian_conversion() {
        let config = ModbusTcpConfig {
            data_type: ModbusDataType::Float32,
            byte_order: ModbusByteOrder::LittleEndian,
            ..Default::default()
        };
        let client = ModbusTcpClient::new(config, "Test");
        let registers = [0x0000, 0x4148];
        let result = client.convert_registers_to_f64(&registers);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!((value - 12.5).abs() < 0.001);
    }
}
