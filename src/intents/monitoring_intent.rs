// 监控视图意图

use crate::models::ProcessedData;

/// 监控视图意图
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum MonitoringIntent {
    /// 清除错误信息
    ClearError,

    /// 重置报警状态
    ResetAlarm,

    /// 已处理数据更新（从共享管道获取）
    ProcessedDataUpdated(ProcessedData),

    /// 传感器断连
    SensorDisconnected,

    /// 传感器重连
    SensorReconnected,
}
