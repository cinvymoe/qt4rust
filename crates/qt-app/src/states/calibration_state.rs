// 参数校准视图状态

/// 参数校准视图状态
#[derive(Debug, Clone, PartialEq)]
pub struct CalibrationState {
    /// AD1 - 载荷传感器原始值
    pub ad1_load: f64,

    /// AD2 - 工作半径传感器原始值
    pub ad2_radius: f64,

    /// AD3 - 吊臂角度传感器原始值
    pub ad3_angle: f64,

    /// 计算后的载荷值（吨）
    pub calculated_load: f64,

    /// 计算后的半径值（米）
    pub calculated_radius: f64,

    /// 计算后的角度值（度）
    pub calculated_angle: f64,

    /// 传感器连接状态
    pub sensor_connected: bool,

    /// 错误信息
    pub error_message: Option<String>,

    /// 最后更新时间
    pub last_update_time: std::time::SystemTime,
}

impl Default for CalibrationState {
    fn default() -> Self {
        Self {
            ad1_load: 0.0,
            ad2_radius: 0.0,
            ad3_angle: 0.0,
            calculated_load: 0.0,
            calculated_radius: 0.0,
            calculated_angle: 0.0,
            sensor_connected: false,
            error_message: None,
            last_update_time: std::time::SystemTime::now(),
        }
    }
}
