#[cxx_qt::bridge]
pub mod moment_curve_bridge {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        include!("cxx-qt-lib/qstringlist.h");
        type QString = cxx_qt_lib::QString;
        type QStringList = cxx_qt_lib::QStringList;
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QStringList, boom_length_list)]
        #[qproperty(i32, selected_boom_index)]
        #[qproperty(f64, current_boom_length)]
        #[qproperty(bool, data_loaded)]
        #[qproperty(QString, error_message)]
        type MomentCurveViewModel = super::MomentCurveViewModelRust;

        #[qinvokable]
        unsafe fn load_data(self: Pin<&mut MomentCurveViewModel>);

        #[qinvokable]
        unsafe fn get_curve_data_json(
            self: Pin<&mut MomentCurveViewModel>,
            boom_length: f64,
        ) -> QString;

        #[qinvokable]
        unsafe fn get_max_load_for_boom(
            self: Pin<&mut MomentCurveViewModel>,
            boom_length: f64,
        ) -> f64;

        #[qinvokable]
        unsafe fn get_max_radius_for_boom(
            self: Pin<&mut MomentCurveViewModel>,
            boom_length: f64,
        ) -> f64;

        #[qinvokable]
        unsafe fn get_data_point_count(
            self: Pin<&mut MomentCurveViewModel>,
            boom_length: f64,
        ) -> i32;

        #[qinvokable]
        unsafe fn get_average_load(self: Pin<&mut MomentCurveViewModel>, boom_length: f64) -> f64;

        #[qinvokable]
        unsafe fn get_load_range(self: Pin<&mut MomentCurveViewModel>, boom_length: f64)
            -> QString;

        #[qinvokable]
        unsafe fn select_boom_by_index(self: Pin<&mut MomentCurveViewModel>, index: i32);

        #[qinvokable]
        unsafe fn get_global_max_radius(self: Pin<&mut MomentCurveViewModel>) -> f64;

        #[qinvokable]
        unsafe fn get_global_max_load(self: Pin<&mut MomentCurveViewModel>) -> f64;
    }
}

use core::pin::Pin;
use cxx_qt_lib::{QString, QStringList};
use qt_rust_demo::config::load_table_manager::LoadTableManager;
use qt_rust_demo::models::rated_load_table::RatedLoadTable;
use std::cell::RefCell;
use std::collections::HashMap;

struct CurveDataPoint {
    radius: f64,
    load: f64,
}

pub struct MomentCurveViewModelRust {
    boom_length_list: QStringList,
    selected_boom_index: i32,
    current_boom_length: f64,
    data_loaded: bool,
    error_message: QString,
    config_path: String,
    curve_data_cache: RefCell<HashMap<i64, Vec<CurveDataPoint>>>,
    boom_lengths: RefCell<Vec<f64>>,
    global_max_radius: RefCell<f64>,
    global_max_load: RefCell<f64>,
}

impl Default for MomentCurveViewModelRust {
    fn default() -> Self {
        Self {
            boom_length_list: QStringList::default(),
            selected_boom_index: 0,
            current_boom_length: 0.0,
            data_loaded: false,
            error_message: QString::from(""),
            config_path: "config/rated_load_table.csv".to_string(),
            curve_data_cache: RefCell::new(HashMap::new()),
            boom_lengths: RefCell::new(Vec::new()),
            global_max_radius: RefCell::new(25.0),
            global_max_load: RefCell::new(60.0),
        }
    }
}

fn boom_to_key(boom_length: f64) -> i64 {
    (boom_length * 1000.0).round() as i64
}

impl moment_curve_bridge::MomentCurveViewModel {
    pub fn load_data(mut self: Pin<&mut Self>) {
        tracing::info!("Loading rated load table from: {}", self.config_path);

        let manager = LoadTableManager::new(&self.config_path);

        match manager.load() {
            Ok(table) => {
                self.process_loaded_data(table);
            }
            Err(e) => {
                let error_msg = format!("加载额定载荷表失败: {}", e);
                tracing::error!("{}", error_msg);
                self.as_mut().set_error_message(QString::from(&error_msg));
                self.as_mut().set_data_loaded(false);
            }
        }
    }

