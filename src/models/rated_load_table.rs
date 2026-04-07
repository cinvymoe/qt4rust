// src/models/rated_load_table.rs

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// 额定载荷表条目（单条记录）
///
/// 表示特定臂长和working_radius组合下的额定载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatedLoadEntry {
    /// 臂长（米）
    #[serde(rename = "boom_length_m")]
    pub boom_length: f64,
    /// 工作幅度（米）
    #[serde(rename = "working_radius_m")]
    pub working_radius: f64,
    /// 额定载荷（吨）
    #[serde(rename = "rated_load_ton")]
    pub rated_load: f64,
}

/// 额定载荷表
///
/// 存储臂长与工作幅度与额定载荷的对应关系，用于查询指定条件下的额定载荷
///
/// # 数据结构
/// 使用 BTreeMap 实现 2D 查找：
/// - 外层 Key: 臂长 (i64, 毫米精度)
/// - 内层 Value: Vec<RatedLoadEntry> 按 working_radius 升序排列
///
/// # 查找算法
/// 1. 找到最接近的臂长（阶梯查找）
/// 2. 在该臂长下，找到第一个 >= working_radius 的条目
/// 3. 如果 working_radius 超过所有条目，返回最后一项
#[derive(Debug, Clone)]
pub struct RatedLoadTable {
    /// 按臂长分组的载荷条目
    entries_by_boom: BTreeMap<i64, Vec<RatedLoadEntry>>,
    /// 排序后的臂长列表（用于阶梯查找）
    boom_lengths: Vec<i64>,
    /// 力矩预警阈值（百分比）
    pub moment_warning_threshold: f64,
    /// 力矩报警阈值（百分比）
    pub moment_alarm_threshold: f64,

    #[deprecated(note = "使用 moment_warning_threshold 替代")]
    #[allow(dead_code)]
    pub alarm_threshold: f64,
    #[deprecated(note = "使用 moment_alarm_threshold 替代")]
    #[allow(dead_code)]
    pub danger_threshold: f64,
}

/// 将 f64 转换为 i64（毫米精度）
fn to_mm(value: f64) -> i64 {
    (value * 1000.0).round() as i64
}

/// 将 i64（毫米精度）转换为 f64
fn from_mm(value: i64) -> f64 {
    value as f64 / 1000.0
}

impl RatedLoadTable {
    pub fn get_rated_load(&self, boom_length: f64, working_radius: f64) -> f64 {
        if self.entries_by_boom.is_empty() {
            return 25.0;
        }

        let boom_key = to_mm(boom_length);
        let radius_key = to_mm(working_radius);

        // Step 1: 找到最接近的臂长
        let selected_boom = self.find_closest_boom_key(boom_key);

        // Step 2: 获取该臂长下的条目
        let entries = match self.entries_by_boom.get(&selected_boom) {
            Some(e) => e,
            None => return 25.0,
        };

        // Step 3: 找到第一个 >= working_radius 的条目
        for entry in entries {
            if to_mm(entry.working_radius) >= radius_key {
                return entry.rated_load;
            }
        }

        entries.last().map(|e| e.rated_load).unwrap_or(25.0)
    }

    fn find_closest_boom_key(&self, boom_key: i64) -> i64 {
        for &bl in &self.boom_lengths {
            if bl >= boom_key {
                return bl;
            }
        }
        self.boom_lengths.last().copied().unwrap_or(0)
    }

    pub fn add_entry(&mut self, boom_length: f64, working_radius: f64, rated_load: f64) {
        let entry = RatedLoadEntry {
            boom_length,
            working_radius,
            rated_load,
        };

        let boom_key = to_mm(boom_length);
        self.entries_by_boom
            .entry(boom_key)
            .or_insert_with(Vec::new)
            .push(entry);

        self.rebuild_boom_lengths();

        for entries in self.entries_by_boom.values_mut() {
            entries.sort_by(|a, b| to_mm(a.working_radius).cmp(&to_mm(b.working_radius)));
        }
    }

    fn rebuild_boom_lengths(&mut self) {
        self.boom_lengths = self.entries_by_boom.keys().copied().collect();
        self.boom_lengths.sort();
    }

    #[allow(deprecated)]
    pub fn from_entries_with_default_boom(
        entries: Vec<(f64, f64)>,
        moment_warning_threshold: f64,
        moment_alarm_threshold: f64,
    ) -> Self {
        let mut table = Self::new();
        let default_boom_length = 20.0;

        for (radius, rated_load) in entries {
            table.add_entry(default_boom_length, radius, rated_load);
        }

        table.moment_warning_threshold = moment_warning_threshold;
        table.moment_alarm_threshold = moment_alarm_threshold;
        table.alarm_threshold = moment_warning_threshold;
        table.danger_threshold = moment_alarm_threshold;

        table
    }

