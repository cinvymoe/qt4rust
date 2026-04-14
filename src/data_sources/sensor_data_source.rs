use crate::models::SensorData;
use modbus_tcp::prelude::*;
use sensor_simulator::prelude::*;

pub enum SensorMode {
    Simulated,
    ModbusTcp,
}

pub struct SensorDataSource {
    mode: SensorMode,

    ad1_simulator: Option<SimulatedSensor>,
    ad2_simulator: Option<SimulatedSensor>,
    ad3_simulator: Option<SimulatedSensor>,

    ad1_modbus: Option<ModbusTcpClient>,
    ad2_modbus: Option<ModbusTcpClient>,
    ad3_modbus: Option<ModbusTcpClient>,
}

impl SensorDataSource {
    pub fn new() -> Self {
        let ad1_simulator = SimulatedSensor::new(SimulatorType::Random {
            min: 0.0,
            max: 4095.0,
        });
        let ad2_simulator = SimulatedSensor::new(SimulatorType::Random {
            min: 0.0,
            max: 4095.0,
        });
        let ad3_simulator = SimulatedSensor::new(SimulatorType::Random {
            min: 0.0,
            max: 4095.0,
        });

        Self {
            mode: SensorMode::Simulated,
            ad1_simulator: Some(ad1_simulator),
            ad2_simulator: Some(ad2_simulator),
            ad3_simulator: Some(ad3_simulator),
            ad1_modbus: None,
            ad2_modbus: None,
            ad3_modbus: None,
        }
    }

    pub fn from_config() -> Result<Self, String> {
        use crate::config::pipeline_config::PipelineConfig;

        let pipeline_config = PipelineConfig::load();

        if pipeline_config.collection.use_simulator {
            tracing::info!("使用模拟传感器（根据 pipeline_config.toml）");
            return Ok(Self::new());
        }

        tracing::info!("尝试加载 Modbus TCP 配置...");
        match Self::load_modbus_config() {
            Ok(source) => {
                tracing::info!("成功加载 Modbus TCP 配置");
                Ok(source)
            }
            Err(e) => {
                tracing::warn!("加载 Modbus TCP 配置失败: {}", e);
                tracing::warn!("回退到模拟传感器");
                Ok(Self::new())
            }
        }
    }

    fn load_modbus_config() -> Result<Self, String> {
        use std::fs;
        use toml::Value;

        let config_path = "config/modbus_sensors.toml";
        let content = fs::read_to_string(config_path)
            .map_err(|e| format!("无法读取配置文件 {}: {}", config_path, e))?;

        let config: Value =
            toml::from_str(&content).map_err(|e| format!("无法解析配置文件: {}", e))?;

        let server = config.get("server").ok_or("配置文件缺少 [server] 部分")?;

        let host = server
            .get("host")
            .and_then(|v| v.as_str())
            .ok_or("缺少 server.host 配置")?;

        let port = server
            .get("port")
            .and_then(|v| v.as_integer())
            .ok_or("缺少 server.port 配置")? as u16;

        let timeout_ms = server
            .get("timeout_ms")
            .and_then(|v| v.as_integer())
            .unwrap_or(1000) as u64;

        let ad1 = config
            .get("ad1_load")
            .ok_or("配置文件缺少 [ad1_load] 部分")?;
        let ad2 = config
            .get("ad2_radius")
            .ok_or("配置文件缺少 [ad2_radius] 部分")?;
        let ad3 = config
            .get("ad3_angle")
            .ok_or("配置文件缺少 [ad3_angle] 部分")?;

        let slave_id = ad1
            .get("slave_id")
            .and_then(|v| v.as_integer())
            .ok_or("缺少 ad1_load.slave_id 配置")? as u8;

        let ad1_register = ad1
            .get("register_address")
            .and_then(|v| v.as_integer())
            .ok_or("缺少 ad1_load.register_address 配置")? as u16;

        let ad2_register = ad2
            .get("register_address")
            .and_then(|v| v.as_integer())
            .ok_or("缺少 ad2_radius.register_address 配置")? as u16;

        let ad3_register = ad3
            .get("register_address")
            .and_then(|v| v.as_integer())
            .ok_or("缺少 ad3_angle.register_address 配置")? as u16;

        let data_type_str = ad1
            .get("data_type")
            .and_then(|v| v.as_str())
            .unwrap_or("UInt16");

        let data_type = match data_type_str {
            "UInt16" => ModbusDataType::UInt16,
            "Float32" => ModbusDataType::Float32,
            _ => return Err(format!("不支持的数据类型: {}", data_type_str)),
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
            _ => return Err(format!("不支持的字节序: {}", byte_order_str)),
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

        tracing::info!("Modbus TCP 配置: {}:{}, slave_id={}", host, port, slave_id);
        tracing::info!(
            "AD1 寄存器: {}, AD2 寄存器: {}, AD3 寄存器: {}",
            ad1_register,
            ad2_register,
            ad3_register
        );

        let mut source = Self {
            mode: SensorMode::ModbusTcp,
            ad1_simulator: None,
            ad2_simulator: None,
            ad3_simulator: None,
            ad1_modbus: Some(ModbusTcpClient::new(ad1_config, "AD1 Load Cell")),
            ad2_modbus: Some(ModbusTcpClient::new(ad2_config, "AD2 Radius")),
            ad3_modbus: Some(ModbusTcpClient::new(ad3_config, "AD3 Angle")),
        };

        source.connect()?;

        Ok(source)
    }

    pub fn with_modbus_tcp(
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
            mode: SensorMode::ModbusTcp,
            ad1_simulator: None,
            ad2_simulator: None,
            ad3_simulator: None,
            ad1_modbus: Some(ModbusTcpClient::new(ad1_config, "AD1 Load Cell")),
            ad2_modbus: Some(ModbusTcpClient::new(ad2_config, "AD2 Radius")),
            ad3_modbus: Some(ModbusTcpClient::new(ad3_config, "AD3 Angle")),
        }
    }