    fn process_loaded_data(mut self: Pin<&mut Self>, table: RatedLoadTable) {
        self.curve_data_cache.borrow_mut().clear();
        self.boom_lengths.borrow_mut().clear();

        let mut max_radius = 0.0_f64;
        let mut max_load = 0.0_f64;

        let boom_lengths = table.get_boom_lengths();
        tracing::info!(
            "Found {} boom lengths in rated load table",
            boom_lengths.len()
        );

        for boom_length in &boom_lengths {
            self.boom_lengths.borrow_mut().push(*boom_length);

            if let Some(entries) = table.get_entries_for_boom(*boom_length) {
                let mut points = Vec::new();

                for entry in entries {
                    points.push(CurveDataPoint {
                        radius: entry.working_radius,
                        load: entry.rated_load,
                    });

                    if entry.working_radius > max_radius {
                        max_radius = entry.working_radius;
                    }
                    if entry.rated_load > max_load {
                        max_load = entry.rated_load;
                    }
                }

                let key = boom_to_key(*boom_length);
                self.curve_data_cache.borrow_mut().insert(key, points);

                tracing::debug!(
                    "Loaded {} points for boom length {}m",
                    entries.len(),
                    boom_length
                );
            }
        }

        *self.global_max_radius.borrow_mut() = (max_radius / 5.0).ceil() * 5.0;
        *self.global_max_load.borrow_mut() = (max_load / 10.0).ceil() * 10.0;

        if *self.global_max_radius.borrow() < 25.0 {
            *self.global_max_radius.borrow_mut() = 25.0;
        }
        if *self.global_max_load.borrow() < 60.0 {
            *self.global_max_load.borrow_mut() = 60.0;
        }

        let mut string_list = QStringList::default();
        for boom in self.boom_lengths.borrow().iter() {
            string_list.append(QString::from(&format!("{:.1}", boom)));
        }
        self.as_mut().set_boom_length_list(string_list);

        if !self.boom_lengths.borrow().is_empty() {
            let first_boom = self.boom_lengths.borrow()[0];
            self.as_mut().set_selected_boom_index(0);
            self.as_mut().set_current_boom_length(first_boom);
        }

        self.as_mut().set_data_loaded(true);
        self.as_mut().set_error_message(QString::from(""));

        tracing::info!(
            "Rated load table loaded successfully: {} boom lengths, max radius: {}, max load: {}",
            self.boom_lengths.borrow().len(),
            *self.global_max_radius.borrow(),
            *self.global_max_load.borrow()
        );
    }

    pub fn get_curve_data_json(self: Pin<&mut Self>, boom_length: f64) -> QString {
        let key = boom_to_key(boom_length);

        if let Some(points) = self.curve_data_cache.borrow().get(&key) {
            let mut json_parts = Vec::new();
            for point in points {
                json_parts.push(format!(
                    "{{\"x\":{:.2},\"y\":{:.2}}}",
                    point.radius, point.load
                ));
            }
            let json = format!("[{}]", json_parts.join(","));
            QString::from(&json)
        } else {
            QString::from("[]")
        }
    }

    pub fn get_max_load_for_boom(self: Pin<&mut Self>, boom_length: f64) -> f64 {
        let key = boom_to_key(boom_length);

        if let Some(points) = self.curve_data_cache.borrow().get(&key) {
            points.iter().map(|p| p.load).fold(0.0, f64::max)
        } else {
            0.0
        }
    }

    pub fn get_max_radius_for_boom(self: Pin<&mut Self>, boom_length: f64) -> f64 {
        let key = boom_to_key(boom_length);

        if let Some(points) = self.curve_data_cache.borrow().get(&key) {
            points.iter().map(|p| p.radius).fold(0.0, f64::max)
        } else {
            0.0
        }
    }

    pub fn get_data_point_count(self: Pin<&mut Self>, boom_length: f64) -> i32 {
        let key = boom_to_key(boom_length);

        if let Some(points) = self.curve_data_cache.borrow().get(&key) {
            points.len() as i32
        } else {
            0
        }
    }

    pub fn get_average_load(self: Pin<&mut Self>, boom_length: f64) -> f64 {
        let key = boom_to_key(boom_length);

        if let Some(points) = self.curve_data_cache.borrow().get(&key) {
            if points.is_empty() {
                return 0.0;
            }
            let sum: f64 = points.iter().map(|p| p.load).sum();
            sum / points.len() as f64
        } else {
            0.0
        }
    }

    pub fn get_load_range(self: Pin<&mut Self>, boom_length: f64) -> QString {
        let key = boom_to_key(boom_length);

        if let Some(points) = self.curve_data_cache.borrow().get(&key) {
            if points.is_empty() {
                return QString::from("0-0");
            }
            let min_load = points.iter().map(|p| p.load).fold(f64::INFINITY, f64::min);
            let max_load = points.iter().map(|p| p.load).fold(0.0_f64, f64::max);
            QString::from(&format!("{:.1}-{:.1}", min_load, max_load))
        } else {
            QString::from("0-0")
        }
    }

    pub fn select_boom_by_index(mut self: Pin<&mut Self>, index: i32) {
        let idx = index as usize;
        if idx < self.boom_lengths.borrow().len() {
            let boom_length = self.boom_lengths.borrow()[idx];
            self.as_mut().set_selected_boom_index(index);
            self.as_mut().set_current_boom_length(boom_length);
            tracing::debug!("Selected boom index: {}, length: {}m", index, boom_length);
        } else {
            tracing::warn!(
                "Invalid boom index: {}, max: {}",
                index,
                self.boom_lengths.borrow().len()
            );
        }
    }

    pub fn get_global_max_radius(self: Pin<&mut Self>) -> f64 {
        *self.global_max_radius.borrow()
    }

    pub fn get_global_max_load(self: Pin<&mut Self>) -> f64 {
        *self.global_max_load.borrow()
    }
}