    pub fn is_moment_warning(&self, moment_percentage: f64) -> bool {
        moment_percentage >= self.moment_warning_threshold
    }

    pub fn is_moment_alarm(&self, moment_percentage: f64) -> bool {
        moment_percentage >= self.moment_alarm_threshold
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.entries_by_boom.is_empty() {
            return Err("载荷表不能为空".to_string());
        }

        for (&boom_key, entries) in &self.entries_by_boom {
            if entries.is_empty() {
                return Err(format!("臂长 {} 的载荷表为空", from_mm(boom_key)));
            }

            for entry in entries {
                if entry.rated_load <= 0.0 {
                    return Err(format!(
                        "额定载荷必须大于 0，当前值: {} (臂长: {}, 幅度: {})",
                        entry.rated_load, entry.boom_length, entry.working_radius
                    ));
                }
                if entry.working_radius < 0.0 {
                    return Err(format!(
                        "工作幅度不能为负数，当前值: {} (臂长: {})",
                        entry.working_radius, entry.boom_length
                    ));
                }
            }

            for i in 1..entries.len() {
                if entries[i].working_radius < entries[i - 1].working_radius {
                    return Err(format!(
                        "臂长 {} 的载荷表必须按工作幅度升序排列",
                        from_mm(boom_key)
                    ));
                }
            }
        }

        if self.moment_warning_threshold < 0.0 || self.moment_warning_threshold > 100.0 {
            return Err(format!(
                "力矩预警阈值必须在 0-100 范围内，当前值: {}",
                self.moment_warning_threshold
            ));
        }

        if self.moment_alarm_threshold < 0.0 || self.moment_alarm_threshold > 100.0 {
            return Err(format!(
                "力矩报警阈值必须在 0-100 范围内，当前值: {}",
                self.moment_alarm_threshold
            ));
        }

        if self.moment_alarm_threshold < self.moment_warning_threshold {
            return Err(format!(
                "力矩报警阈值 ({}) 必须大于等于力矩预警阈值 ({})",
                self.moment_alarm_threshold, self.moment_warning_threshold
            ));
        }

        Ok(())
    }

    pub fn get_boom_lengths(&self) -> Vec<f64> {
        self.boom_lengths.iter().map(|&k| from_mm(k)).collect()
    }

    pub fn get_entries_for_boom(&self, boom_length: f64) -> Option<&[RatedLoadEntry]> {
        let boom_key = to_mm(boom_length);
        let selected = self.find_closest_boom_key(boom_key);
        self.entries_by_boom.get(&selected).map(|v| v.as_slice())
    }

    pub fn get_all_entries(&self) -> Vec<&[RatedLoadEntry]> {
        self.boom_lengths
            .iter()
            .filter_map(|&bl| self.entries_by_boom.get(&bl).map(|v| v.as_slice()))
            .collect()
    }
}

impl Default for RatedLoadTable {
    fn default() -> Self {
        let mut table = Self::new();

        let default_entries = vec![
            (3.0, 50.0),
            (5.0, 40.0),
            (8.0, 30.0),
            (10.0, 25.0),
            (12.0, 20.0),
            (15.0, 15.0),
            (18.0, 10.0),
            (20.0, 8.0),
        ];

        let default_boom = 20.0;
        for (radius, load) in default_entries {
            table.add_entry(default_boom, radius, load);
        }

        table.moment_warning_threshold = 85.0;
        table.moment_alarm_threshold = 95.0;
        #[allow(deprecated)]
        {
            table.alarm_threshold = 85.0;
            table.danger_threshold = 95.0;
        }

        table
    }
}

impl RatedLoadTable {
    pub fn new() -> Self {
        Self {
            entries_by_boom: BTreeMap::new(),
            boom_lengths: Vec::new(),
            moment_warning_threshold: 85.0,
            moment_alarm_threshold: 95.0,
            #[allow(deprecated)]
            alarm_threshold: 85.0,
            #[allow(deprecated)]
            danger_threshold: 95.0,
        }
    }

    pub fn clear(&mut self) {
        self.entries_by_boom.clear();
        self.boom_lengths.clear();
    }

