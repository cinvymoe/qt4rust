// 参数校准视图意图

/// 参数校准视图意图
#[derive(Debug, Clone)]
pub enum CalibrationIntent {
    /// 数据更新（包含AD值和计算后的物理量）
    DataUpdated {
        ad1_load: f64,
        ad2_radius: f64,
        ad3_angle: f64,
        calculated_load: f64,
        calculated_radius: f64,
        calculated_angle: f64,
    },
    
    /// 传感器断连
    SensorDisconnected,
    
    /// 传感器重连
    SensorReconnected,
    
    /// 清除错误信息
    ClearError,
}
