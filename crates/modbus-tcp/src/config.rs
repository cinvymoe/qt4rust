// Modbus TCP Configuration

/// Modbus data type
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ModbusDataType {
    UInt16,
    Int16,
    #[default]
    Float32,
}

/// Modbus byte order
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ModbusByteOrder {
    #[default]
    BigEndian,
    LittleEndian,
}

/// Modbus TCP sensor configuration
#[derive(Debug, Clone)]
pub struct ModbusTcpConfig {
    /// Modbus TCP server address
    pub host: String,
    /// Modbus TCP port (default 502)
    pub port: u16,
    /// Slave ID (1-247)
    pub slave_id: u8,
    /// Register address (starting address)
    pub register_address: u16,
    /// Register count (1 or 2, depending on data type)
    pub register_count: u8,
    /// Data type: UInt16, Int16, Float32
    pub data_type: ModbusDataType,
    /// Timeout in milliseconds
    pub timeout_ms: u64,
    /// Byte order for float data
    pub byte_order: ModbusByteOrder,
}

impl Default for ModbusTcpConfig {
    fn default() -> Self {
        Self {
            host: "192.168.1.100".to_string(),
            port: 502,
            slave_id: 1,
            register_address: 0,
            register_count: 2,
            data_type: ModbusDataType::Float32,
            timeout_ms: 1000,
            byte_order: ModbusByteOrder::BigEndian,
        }
    }
}