    pub fn len(&self) -> usize {
        self.entries_by_boom.values().map(|v| v.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.entries_by_boom.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_default_table() -> RatedLoadTable {
        RatedLoadTable::default()
    }

    #[test]
    fn test_get_rated_load_basic() {
        let mut table = RatedLoadTable::new();
        table.add_entry(10.0, 3.0, 50.0);
        table.add_entry(10.0, 5.0, 40.0);
        table.add_entry(10.0, 8.0, 30.0);
        table.add_entry(15.0, 3.0, 45.0);
        table.add_entry(15.0, 5.0, 35.0);
        table.add_entry(15.0, 8.0, 28.0);

        assert_eq!(table.get_rated_load(10.0, 5.0), 40.0);
        assert_eq!(table.get_rated_load(15.0, 5.0), 35.0);

        // 阶梯查找幅度
        assert_eq!(table.get_rated_load(10.0, 4.0), 40.0);
        assert_eq!(table.get_rated_load(10.0, 6.0), 30.0);

        // 阶梯查找臂长
        assert_eq!(table.get_rated_load(12.0, 3.0), 45.0);
    }

    #[test]
    fn test_get_rated_load_empty_table() {
        let table = RatedLoadTable::new();
        assert_eq!(table.get_rated_load(10.0, 5.0), 25.0);
    }

    #[test]
    fn test_find_closest_boom_key() {
        let mut table = RatedLoadTable::new();
        table.add_entry(10.0, 3.0, 50.0);
        table.add_entry(15.0, 3.0, 45.0);
        table.add_entry(20.0, 3.0, 40.0);

        assert_eq!(table.find_closest_boom_key(to_mm(15.0)), to_mm(15.0));
        assert_eq!(table.find_closest_boom_key(to_mm(12.0)), to_mm(15.0));
        assert_eq!(table.find_closest_boom_key(to_mm(8.0)), to_mm(10.0));
        assert_eq!(table.find_closest_boom_key(to_mm(25.0)), to_mm(20.0));
    }

    #[test]
    fn test_default_table() {
        let table = create_default_table();
        assert_eq!(table.get_rated_load(20.0, 5.0), 40.0);
        assert_eq!(table.get_rated_load(20.0, 10.0), 25.0);
        assert_eq!(table.get_rated_load(18.0, 5.0), 40.0);
    }

    #[test]
    fn test_validate_success() {
        let table = create_default_table();
        assert!(table.validate().is_ok());
    }

    #[test]
    fn test_validate_empty_table() {
        let table = RatedLoadTable::new();
        let result = table.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("载荷表不能为空"));
    }

    #[test]
    fn test_validate_negative_load() {
        let mut table = RatedLoadTable::new();
        table.add_entry(10.0, 5.0, -10.0);

        let result = table.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("额定载荷必须大于 0"));
    }

    #[test]
    fn test_validate_negative_radius() {
        let mut table = RatedLoadTable::new();
        table.add_entry(10.0, -5.0, 40.0);

        let result = table.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("工作幅度不能为负数"));
    }

    #[test]
    fn test_is_moment_warning() {
        let table = create_default_table();
        assert!(!table.is_moment_warning(80.0));
        assert!(table.is_moment_warning(85.0));
        assert!(table.is_moment_warning(90.0));
    }

    #[test]
    fn test_is_moment_alarm() {
        let table = create_default_table();
        assert!(!table.is_moment_alarm(94.0));
        assert!(table.is_moment_alarm(95.0));
        assert!(table.is_moment_alarm(100.0));
    }

    #[test]
    fn test_get_entries_for_boom() {
        let mut table = RatedLoadTable::new();
        table.add_entry(10.0, 3.0, 50.0);
        table.add_entry(10.0, 5.0, 40.0);
        table.add_entry(15.0, 3.0, 45.0);

        let entries = table.get_entries_for_boom(10.0).unwrap();
        assert_eq!(entries.len(), 2);

        let entries = table.get_entries_for_boom(12.0).unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[allow(deprecated)]
    #[test]
    fn test_from_entries_with_default_boom() {
        let entries = vec![(3.0, 50.0), (5.0, 40.0), (8.0, 30.0)];
        let table = RatedLoadTable::from_entries_with_default_boom(entries, 85.0, 95.0);

        assert_eq!(table.get_rated_load(20.0, 5.0), 40.0);
        assert_eq!(table.get_rated_load(25.0, 5.0), 40.0);
    }

    #[test]
    fn test_mm_conversion() {
        assert_eq!(to_mm(10.0), 10000);
        assert_eq!(to_mm(3.5), 3500);
        assert_eq!(from_mm(10000), 10.0);
        assert_eq!(from_mm(3500), 3.5);
    }
}
