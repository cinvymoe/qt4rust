// src/models/rated_load_table.rs

use serde::{Deserialize, Serialize};

/// 额定载荷表条目
/// 
/// 表示特定工作半径对应的额定载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatedLoadEntry {
    /// 工作半径（米）
    #[serde(rename = "radius_m")]
    pub radius: f64,
    /// 额定载荷（吨）
    #[serde(rename = "rated_load_ton")]
    pub rated_load: f64,
}

/// 额定载荷表
/// 
/// 存储工作半径与额定载荷的对应关系，用于查询指定半径下的额定载荷
/// 并提供力矩预警和报警判断功能
#[derive(Debug, Clone)]
pub struct RatedLoadTable {
    /// 载荷表条目列表（按半径升序排列）
    pub entries: Vec<RatedLoadEntry>,
    /// 力矩预警阈值（百分比）
    pub moment_warning_threshold: f64,
    /// 力矩报警阈值（百分比）
    pub moment_alarm_threshold: f64,
    
    // 保留旧字段名以兼容（已废弃）
    #[deprecated(note = "使用 moment_warning_threshold 替代")]
    #[allow(dead_code)]
    pub alarm_threshold: f64,
    #[deprecated(note = "使用 moment_alarm_threshold 替代")]
    #[allow(dead_code)]
    pub danger_threshold: f64,
}

impl RatedLoadTable {
    /// 根据工作半径查询额定载荷（阶梯查找）
    /// 
    /// 查找规则：
    /// - 找到第一个 >= 当前半径的表项，返回其额定载荷
    /// - 如果当前半径大于所有表项，返回最后一项的额定载荷
    /// - 如果表为空，返回默认值 25.0 吨
    /// 
    /// # 参数
    /// - `radius`: 当前工作半径（米）
    /// 
    /// # 返回
    /// 对应的额定载荷（吨）
    /// 
    /// # 示例
    /// ```
    /// use crate::models::rated_load_table::{RatedLoadTable, RatedLoadEntry};
    /// 
    /// let table = RatedLoadTable {
    ///     entries: vec![
    ///         RatedLoadEntry { radius: 5.0, rated_load: 40.0 },
    ///         RatedLoadEntry { radius: 10.0, rated_load: 25.0 },
    ///     ],
    ///     moment_warning_threshold: 85.0,
    ///     moment_alarm_threshold: 95.0,
    ///     alarm_threshold: 85.0,
    ///     danger_threshold: 95.0,
    /// };
    /// 
    /// assert_eq!(table.get_rated_load(3.0), 40.0);   // < 5.0
    /// assert_eq!(table.get_rated_load(5.0), 40.0);   // = 5.0
    /// assert_eq!(table.get_rated_load(7.5), 25.0);   // 5.0 < x < 10.0
    /// assert_eq!(table.get_rated_load(15.0), 25.0);  // > 10.0
    /// ```
    pub fn get_rated_load(&self, radius: f64) -> f64 {
        if self.entries.is_empty() {
            return 25.0; // 默认值
        }
        
        // 找到第一个 >= 当前幅度的表项
        for entry in &self.entries {
            if entry.radius >= radius {
                return entry.rated_load;
            }
        }
        
        // 如果当前幅度大于所有表项，返回最后一项
        self.entries.last().unwrap().rated_load
    }
    
    /// 检查力矩百分比是否达到预警阈值
    /// 
    /// # 参数
    /// - `moment_percentage`: 当前力矩百分比（%）
    /// 
    /// # 返回
    /// 如果力矩百分比 >= 预警阈值，返回 true
    pub fn is_moment_warning(&self, moment_percentage: f64) -> bool {
        moment_percentage >= self.moment_warning_threshold
    }
    
    /// 检查力矩百分比是否达到报警阈值
    /// 
    /// # 参数
    /// - `moment_percentage`: 当前力矩百分比（%）
    /// 
    /// # 返回
    /// 如果力矩百分比 >= 报警阈值，返回 true
    pub fn is_moment_alarm(&self, moment_percentage: f64) -> bool {
        moment_percentage >= self.moment_alarm_threshold
    }
    
