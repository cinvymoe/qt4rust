// 图表视图状态

use std::time::SystemTime;

/// 图表数据点
#[derive(Debug, Clone, PartialEq)]
pub struct ChartDataPoint {
    /// 时间戳
    pub timestamp: SystemTime,
    
    /// 载荷值（吨）
    pub load: f64,
    
    /// 力矩百分比
    pub moment_percentage: f64,
    
    /// 工作半径（米）
    pub radius: f64,
}

/// 图表视图状态
#[derive(Debug, Clone, PartialEq)]
pub struct ChartState {
    /// 历史数据点列表（最多保存 100 个点）
    pub data_points: Vec<ChartDataPoint>,
    
    /// 当前显示的时间范围（秒）
    pub time_range: u32,
    
    /// 是否正在加载数据
    pub is_loading: bool,
    
    /// 错误信息
    pub error_message: Option<String>,
    
    /// 最大数据点数量
    pub max_data_points: usize,
}

impl Default for ChartState {
    fn default() -> Self {
        Self {
            data_points: Vec::new(),
            time_range: 60, // 默认显示 60 秒
            is_loading: false,
            error_message: None,
            max_data_points: 100,
        }
    }
}

impl ChartState {
    /// 创建新的图表状态
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 添加新的数据点
    pub fn add_data_point(&mut self, point: ChartDataPoint) {
        self.data_points.push(point);
        
        // 保持数据点数量在限制内
        if self.data_points.len() > self.max_data_points {
            self.data_points.remove(0);
        }
    }
    
    /// 清空数据点
    pub fn clear_data_points(&mut self) {
        self.data_points.clear();
    }
    
    /// 获取指定时间范围内的数据点
    pub fn get_data_points_in_range(&self, seconds: u32) -> Vec<ChartDataPoint> {
        let now = SystemTime::now();
        self.data_points
            .iter()
            .filter(|point| {
                if let Ok(duration) = now.duration_since(point.timestamp) {
                    duration.as_secs() <= seconds as u64
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }
    
    /// 获取最大载荷值
    pub fn max_load(&self) -> f64 {
        self.data_points
            .iter()
            .map(|p| p.load)
            .fold(0.0, f64::max)
    }
    
    /// 获取平均载荷值
    pub fn average_load(&self) -> f64 {
        if self.data_points.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.data_points.iter().map(|p| p.load).sum();
        sum / self.data_points.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[test]
    fn test_default_state() {
        let state = ChartState::default();
        assert_eq!(state.data_points.len(), 0);
        assert_eq!(state.time_range, 60);
        assert!(!state.is_loading);
    }
    
    #[test]
    fn test_add_data_point() {
        let mut state = ChartState::default();
        
        let point = ChartDataPoint {
            timestamp: SystemTime::now(),
            load: 15.0,
            moment_percentage: 60.0,
            radius: 10.0,
        };
        
        state.add_data_point(point.clone());
        assert_eq!(state.data_points.len(), 1);
        assert_eq!(state.data_points[0].load, 15.0);
    }
    
    #[test]
    fn test_max_data_points_limit() {
        let mut state = ChartState::default();
        state.max_data_points = 5;
        
        for i in 0..10 {
            let point = ChartDataPoint {
                timestamp: SystemTime::now(),
                load: i as f64,
                moment_percentage: 50.0,
                radius: 10.0,
            };
            state.add_data_point(point);
        }
        
        assert_eq!(state.data_points.len(), 5);
        assert_eq!(state.data_points[0].load, 5.0); // 前 5 个被移除
    }
    
    #[test]
    fn test_statistics() {
        let mut state = ChartState::default();
        
        for load in [10.0, 20.0, 30.0] {
            let point = ChartDataPoint {
                timestamp: SystemTime::now(),
                load,
                moment_percentage: 50.0,
                radius: 10.0,
            };
            state.add_data_point(point);
        }
        
        assert_eq!(state.max_load(), 30.0);
        assert_eq!(state.average_load(), 20.0);
    }
}
