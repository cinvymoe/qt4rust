use crate::client::{ModbusProvider, ModbusTcpClient};
use crate::config::{ModbusByteOrder, ModbusDataType, ModbusTcpConfig};
use crate::error::{ModbusError, ModbusResult};
use sensor_traits::{SensorError, SensorReading, SensorResult, SensorSource};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 传感器类型
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SensorKind {
    Analog,
    Digital,
}

/// 单个传感器的 Modbus 配置
#[derive(Debug, Clone)]
pub struct SensorModbusConfig {
    pub name: String,
    pub kind: SensorKind,
    pub slave_id: u8,
    pub register_address: u16,
    pub register_count: u8,
    pub data_type: ModbusDataType,
    pub byte_order: ModbusByteOrder,
}

/// 动态 Modbus 数据源
pub struct ModbusDataSource {
    /// 服务器配置
    host: String,
    port: u16,
    timeout_ms: u64,
    /// 模拟传感器客户端 (key = 传感器ID)
    analog_clients: HashMap<String, ModbusTcpClient>,
    /// 数字输入客户端 (key = 传感器ID)
    digital_clients: HashMap<String, ModbusTcpClient>,
}

impl ModbusDataSource {
    /// 创建新的数据源
    pub fn new(host: String, port: u16, timeout_ms: u64) -> Self {
        Self {
            host,
            port,
            timeout_ms,
            analog_clients: HashMap::new(),
            digital_clients: HashMap::new(),
        }
    }

    /// 添加传感器
    pub fn add_sensor(&mut self, sensor_id: &str, config: SensorModbusConfig) {
        let modbus_config = ModbusTcpConfig {
            host: self.host.clone(),
            port: self.port,
            slave_id: config.slave_id,
            register_address: config.register_address,
            register_count: config.register_count,
            data_type: config.data_type,
            timeout_ms: self.timeout_ms,
            byte_order: config.byte_order,
        };

        let client = ModbusTcpClient::new(modbus_config, &config.name);

        match config.kind {
            SensorKind::Analog => {
                self.analog_clients.insert(sensor_id.to_string(), client);
            }
            SensorKind::Digital => {
                self.digital_clients.insert(sensor_id.to_string(), client);
            }
        }
    }

    /// 从配置列表创建数据源
    pub fn from_config_list(
        host: String,
        port: u16,
        timeout_ms: u64,
        sensors: Vec<(String, SensorModbusConfig)>,
    ) -> Self {
        let mut source = Self::new(host, port, timeout_ms);
        for (id, config) in sensors {
            source.add_sensor(&id, config);
        }
        source
    }

    /// 连接所有客户端
    pub fn connect(&mut self) -> ModbusResult<()> {
        for (id, client) in &mut self.analog_clients {
            client
                .connect()
                .map_err(|e| ModbusError::InitError(format!("{} 连接失败: {:?}", id, e)))?;
        }
        for (id, client) in &mut self.digital_clients {
            client
                .connect()
                .map_err(|e| ModbusError::InitError(format!("{} 连接失败: {:?}", id, e)))?;
        }
        Ok(())
    }

    /// 读取所有传感器数据
    pub fn read_sensors(&self) -> ModbusResult<SensorReading> {
        let mut analog = HashMap::new();
        let mut digital = HashMap::new();

        for (id, client) in &self.analog_clients {
            match client.read() {
                Ok(value) => {
                    analog.insert(id.clone(), value);
                }
                Err(e) => {
                    tracing::warn!("传感器 {} 读取失败: {:?}", id, e);
                }
            }
        }

        for (id, client) in &self.digital_clients {
            match client.read() {
                Ok(value) => {
                    digital.insert(id.clone(), value > 0.5);
                }
                Err(e) => {
                    tracing::warn!("数字输入 {} 读取失败: {:?}", id, e);
                }
            }
        }

        Ok(SensorReading { analog, digital })
    }

    /// 检查连接状态
    pub fn is_connected(&self) -> bool {
        self.analog_clients.values().all(|c| c.is_connected())
            && self.digital_clients.values().all(|c| c.is_connected())
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
    fn read_all(&self) -> SensorResult<SensorReading> {
        self.read_sensors().map_err(Self::convert_error)
    }

    fn is_connected(&self) -> bool {
        ModbusDataSource::is_connected(self)
    }
}

// ===== 兼容旧 API =====

/// 旧版 ModbusDataSource（保持向后兼容）
pub struct LegacyModbusDataSource {
    ad1: ModbusTcpClient,
    ad2: ModbusTcpClient,
    ad3: ModbusTcpClient,
}

impl LegacyModbusDataSource {
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

    pub fn is_connected(&self) -> bool {
        self.ad1.is_connected() && self.ad2.is_connected() && self.ad3.is_connected()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_data_source() {
        let mut source = ModbusDataSource::new("127.0.0.1".to_string(), 502, 1000);

        source.add_sensor(
            "main_hook_weight",
            SensorModbusConfig {
                name: "主钩重量".to_string(),
                kind: SensorKind::Analog,
                slave_id: 1,
                register_address: 0,
                register_count: 1,
                data_type: ModbusDataType::UInt16,
                byte_order: ModbusByteOrder::BigEndian,
            },
        );

        source.add_sensor(
            "aux_hook_weight",
            SensorModbusConfig {
                name: "副钩重量".to_string(),
                kind: SensorKind::Analog,
                slave_id: 1,
                register_address: 3,
                register_count: 1,
                data_type: ModbusDataType::UInt16,
                byte_order: ModbusByteOrder::BigEndian,
            },
        );

        assert_eq!(source.analog_clients.len(), 2);
    }
}