    /// 验证载荷表的有效性
    /// 
    /// 检查载荷表是否满足以下条件：
    /// - 载荷表不能为空
    /// - 所有额定载荷必须大于 0
    /// - 所有半径不能为负数
    /// - 载荷表必须按半径升序排列
    /// - 力矩预警阈值必须在 0-100 范围内
    /// - 力矩报警阈值必须在 0-100 范围内
    /// - 力矩报警阈值必须大于等于预警阈值
    /// 
    /// # 返回
    /// - `Ok(())`: 载荷表有效
    /// - `Err(String)`: 载荷表无效，包含错误描述
    pub fn validate(&self) -> Result<(), String> {
        // 检查载荷表是否为空
        if self.entries.is_empty() {
            return Err("载荷表不能为空".to_string());
        }
        
        // 检查每个条目的有效性
        for entry in &self.entries {
            if entry.rated_load <= 0.0 {
                return Err(format!(
                    "额定载荷必须大于 0，当前值: {}",
                    entry.rated_load
                ));
            }
            if entry.radius < 0.0 {
                return Err(format!(
                    "半径不能为负数，当前值: {}",
                    entry.radius
                ));
            }
        }
        
        // 检查是否按半径升序排列
        for i in 1..self.entries.len() {
            if self.entries[i].radius < self.entries[i - 1].radius {
                return Err("载荷表必须按半径升序排列".to_string());
            }
        }
        
        // 检查力矩预警阈值范围
        if self.moment_warning_threshold < 0.0 || self.moment_warning_threshold > 100.0 {
            return Err(format!(
                "力矩预警阈值必须在 0-100 范围内，当前值: {}",
                self.moment_warning_threshold
            ));
        }
        
        // 检查力矩报警阈值范围
        if self.moment_alarm_threshold < 0.0 || self.moment_alarm_threshold > 100.0 {
            return Err(format!(
                "力矩报警阈值必须在 0-100 范围内，当前值: {}",
                self.moment_alarm_threshold
            ));
        }
        
        // 检查报警阈值必须大于等于预警阈值
        if self.moment_alarm_threshold < self.moment_warning_threshold {
            return Err(format!(
                "力矩报警阈值 ({}) 必须大于等于力矩预警阈值 ({})",
                self.moment_alarm_threshold, self.moment_warning_threshold
            ));
        }
        
        Ok(())
    }
}

