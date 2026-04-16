use super::channel::{
    create_sensor_data_channels, create_storage_channels,
    SensorDataEventReceiver, SensorDataEventSender,
    StorageEventReceiver, StorageEventSender,
};
use crate::models::ProcessedData;
use crate::pipeline::core::StorageError;
use sensor_core::SensorData;

/// 管道事件类型
#[derive(Debug, Clone)]
pub enum PipelineEvent {
    /// 存储事件 - 处理后的数据
    Storage(Vec<ProcessedData>),
    /// 报警事件 - 触发报警
    Alarm(ProcessedData),
    /// 报警解除事件
    AlarmCleared,
    /// 传感器原始数据事件
    SensorData(Vec<SensorData>),
    /// 关闭事件
    Shutdown,
}

/// 统一事件总线
///
/// 提供统一的事件发布接口，支持多种事件类型：
/// - Storage: 处理后数据存储
/// - Alarm: 报警触发/解除
/// - SensorData: 原始传感器数据
#[derive(Clone)]
pub struct EventBus {
    storage_sender: Option<StorageEventSender>,
    sensor_sender: Option<SensorDataEventSender>,
}

/// Factory for creating EventBus with receiver access
pub struct EventBusChannels {
    pub bus: EventBus,
    pub storage_receiver: StorageEventReceiver,
    pub sensor_receiver: SensorDataEventReceiver,
}

impl EventBusChannels {
    pub fn new(capacity: usize) -> Self {
        let (storage_tx, storage_rx) = create_storage_channels(capacity);
        let (sensor_tx, sensor_rx) = create_sensor_data_channels(capacity);

        Self {
            bus: EventBus {
                storage_sender: Some(storage_tx),
                sensor_sender: Some(sensor_tx),
            },
            storage_receiver: storage_rx,
            sensor_receiver: sensor_rx,
        }
    }
}

impl EventBus {
    /// 创建新的事件总线
    pub fn new(capacity: usize) -> Self {
        let (storage_tx, _storage_rx) = create_storage_channels(capacity);
        let (sensor_tx, _sensor_rx) = create_sensor_data_channels(capacity);

        Self {
            storage_sender: Some(storage_tx),
            sensor_sender: Some(sensor_tx),
        }
    }

    /// 使用现有 channel 创建
    pub fn with_channels(
        storage_sender: StorageEventSender,
        sensor_sender: SensorDataEventSender,
    ) -> Self {
        Self {
            storage_sender: Some(storage_sender),
            sensor_sender: Some(sensor_sender),
        }
    }

    /// 仅存储通道
    pub fn storage_only(storage_sender: StorageEventSender) -> Self {
        Self {
            storage_sender: Some(storage_sender),
            sensor_sender: None,
        }
    }

    /// 发布事件
    pub fn emit(&self, event: PipelineEvent) -> Result<(), StorageError> {
        match event {
            PipelineEvent::Storage(data) => {
                if let Some(ref sender) = self.storage_sender {
                    sender.try_send_data(data)?;
                }
            }
            PipelineEvent::Alarm(data) => {
                if let Some(ref sender) = self.storage_sender {
                    sender.try_send_data(vec![data])?;
                }
            }
            PipelineEvent::AlarmCleared => {
                tracing::info!("Alarm cleared event emitted");
            }
            PipelineEvent::SensorData(data) => {
                if let Some(ref sender) = self.sensor_sender {
                    sender.try_send_data(data)?;
                }
            }
            PipelineEvent::Shutdown => {
                if let Some(ref sender) = self.storage_sender {
                    sender.shutdown();
                }
                if let Some(ref sender) = self.sensor_sender {
                    sender.shutdown();
                }
            }
        }
        Ok(())
    }

    /// 获取存储事件发送器
    pub fn storage_sender(&self) -> Option<StorageEventSender> {
        self.storage_sender.clone()
    }

    /// 获取传感器数据发送器
    pub fn sensor_sender(&self) -> Option<SensorDataEventSender> {
        self.sensor_sender.clone()
    }

    /// 检查是否有存储通道
    pub fn has_storage(&self) -> bool {
        self.storage_sender.is_some()
    }

    /// 检查是否有传感器通道
    pub fn has_sensor(&self) -> bool {
        self.sensor_sender.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_data() -> ProcessedData {
        ProcessedData {
            current_load: 10.0,
            rated_load: 25.0,
            working_radius: 5.0,
            boom_angle: 45.0,
            boom_length: 10.0,
            moment_percentage: 50.0,
            is_warning: false,
            is_danger: false,
            validation_error: None,
            timestamp: std::time::SystemTime::now(),
            sequence_number: 1,
            alarm_sources: Vec::new(),
            alarm_messages: Vec::new(),
        }
    }

    fn create_test_sensor_data() -> SensorData {
        SensorData {
            ad1_load: 10.0,
            ad2_radius: 5.0,
            ad3_angle: 45.0,
            digital_input_0: false,
            digital_input_1: false,
        }
    }

    #[test]
    fn test_event_bus_creation() {
        let channels = EventBusChannels::new(100);
        assert!(channels.bus.storage_sender().is_some());
        assert!(channels.bus.sensor_sender().is_some());
        assert!(channels.bus.has_storage());
        assert!(channels.bus.has_sensor());
    }

    #[test]
    fn test_event_bus_storage_only() {
        let (storage_tx, _storage_rx) = create_storage_channels(100);
        let bus = EventBus::storage_only(storage_tx);

        assert!(bus.has_storage());
        assert!(!bus.has_sensor());
        assert!(bus.storage_sender().is_some());
        assert!(bus.sensor_sender().is_none());
    }

    #[test]
    fn test_event_bus_clone() {
        let channels = EventBusChannels::new(100);
        let bus2 = channels.bus.clone();

        assert!(bus2.storage_sender().is_some());
        assert!(bus2.sensor_sender().is_some());
    }

    #[tokio::test]
    async fn test_emit_storage_event() {
        let channels = EventBusChannels::new(100);
        let data = vec![create_test_data()];

        let result = channels.bus.emit(PipelineEvent::Storage(data));
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_emit_alarm_event() {
        let channels = EventBusChannels::new(100);
        let data = create_test_data();

        let result = channels.bus.emit(PipelineEvent::Alarm(data));
        assert!(result.is_ok());
    }

    #[test]
    fn test_emit_alarm_cleared_event() {
        let channels = EventBusChannels::new(100);

        let result = channels.bus.emit(PipelineEvent::AlarmCleared);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_emit_sensor_data_event() {
        let channels = EventBusChannels::new(100);
        let data = vec![create_test_sensor_data()];

        let result = channels.bus.emit(PipelineEvent::SensorData(data));
        assert!(result.is_ok());
    }

    #[test]
    fn test_emit_shutdown_event() {
        let channels = EventBusChannels::new(100);

        let result = channels.bus.emit(PipelineEvent::Shutdown);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_emit_to_storage_only_bus() {
        let (storage_tx, _storage_rx) = create_storage_channels(100);
        let bus = EventBus::storage_only(storage_tx);

        let result = bus.emit(PipelineEvent::Storage(vec![create_test_data()]));
        assert!(result.is_ok());

        let result = bus.emit(PipelineEvent::SensorData(vec![create_test_sensor_data()]));
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_emit_returns_queue_full_error() {
        let channels = EventBusChannels::new(1);

        let _ = channels
            .bus
            .emit(PipelineEvent::Storage(vec![create_test_data()]));

        let result = channels
            .bus
            .emit(PipelineEvent::Storage(vec![create_test_data()]));
        assert!(matches!(result, Err(StorageError::QueueFull)));
    }
}
