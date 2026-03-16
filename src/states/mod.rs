// States 模块
// 定义应用的所有状态结构

pub mod monitoring_state;
pub mod chart_state;
pub mod alarm_state;
pub mod common_state;

// 重新导出常用类型
pub use monitoring_state::MonitoringState;
pub use chart_state::ChartState;
pub use alarm_state::AlarmState;
pub use common_state::{LoadingState, ErrorState};
