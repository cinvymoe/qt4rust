// 监控视图意图

use crate::models::SensorData;
use cxx_qt_mvi_core::prelude::Intent;

/// 监控视图意图
#[derive(Debug, Clone)]
pub enum MonitoringIntent {
    /// 传感器数据更新（后台线程触发）
    SensorDataUpdated(SensorData),
    
    /// 清除错误信息
    ClearError,
    
    /// 重置报警状态
    ResetAlarm,
    
    /// 传感器断连
    SensorDisconnected,
    
    /// 传感器重连
    SensorReconnected,
}

impl Intent for MonitoringIntent {}
