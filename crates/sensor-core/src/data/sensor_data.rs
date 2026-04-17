use crate::storage::{ColumnDef, DatabaseSchema};
use rusqlite::types::ToSql;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 传感器数据 - 使用 HashMap 存储所有传感器值
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SensorData {
    /// 模拟传感器值 (AD值或已转换值)
    pub analog: HashMap<String, f64>,
    /// 数字输入值 (开关状态)
    pub digital: HashMap<String, bool>,
}

impl SensorData {
    /// 创建新的传感器数据
    pub fn new(analog: HashMap<String, f64>, digital: HashMap<String, bool>) -> Self {
        Self { analog, digital }
    }

    /// 创建空的传感器数据
    pub fn empty() -> Self {
        Self {
            analog: HashMap::new(),
            digital: HashMap::new(),
        }
    }

    /// 获取模拟传感器值
    pub fn get_analog(&self, sensor_id: &str) -> Option<&f64> {
        self.analog.get(sensor_id)
    }

    /// 获取模拟传感器值，不存在则返回默认值
    pub fn get_analog_or(&self, sensor_id: &str, default: f64) -> f64 {
        self.analog.get(sensor_id).copied().unwrap_or(default)
    }

    /// 获取数字输入值
    pub fn get_digital(&self, sensor_id: &str) -> Option<&bool> {
        self.digital.get(sensor_id)
    }

    /// 获取数字输入值，不存在则返回默认值
    pub fn get_digital_or(&self, sensor_id: &str, default: bool) -> bool {
        self.digital.get(sensor_id).copied().unwrap_or(default)
    }

    /// 设置模拟传感器值
    pub fn set_analog(&mut self, sensor_id: &str, value: f64) {
        self.analog.insert(sensor_id.to_string(), value);
    }

    /// 设置数字输入值
    pub fn set_digital(&mut self, sensor_id: &str, value: bool) {
        self.digital.insert(sensor_id.to_string(), value);
    }

    /// 验证数据有效性
    pub fn validate(&self) -> Result<(), String> {
        for (id, value) in &self.analog {
            if *value < 0.0 {
                return Err(format!("传感器 {} 数据异常：负值 {}", id, value));
            }
        }
        Ok(())
    }

    // ===== 兼容旧 API =====

    /// 兼容旧 API: 获取主钩重量 (ad1_load)
    pub fn ad1_load(&self) -> f64 {
        self.get_analog_or("main_hook_weight", 0.0)
    }

    /// 兼容旧 API: 获取半径 (ad2_radius)
    pub fn ad2_radius(&self) -> f64 {
        self.get_analog_or("radius", 0.0)
    }

    /// 兼容旧 API: 获取角度 (ad3_angle)
    pub fn ad3_angle(&self) -> f64 {
        self.get_analog_or("angle", 0.0)
    }

    /// 兼容旧 API: 获取主钩开关
    pub fn digital_input_0(&self) -> bool {
        self.get_digital_or("main_hook_switch", false)
    }

    /// 兼容旧 API: 获取副钩开关
    pub fn digital_input_1(&self) -> bool {
        self.get_digital_or("aux_hook_switch", false)
    }

    /// 兼容旧 API: 从元组创建
    pub fn from_tuple(ad1: f64, ad2: f64, ad3: f64, di0: bool, di1: bool) -> Self {
        let mut analog = HashMap::new();
        analog.insert("main_hook_weight".to_string(), ad1);
        analog.insert("radius".to_string(), ad2);
        analog.insert("angle".to_string(), ad3);

        let mut digital = HashMap::new();
        digital.insert("main_hook_switch".to_string(), di0);
        digital.insert("aux_hook_switch".to_string(), di1);

        Self { analog, digital }
    }
}

impl DatabaseSchema for SensorData {
    fn table_name() -> &'static str {
        "sensor_data"
    }

    fn columns() -> &'static [ColumnDef] {
        static COLUMNS: std::sync::OnceLock<Vec<ColumnDef>> = std::sync::OnceLock::new();
        let cols = COLUMNS.get_or_init(|| {
            vec![
                ColumnDef {
                    name: "analog_json".to_string(),
                    sql_type: "TEXT".to_string(),
                    nullable: false,
                },
                ColumnDef {
                    name: "digital_json".to_string(),
                    sql_type: "TEXT".to_string(),
                    nullable: false,
                },
            ]
        });
        cols.as_slice()
    }

    fn field_values(&self) -> Vec<Box<dyn ToSql>> {
        let analog_json = serde_json::to_string(&self.analog).unwrap_or_else(|_| "{}".to_string());
        let digital_json =
            serde_json::to_string(&self.digital).unwrap_or_else(|_| "{}".to_string());

        vec![Box::new(analog_json), Box::new(digital_json)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_sensor_data() {
        let mut analog = HashMap::new();
        analog.insert("main_hook_weight".to_string(), 17.0);
        analog.insert("radius".to_string(), 10.0);
        analog.insert("angle".to_string(), 62.7);

        let mut digital = HashMap::new();
        digital.insert("main_hook_switch".to_string(), false);

        let data = SensorData::new(analog, digital);

        assert_eq!(data.get_analog("main_hook_weight"), Some(&17.0));
        assert_eq!(data.get_analog("radius"), Some(&10.0));
        assert_eq!(data.get_digital("main_hook_switch"), Some(&false));
    }

    #[test]
    fn test_get_with_default() {
        let data = SensorData::empty();

        assert_eq!(data.get_analog_or("nonexistent", 99.0), 99.0);
        assert_eq!(data.get_digital_or("nonexistent", true), true);
    }

    #[test]
    fn test_compatibility_api() {
        let data = SensorData::from_tuple(20.0, 10.0, 60.0, false, true);

        assert_eq!(data.ad1_load(), 20.0);
        assert_eq!(data.ad2_radius(), 10.0);
        assert_eq!(data.ad3_angle(), 60.0);
        assert_eq!(data.digital_input_0(), false);
        assert_eq!(data.digital_input_1(), true);
    }

    #[test]
    fn test_validate() {
        let mut analog = HashMap::new();
        analog.insert("main_hook_weight".to_string(), 17.0);
        let data = SensorData::new(analog, HashMap::new());
        assert!(data.validate().is_ok());

        let mut analog_invalid = HashMap::new();
        analog_invalid.insert("main_hook_weight".to_string(), -5.0);
        let invalid_data = SensorData::new(analog_invalid, HashMap::new());
        assert!(invalid_data.validate().is_err());
    }

    #[test]
    fn test_set_methods() {
        let mut data = SensorData::empty();

        data.set_analog("new_sensor", 42.0);
        assert_eq!(data.get_analog("new_sensor"), Some(&42.0));

        data.set_digital("new_switch", true);
        assert_eq!(data.get_digital("new_switch"), Some(&true));
    }

    #[test]
    fn test_aux_hook_sensor() {
        let mut analog = HashMap::new();
        analog.insert("main_hook_weight".to_string(), 30.0);
        analog.insert("aux_hook_weight".to_string(), 15.0);

        let data = SensorData::new(analog, HashMap::new());

        assert_eq!(data.get_analog("main_hook_weight"), Some(&30.0));
        assert_eq!(data.get_analog("aux_hook_weight"), Some(&15.0));
    }
}
