// 历史数据视图 ViewModel

use core::pin::Pin;
use cxx_qt_lib::QString;
use qt_rust_demo::models::{AlarmRecord, ProcessedData};
use qt_rust_demo::repositories::storage_repository::StorageRepository;
use qt_rust_demo::repositories::sqlite_storage_repository::SqliteStorageRepository;
use serde_json::json;
use std::cell::RefCell;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[cxx_qt::bridge]
pub mod history_viewmodel_bridge {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(i32, total_alarm_count)]
        #[qproperty(i32, warning_count)]
        #[qproperty(i32, danger_count)]
        #[qproperty(QString, selected_filter)]
        #[qproperty(bool, is_loading)]
        #[qproperty(QString, error_message)]
        #[qproperty(QString, chart_data_json)]
        #[qproperty(QString, alarm_records_json)]
        type HistoryViewModel = super::HistoryViewModelRust;

        /// 刷新图表数据
        #[qinvokable]
        fn refresh_chart_data(self: Pin<&mut HistoryViewModel>);

        /// 刷新报警记录
        #[qinvokable]
        fn refresh_alarm_records(self: Pin<&mut HistoryViewModel>);

        /// 设置筛选条件
        #[qinvokable]
        fn set_filter(self: Pin<&mut HistoryViewModel>, filter: QString);

        /// 设置自定义时间范围
        #[qinvokable]
        fn set_custom_time_range(
            self: Pin<&mut HistoryViewModel>,
            start_timestamp: i64,
            end_timestamp: i64,
        );

        /// 初始化（设置仓库引用）
        #[qinvokable]
        fn init_with_repository(self: Pin<&mut HistoryViewModel>);
    }
}

/// HistoryViewModel 实现
pub struct HistoryViewModelRust {
    total_alarm_count: i32,
    warning_count: i32,
    danger_count: i32,
    selected_filter: QString,
    is_loading: bool,
    error_message: QString,
    chart_data_json: QString,
    alarm_records_json: QString,

    // 内部状态（使用 RefCell 实现内部可变性）
    custom_start_time: RefCell<Option<SystemTime>>,
    custom_end_time: RefCell<Option<SystemTime>>,
    storage: RefCell<Option<Arc<dyn StorageRepository>>>,
}

impl Default for HistoryViewModelRust {
    fn default() -> Self {
        Self {
            total_alarm_count: 0,
            warning_count: 0,
            danger_count: 0,
            selected_filter: QString::from("all"),
            is_loading: false,
            error_message: QString::from(""),
            chart_data_json: QString::from("{}"),
            alarm_records_json: QString::from("[]"),
            custom_start_time: RefCell::new(None),
            custom_end_time: RefCell::new(None),
            storage: RefCell::new(None),
        }
    }
}

impl history_viewmodel_bridge::HistoryViewModel {
    /// 设置存储仓库引用（从外部调用）
    pub fn set_storage(&self, storage: Arc<dyn StorageRepository>) {
        *self.storage.borrow_mut() = Some(storage);
    }
}

impl history_viewmodel_bridge::HistoryViewModel {
    /// 刷新图表数据
    fn refresh_chart_data(mut self: Pin<&mut Self>) {
        self.as_mut().set_is_loading(true);
        self.as_mut().set_error_message(QString::from(""));

        let filter = self.as_ref().selected_filter().to_string();
        let start_time = *self.custom_start_time.borrow();
        let end_time = *self.custom_end_time.borrow();
        let storage = self.storage.borrow().clone();

        if let Some(storage) = storage {
            // 使用全局 tokio runtime 进行同步查询（block_on）
            // 注意：在 Qt 主线程上执行，会阻塞 UI，但数据量小（100条）影响不大
            let result = if let Some((start, end)) = start_time.zip(end_time) {
                qt_threading_utils::runtime::global_runtime().block_on(async {
                    storage.query_runtime_data_by_time_range(start, end, 100).await
                })
            } else {
                qt_threading_utils::runtime::global_runtime().block_on(async {
                    storage.query_runtime_data_by_filter(&filter, 100).await
                })
            };

            match result {
                Ok(data) => {
                    tracing::info!("Loaded {} chart data points", data.len());
                    let json = processed_data_to_json(&data);
                    self.as_mut().set_chart_data_json(json);
                }
                Err(e) => {
                    tracing::error!("Failed to load chart data: {}", e);
                    self.as_mut().set_error_message(QString::from(&e));
                }
            }
        } else {
            tracing::warn!("Storage repository not initialized");
            self.as_mut().set_error_message(QString::from("存储未初始化，请先调用 init_with_repository"));
        }

        self.as_mut().set_is_loading(false);
    }