impl Default for RatedLoadTable {
    /// 提供默认载荷表
    /// 
    /// 默认配置：
    /// - 8 个载荷点，覆盖 3-20 米工作半径
    /// - 力矩预警阈值: 85%
    /// - 力矩报警阈值: 95%
    fn default() -> Self {
        Self {
            entries: vec![
                RatedLoadEntry { radius: 3.0, rated_load: 50.0 },
                RatedLoadEntry { radius: 5.0, rated_load: 40.0 },
                RatedLoadEntry { radius: 8.0, rated_load: 30.0 },
                RatedLoadEntry { radius: 10.0, rated_load: 25.0 },
                RatedLoadEntry { radius: 12.0, rated_load: 20.0 },
                RatedLoadEntry { radius: 15.0, rated_load: 15.0 },
                RatedLoadEntry { radius: 18.0, rated_load: 10.0 },
                RatedLoadEntry { radius: 20.0, rated_load: 8.0 },
            ],
            moment_warning_threshold: 85.0,   // 力矩预警阈值 85%
            moment_alarm_threshold: 95.0,     // 力矩报警阈值 95%
            #[allow(deprecated)]
            alarm_threshold: 85.0,            // 兼容旧字段
            #[allow(deprecated)]
            danger_threshold: 95.0,           // 兼容旧字段
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_rated_load_basic() {
        let table = RatedLoadTable::default();
        
        // 测试小于最小半径
        assert_eq!(table.get_rated_load(2.5), 50.0);
        
        // 测试等于表项半径
        assert_eq!(table.get_rated_load(3.0), 50.0);
        assert_eq!(table.get_rated_load(5.0), 40.0);
        assert_eq!(table.get_rated_load(10.0), 25.0);
        
        // 测试介于两个表项之间
        assert_eq!(table.get_rated_load(7.5), 30.0);  // 5.0 < 7.5 < 8.0
        assert_eq!(table.get_rated_load(11.0), 20.0); // 10.0 < 11.0 < 12.0
        
        // 测试大于最大半径
        assert_eq!(table.get_rated_load(25.0), 8.0);
    }
    
    #[test]
    fn test_get_rated_load_empty_table() {
        let table = RatedLoadTable {
            entries: vec![],
            moment_warning_threshold: 85.0,
            moment_alarm_threshold: 95.0,
            alarm_threshold: 85.0,
            danger_threshold: 95.0,
        };
        
        // 空表返回默认值
        assert_eq!(table.get_rated_load(10.0), 25.0);
    }
    
    #[test]
    fn test_get_rated_load_single_entry() {
        let table = RatedLoadTable {
            entries: vec![
                RatedLoadEntry { radius: 10.0, rated_load: 30.0 },
            ],
            moment_warning_threshold: 85.0,
            moment_alarm_threshold: 95.0,
            alarm_threshold: 85.0,
            danger_threshold: 95.0,
        };
        
        // 小于唯一表项
        assert_eq!(table.get_rated_load(5.0), 30.0);
        
        // 等于唯一表项
        assert_eq!(table.get_rated_load(10.0), 30.0);
        
        // 大于唯一表项
        assert_eq!(table.get_rated_load(15.0), 30.0);
    }
    
    #[test]
    fn test_is_moment_warning() {
        let table = RatedLoadTable::default();
        
        // 低于预警阈值
        assert!(!table.is_moment_warning(80.0));
        assert!(!table.is_moment_warning(84.9));
        
        // 等于预警阈值
        assert!(table.is_moment_warning(85.0));
        
        // 高于预警阈值
        assert!(table.is_moment_warning(90.0));
        assert!(table.is_moment_warning(95.0));
    }
    
    #[test]
    fn test_is_moment_alarm() {
        let table = RatedLoadTable::default();
        
        // 低于报警阈值
        assert!(!table.is_moment_alarm(80.0));
        assert!(!table.is_moment_alarm(94.9));
        
        // 等于报警阈值
        assert!(table.is_moment_alarm(95.0));
        
        // 高于报警阈值
        assert!(table.is_moment_alarm(100.0));
        assert!(table.is_moment_alarm(105.0));
    }
    
    #[test]
    fn test_validate_success() {
        let table = RatedLoadTable::default();
        assert!(table.validate().is_ok());
    }
    
    #[test]
    fn test_validate_empty_table() {
        let table = RatedLoadTable {
            entries: vec![],
            moment_warning_threshold: 85.0,
            moment_alarm_threshold: 95.0,
            alarm_threshold: 85.0,
            danger_threshold: 95.0,
        };
        
        let result = table.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("载荷表不能为空"));
    }
    
    #[test]
    fn test_validate_negative_load() {
        let table = RatedLoadTable {
            entries: vec![
                RatedLoadEntry { radius: 5.0, rated_load: -10.0 },
            ],
            moment_warning_threshold: 85.0,
            moment_alarm_threshold: 95.0,
            alarm_threshold: 85.0,
            danger_threshold: 95.0,
        };
        
        let result = table.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("额定载荷必须大于 0"));
    }
    