    pub fn connect(&mut self) -> Result<(), String> {
        match self.mode {
            SensorMode::Simulated => Ok(()),
            SensorMode::ModbusTcp => {
                if let Some(ref mut sensor) = self.ad1_modbus {
                    sensor
                        .connect()
                        .map_err(|e| format!("AD1 Modbus 连接失败: {:?}", e))?;
                }
                if let Some(ref mut sensor) = self.ad2_modbus {
                    sensor
                        .connect()
                        .map_err(|e| format!("AD2 Modbus 连接失败: {:?}", e))?;
                }
                if let Some(ref mut sensor) = self.ad3_modbus {
                    sensor
                        .connect()
                        .map_err(|e| format!("AD3 Modbus 连接失败: {:?}", e))?;
                }
                Ok(())
            }
        }
    }

    pub fn read_data(&self) -> Result<SensorData, String> {
        match self.mode {
            SensorMode::Simulated => {
                let ad1_load = self
                    .ad1_simulator
                    .as_ref()
                    .unwrap()
                    .read()
                    .map_err(|e| format!("AD1 读取失败: {:?}", e))?;
                let ad2_radius = self
                    .ad2_simulator
                    .as_ref()
                    .unwrap()
                    .read()
                    .map_err(|e| format!("AD2 读取失败: {:?}", e))?;
                let ad3_angle = self
                    .ad3_simulator
                    .as_ref()
                    .unwrap()
                    .read()
                    .map_err(|e| format!("AD3 读取失败: {:?}", e))?;
                Ok(SensorData::new(ad1_load, ad2_radius, ad3_angle))
            }
            SensorMode::ModbusTcp => {
                let ad1_load = self
                    .ad1_modbus
                    .as_ref()
                    .unwrap()
                    .read()
                    .map_err(|e| format!("AD1 读取失败: {:?}", e))?;
                let ad2_radius = self
                    .ad2_modbus
                    .as_ref()
                    .unwrap()
                    .read()
                    .map_err(|e| format!("AD2 读取失败: {:?}", e))?;
                let ad3_angle = self
                    .ad3_modbus
                    .as_ref()
                    .unwrap()
                    .read()
                    .map_err(|e| format!("AD3 读取失败: {:?}", e))?;
                Ok(SensorData::new(ad1_load, ad2_radius, ad3_angle))
            }
        }
    }
}

impl Default for SensorDataSource {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_data() {
        let source = SensorDataSource::new();
        let result = source.read_data();

        assert!(result.is_ok());

        let data = result.unwrap();
        assert!(data.ad1_load >= 0.0 && data.ad1_load <= 4095.0);
        assert!(data.ad2_radius >= 0.0 && data.ad2_radius <= 4095.0);
        assert!(data.ad3_angle >= 0.0 && data.ad3_angle <= 4095.0);
    }

    #[test]
    fn test_multiple_reads() {
        let source = SensorDataSource::new();

        let data1 = source.read_data().unwrap();
        let data2 = source.read_data().unwrap();

        assert!(
            data1.ad1_load != data2.ad1_load
                || data1.ad2_radius != data2.ad2_radius
                || data1.ad3_angle != data2.ad3_angle
        );
    }

    #[test]
    fn test_from_config() {
        let result = SensorDataSource::from_config();

        assert!(result.is_ok());

        let source = result.unwrap();

        let data_result = source.read_data();
        assert!(data_result.is_ok());
    }
}