    /// 刷新报警记录
    fn refresh_alarm_records(mut self: Pin<&mut Self>) {
        self.as_mut().set_is_loading(true);
        self.as_mut().set_error_message(QString::from(""));

        let filter = self.as_ref().selected_filter().to_string();
        let start_time = *self.custom_start_time.borrow();
        let end_time = *self.custom_end_time.borrow();
        let storage = self.storage.borrow().clone();

        if let Some(storage) = storage {
            // 使用全局 tokio runtime 进行同步查询
            let result = if let Some((start, end)) = start_time.zip(end_time) {
                qt_threading_utils::runtime::global_runtime().block_on(async {
                    let records = storage.query_alarm_records_by_time_range(start, end).await;
                    let stats = storage.get_alarm_statistics_by_time_range(start, end).await;
                    (records, stats)
                })
            } else {
                qt_threading_utils::runtime::global_runtime().block_on(async {
                    let records = storage.query_alarm_records_by_filter(&filter).await;
                    let stats = storage.get_alarm_statistics().await;
                    (records, stats)
                })
            };

            match result {
                (Ok(records), Ok(stats)) => {
                    tracing::info!(
                        "Loaded {} alarm records, stats: total={}, warning={}, danger={}",
                        records.len(),
                        stats.total_count,
                        stats.warning_count,
                        stats.danger_count
                    );

                    self.as_mut().set_total_alarm_count(stats.total_count);
                    self.as_mut().set_warning_count(stats.warning_count);
                    self.as_mut().set_danger_count(stats.danger_count);

                    let json = alarm_records_to_json(&records);
                    self.as_mut().set_alarm_records_json(json);
                }
                (Err(e), _) | (_, Err(e)) => {
                    tracing::error!("Failed to load alarm records: {}", e);
                    self.as_mut().set_error_message(QString::from(&e));
                }
            }
        } else {
            tracing::warn!("Storage repository not initialized");
            self.as_mut().set_error_message(QString::from("存储未初始化，请先调用 init_with_repository"));
        }

        self.as_mut().set_is_loading(false);
    }

    /// 设置筛选条件
    fn set_filter(mut self: Pin<&mut Self>, filter: QString) {
        self.as_mut().set_selected_filter(filter);
        *self.custom_start_time.borrow_mut() = None;
        *self.custom_end_time.borrow_mut() = None;
    }

    /// 设置自定义时间范围
    fn set_custom_time_range(mut self: Pin<&mut Self>, start_timestamp: i64, end_timestamp: i64) {
        if start_timestamp < 0 || end_timestamp < 0 {
            tracing::warn!("Invalid timestamps: start={}, end={}", start_timestamp, end_timestamp);
            return;
        }
        *self.custom_start_time.borrow_mut() =
            Some(UNIX_EPOCH + Duration::from_secs(start_timestamp as u64));
        *self.custom_end_time.borrow_mut() = Some(UNIX_EPOCH + Duration::from_secs(end_timestamp as u64));
        self.as_mut().set_selected_filter(QString::from("custom"));
    }

    /// 初始化（设置仓库引用）
    fn init_with_repository(mut self: Pin<&mut Self>) {
        let db_path = "crane_data.db";
        let storage_result = qt_threading_utils::runtime::global_runtime().block_on(async {
            SqliteStorageRepository::new(db_path).await
        });

        match storage_result {
            Ok(storage) => {
                *self.storage.borrow_mut() = Some(Arc::new(storage));
                tracing::info!("HistoryViewModel initialized with storage repository");
            }
            Err(e) => {
                tracing::error!("Failed to initialize storage repository: {}", e);
                self.as_mut().set_error_message(QString::from(&format!("初始化存储失败: {}", e)));
            }
        }
    }
}

// 辅助函数：将 ProcessedData 转换为 JSON
fn processed_data_to_json(data: &[ProcessedData]) -> QString {
    let time_labels: Vec<String> = data
        .iter()
        .map(|d| {
            let datetime: chrono::DateTime<chrono::Local> = chrono::DateTime::from(d.timestamp);
            datetime.format("%H:%M:%S").to_string()
        })
        .collect();

    let moment_data: Vec<f64> = data.iter().map(|d| d.moment_percentage).collect();
    let load_data: Vec<f64> = data.iter().map(|d| d.current_load).collect();
    let radius_data: Vec<f64> = data.iter().map(|d| d.working_radius).collect();
    let angle_data: Vec<f64> = data.iter().map(|d| d.boom_angle).collect();

    let json = json!({
        "timeLabels": time_labels,
        "momentData": moment_data,
        "loadData": load_data,
        "radiusData": radius_data,
        "angleData": angle_data
    });

    QString::from(&json.to_string())
}

// 辅助函数：将 AlarmRecord 转换为 JSON
fn alarm_records_to_json(records: &[AlarmRecord]) -> QString {
    use qt_rust_demo::models::AlarmType;

    let json_records: Vec<serde_json::Value> = records
        .iter()
        .map(|r| {
            let datetime: chrono::DateTime<chrono::Local> = chrono::DateTime::from(r.timestamp);
            let alarm_type_str = match r.alarm_type {
                AlarmType::Warning => "warning",
                AlarmType::Danger => "danger",
            };
            let title = if r.alarm_type == AlarmType::Danger {
                "危险报警"
            } else {
                "预警提示"
            };

            json!({
                "id": r.id.unwrap_or(0),
                "alarmType": alarm_type_str,
                "title": title,
                "message": r.description,
                "date": datetime.format("%Y/%m/%d").to_string(),
                "time": datetime.format("%H:%M:%S").to_string(),
                "momentValue": format!("{:.1}%", r.moment_percentage)
            })
        })
        .collect();

    QString::from(&json!(json_records).to_string())
}
