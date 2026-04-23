// 报警统计结构体

/// 报警统计信息
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AlarmStatistics {
    /// 总报警次数
    pub total_count: i32,
    
    /// 预警次数
    pub warning_count: i32,
    
    /// 危险次数
    pub danger_count: i32,
}

impl AlarmStatistics {
    /// 创建新的统计实例
    pub fn new(total_count: i32, warning_count: i32, danger_count: i32) -> Self {
        Self {
            total_count,
            warning_count,
            danger_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let stats = AlarmStatistics::default();
        assert_eq!(stats.total_count, 0);
        assert_eq!(stats.warning_count, 0);
        assert_eq!(stats.danger_count, 0);
    }

    #[test]
    fn test_new() {
        let stats = AlarmStatistics::new(10, 7, 3);
        assert_eq!(stats.total_count, 10);
        assert_eq!(stats.warning_count, 7);
        assert_eq!(stats.danger_count, 3);
    }
}
