use crate::client::{ModbusProvider, ModbusTcpClient};
use crate::config::{ModbusByteOrder, ModbusDataType, ModbusTcpConfig};
use crate::error::{ModbusError, ModbusResult};
use crane_data_layer::prelude::*;

pub struct ModbusDataSource {
    ad1: ModbusTcpClient,
    ad2: ModbusTcpClient,
    ad3: ModbusTcpClient,
}

impl ModbusDataSource {
    pub fn new(
        host: &str,
        port: u16,
        slave_id: u8,
        ad1_register: u16,
        ad2_register: u16,
        ad3_register: u16,
    ) -> Self {
        let ad1_config = ModbusTcpConfig {
            host: host.to_string(),
            port,
            slave_id,
            register_address: ad1_register,
            register_count: 2,
            data_type: ModbusDataType::Float32,
            timeout_ms: 1000,
            byte_order: ModbusByteOrder::BigEndian,
        };
        let ad2_config = ModbusTcpConfig {
            host: host.to_string(),
            port,
            slave_id,
            register_address: ad2_register,
            register_count: 2,
            data_type: ModbusDataType::Float32,
            timeout_ms: 1000,
            byte_order: ModbusByteOrder::BigEndian,
        };
        let ad3_config = ModbusTcpConfig {
            host: host.to_string(),
            port,
            slave_id,
            register_address: ad3_register,
            register_count: 2,
            data_type: ModbusDataType::Float32,
            timeout_ms: 1000,
            byte_order: ModbusByteOrder::BigEndian,
        };

        Self {
            ad1: ModbusTcpClient::new(ad1_config, "AD1 Load Cell"),
            ad2: ModbusTcpClient::new(ad2_config, "AD2 Radius"),
            ad3: ModbusTcpClient::new(ad3_config, "AD3 Angle"),
        }
    }

    pub fn from_config(config: &str) -> ModbusResult<Self> {
        let config: toml::Value = toml::from_str(config)
            .map_err(|e| ModbusError::ConfigError(format!("Failed to parse config: {}", e)))?;

        let server = config
            .get("server")
            .ok_or_else(|| ModbusError::ConfigError("Missing [server] section".to_string()))?;

        let host = server
            .get("host")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ModbusError::ConfigError("Missing server.host".to_string()))?;

        let port = server
            .get("port")
            .and_then(|v| v.as_integer())
            .map(|v| v as u16)
            .ok_or_else(|| ModbusError::ConfigError("Missing server.port".to_string()))?;

        let timeout_ms = server
            .get("timeout_ms")
            .and_then(|v| v.as_integer())
            .map(|v| v as u64)
            .unwrap_or(1000);

        let ad1 = config
            .get("ad1_load")
            .ok_or_else(|| ModbusError::ConfigError("Missing [ad1_load] section".to_string()))?;
        let ad2 = config
            .get("ad2_radius")
            .ok_or_else(|| ModbusError::ConfigError("Missing [ad2_radius] section".to_string()))?;
        let ad3 = config
            .get("ad3_angle")
            .ok_or_else(|| ModbusError::ConfigError("Missing [ad3_angle] section".to_string()))?;

        let slave_id = ad1
            .get("slave_id")
            .and_then(|v| v.as_integer())
            .map(|v| v as u8)
            .ok_or_else(|| ModbusError::ConfigError("Missing ad1_load.slave_id".to_string()))?;

        let ad1_register = ad1
            .get("register_address")
            .and_then(|v| v.as_integer())
            .map(|v| v as u16)
            .ok_or_else(|| {
                ModbusError::ConfigError("Missing ad1_load.register_address".to_string())
            })?;

        let ad2_register = ad2
            .get("register_address")
            .and_then(|v| v.as_integer())
            .map(|v| v as u16)
            .ok_or_else(|| {
                ModbusError::ConfigError("Missing ad2_radius.register_address".to_string())
            })?;

        let ad3_register = ad3
            .get("register_address")
            .and_then(|v| v.as_integer())
            .map(|v| v as u16)
            .ok_or_else(|| {
                ModbusError::ConfigError("Missing ad3_angle.register_address".to_string())
            })?;