    #[test]
    fn test_validate_zero_load() {
        let table = RatedLoadTable {
            entries: vec![
                RatedLoadEntry { radius: 5.0, rated_load: 0.0 },
            ],
            moment_warning_threshold: 85.0,
            moment_alarm_threshold: 95.0,
            alarm_threshold: 85.0,
            danger_threshold: 95.0,
        };
        
        let result = table.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("额定载荷必须大于 0"));
    }
    
    #[test]
    fn test_validate_negative_radius() {
        let table = RatedLoadTable {
            entries: vec![
                RatedLoadEntry { radius: -5.0, rated_load: 30.0 },
            ],
            moment_warning_threshold: 85.0,
            moment_alarm_threshold: 95.0,
            alarm_threshold: 85.0,
            danger_threshold: 95.0,
        };
        
        let result = table.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("半径不能为负数"));
    }
    
    #[test]
    fn test_validate_unsorted_entries() {
        let table = RatedLoadTable {
            entries: vec![
                RatedLoadEntry { radius: 10.0, rated_load: 25.0 },
                RatedLoadEntry { radius: 5.0, rated_load: 40.0 },  // 乱序
            ],
            moment_warning_threshold: 85.0,
            moment_alarm_threshold: 95.0,
            alarm_threshold: 85.0,
            danger_threshold: 95.0,
        };
        
        let result = table.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("载荷表必须按半径升序排列"));
    }
    
    #[test]
    fn test_validate_warning_threshold_range() {
        let mut table = RatedLoadTable::default();
        
        // 负值
        table.moment_warning_threshold = -10.0;
        assert!(table.validate().is_err());
        
        // 超过 100%
        table.moment_warning_threshold = 110.0;
        assert!(table.validate().is_err());
        
        // 边界值有效
        table.moment_warning_threshold = 0.0;
        table.moment_alarm_threshold = 100.0;
        assert!(table.validate().is_ok());
    }
    
    #[test]
    fn test_validate_alarm_threshold_range() {
        let mut table = RatedLoadTable::default();
        
        // 负值
        table.moment_alarm_threshold = -10.0;
        assert!(table.validate().is_err());
        
        // 超过 100%
        table.moment_alarm_threshold = 110.0;
        assert!(table.validate().is_err());
    }
    
    #[test]
    fn test_validate_alarm_less_than_warning() {
        let mut table = RatedLoadTable::default();
        table.moment_alarm_threshold = 80.0;
        table.moment_warning_threshold = 90.0;
        
        let result = table.validate();
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("力矩报警阈值"));
        assert!(error_msg.contains("力矩预警阈值"));
    }
    
    #[test]
    fn test_validate_alarm_equal_warning() {
        let mut table = RatedLoadTable::default();
        table.moment_alarm_threshold = 90.0;
        table.moment_warning_threshold = 90.0;
        
        assert!(table.validate().is_ok());
    }
    
    #[test]
    fn test_get_rated_load_boundary_cases() {
        let table = RatedLoadTable::default();
        
        // 测试边界值
        assert_eq!(table.get_rated_load(0.0), 50.0);
        assert_eq!(table.get_rated_load(3.0), 50.0);
        assert_eq!(table.get_rated_load(3.1), 40.0);
        assert_eq!(table.get_rated_load(4.9), 40.0);
        assert_eq!(table.get_rated_load(5.0), 40.0);
        assert_eq!(table.get_rated_load(5.1), 30.0);
    }
    
    #[test]
    fn test_default_table_is_valid() {
        let table = RatedLoadTable::default();
        assert!(table.validate().is_ok());
        
        // 验证默认表是升序的
        for i in 1..table.entries.len() {
            assert!(table.entries[i].radius > table.entries[i - 1].radius);
        }
        
        // 验证所有载荷都是正数
        for entry in &table.entries {
            assert!(entry.rated_load > 0.0);
            assert!(entry.radius >= 0.0);
        }
    }
    
    #[test]
    fn test_moment_thresholds() {
        let table = RatedLoadTable::default();
        
        // 测试预警和报警的关系
        assert!(table.moment_alarm_threshold >= table.moment_warning_threshold);
        
        // 测试阈值在合理范围内
        assert!(table.moment_warning_threshold >= 0.0);
        assert!(table.moment_warning_threshold <= 100.0);
        assert!(table.moment_alarm_threshold >= 0.0);
        assert!(table.moment_alarm_threshold <= 100.0);
    }
}
