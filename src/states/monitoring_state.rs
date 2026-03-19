// 监控视图状态

/// 监控视图状态
#[derive(Debug, Clone, PartialEq)]
pub struct MonitoringState {
    /// 当前载荷（吨）
    pub current_load: f64,
    /// 额定载荷（吨）
    pub rated_load: f64,
    /// 工作半径（米）
    pub working_radius: f64,
    /// 吊臂角度（度）
    pub boom_angle: f64,
    /// 臂长（米）
    pub boom_length: f64,
    /// 力矩百分比
    pub moment_percentage: f64,
    /// 是否处于危险状态
    pub is_danger: bool,
    /// 传感器连接状态
    pub sensor_connected: bool,
    /// 错误信息
    pub error_message: Option<String>,
    /// 最后更新时间
    pub last_update_time: std::time::SystemTime,
}

impl Default for MonitoringState {
    fn default() -> Self {
        Self {
            current_load: 0.0,
            rated_load: 25.0,
            working_radius: 0.0,
            boom_angle: 0.0,
            boom_length: 0.0,
            moment_percentage: 0.0,
            is_danger: false,
            sensor_connected: false,
            error_message: None,
            last_update_time: std::time::SystemTime::now(),
        }
    }
}