        let data_type_str = ad1
            .get("data_type")
            .and_then(|v| v.as_str())
            .unwrap_or("Float32");

        let data_type = match data_type_str {
            "UInt16" => ModbusDataType::UInt16,
            "Float32" => ModbusDataType::Float32,
            _ => {
                return Err(ModbusError::ConfigError(format!(
                    "Unsupported data type: {}",
                    data_type_str
                )))
            }
        };

        let register_count = match data_type {
            ModbusDataType::UInt16 => 1,
            ModbusDataType::Float32 => 2,
            _ => 1,
        };

        let byte_order_str = ad1
            .get("byte_order")
            .and_then(|v| v.as_str())
            .unwrap_or("BigEndian");

        let byte_order = match byte_order_str {
            "BigEndian" => ModbusByteOrder::BigEndian,
            "LittleEndian" => ModbusByteOrder::LittleEndian,
            _ => {
                return Err(ModbusError::ConfigError(format!(
                    "Unsupported byte order: {}",
                    byte_order_str
                )))
            }
        };

        let ad1_config = ModbusTcpConfig {
            host: host.to_string(),
            port,
            slave_id,
            register_address: ad1_register,
            register_count,
            data_type: data_type.clone(),
            timeout_ms,
            byte_order: byte_order.clone(),
        };

        let ad2_config = ModbusTcpConfig {
            host: host.to_string(),
            port,
            slave_id,
            register_address: ad2_register,
            register_count,
            data_type: data_type.clone(),
            timeout_ms,
            byte_order: byte_order.clone(),
        };

        let ad3_config = ModbusTcpConfig {
            host: host.to_string(),
            port,
            slave_id,
            register_address: ad3_register,
            register_count,
            data_type,
            timeout_ms,
            byte_order,
        };

        Ok(Self {
            ad1: ModbusTcpClient::new(ad1_config, "AD1 Load Cell"),
            ad2: ModbusTcpClient::new(ad2_config, "AD2 Radius"),
            ad3: ModbusTcpClient::new(ad3_config, "AD3 Angle"),
        })
    }

    pub fn connect(&mut self) -> ModbusResult<()> {
        self.ad1.connect().map_err(|e| {
            ModbusError::InitError(format!("AD1 Modbus connection failed: {:?}", e))
        })?;
        self.ad2.connect().map_err(|e| {
            ModbusError::InitError(format!("AD2 Modbus connection failed: {:?}", e))
        })?;
        self.ad3.connect().map_err(|e| {
            ModbusError::InitError(format!("AD3 Modbus connection failed: {:?}", e))
        })?;
        Ok(())
    }

    pub fn read_all(&self) -> ModbusResult<(f64, f64, f64)> {
        let ad1 = self.ad1.read()?;
        let ad2 = self.ad2.read()?;
        let ad3 = self.ad3.read()?;
        Ok((ad1, ad2, ad3))
    }

    pub fn ad1(&self) -> &ModbusTcpClient {
        &self.ad1
    }

    pub fn ad2(&self) -> &ModbusTcpClient {
        &self.ad2
    }

    pub fn ad3(&self) -> &ModbusTcpClient {
        &self.ad3
    }

    pub fn is_connected(&self) -> bool {
        self.ad1.is_connected() && self.ad2.is_connected() && self.ad3.is_connected()
    }

    fn convert_error(e: ModbusError) -> SensorError {
        match e {
            ModbusError::ReadError(msg) => SensorError::ReadError(msg),
            ModbusError::InitError(msg) => SensorError::InitError(msg),
            ModbusError::Timeout => SensorError::Timeout,
            ModbusError::ConfigError(msg) => SensorError::ConfigError(msg),
            ModbusError::NotConnected => SensorError::NotConnected,
            ModbusError::IoError(io) => SensorError::IoError(io.to_string()),
            ModbusError::ProtocolError(msg) => SensorError::ReadError(msg),
        }
    }
}

impl SensorSource for ModbusDataSource {
    fn read_all(&self) -> SensorResult<(f64, f64, f64)> {
        self.read_all().map_err(Self::convert_error)
    }

    fn is_connected(&self) -> bool {
        ModbusDataSource::is_connected(self)
    }
}
