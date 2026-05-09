# i18n Multi-Language Support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add multi-language support to qt4rust with runtime language switching, decoupled Fluent backend architecture, and Chinese + English translations.

**Architecture:** Trait-based abstraction with pluggable backends. Fluent-backed implementation for rich i18n features (plurals, variables), exposed to QML via cxx-qt bridge.

**Tech Stack:** 
- Rust + cxx-qt 0.8
- Fluent `.ftl` format
- QML (Qt Quick)
- Cargo workspace

---

## Phase 1: Core Infrastructure

### Task 1: Add Fluent Dependencies

**Files:**
- Modify: `crates/qt-app/Cargo.toml`

- [ ] **Step 1: Add fluent dependencies to Cargo.toml**

```toml
[dependencies]
# i18n support
fluent = "0.16"
fluent-bundle = "0.16"
fluent-syntax = "0.12"
rust-embed = "8"
```

- [ ] **Step 2: Run cargo check to verify dependencies**

Run: `cd /mnt/sdb1/qt4rust/.worktrees/i18n && cargo check -p qt-app`
Expected: No errors, fluent crates added

- [ ] **Step 3: Commit**

```bash
cd /mnt/sdb1/qt4rust/.worktrees/i18n
git add crates/qt-app/Cargo.toml
git commit -m "feat(i18n): add fluent dependencies"
```

---

### Task 2: Create i18n Module Structure

**Files:**
- Create: `crates/qt-app/src/i18n/mod.rs`
- Create: `crates/qt-app/src/i18n/traits.rs`
- Create: `crates/qt-app/src/i18n/fluent_backend.rs`
- Create: `crates/qt-app/src/i18n/locale_manager.rs`
- Create: `crates/qt-app/src/i18n/bridge.rs`

- [ ] **Step 1: Create i18n directory**

```bash
mkdir -p /mnt/sdb1/qt4rust/.worktrees/i18n/crates/qt-app/src/i18n
```

- [ ] **Step 2: Write traits.rs with Translate trait**

```rust
use std::sync::Arc;

/// Core trait for translation providers
pub trait Translate: Send + Sync {
    /// Get translation for key
    fn t(&self, key: &str) -> String;
    
    /// Get translation with variables
    fn t_with_args(&self, key: &str, args: &[(&str, &str)]) -> String;
    
    /// Get current locale code
    fn current_locale(&self) -> &str;
    
    /// Get list of available locales
    fn available_locales(&self) -> Vec<String>;
    
    /// Switch to different locale
    fn set_locale(&self, locale: &str) -> Result<(), String>;
}

/// Thread-safe wrapper around Translate
pub type TranslationProvider = Arc<dyn Translate>;
```

- [ ] **Step 3: Write locale_manager.rs**

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

const DEFAULT_LOCALE: &str = "zh-CN";
const CONFIG_FILENAME: &str = "app_config.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub app: AppSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub language: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app: AppSettings {
                language: DEFAULT_LOCALE.to_string(),
            },
        }
    }
}

/// Manages language preference and persistence
pub struct LocaleManager {
    config: RwLock<AppConfig>,
    config_path: PathBuf,
}

impl LocaleManager {
    pub fn new(config_dir: PathBuf) -> Self {
        let config_path = config_dir.join(CONFIG_FILENAME);
        let config = Self::load_config(&config_path).unwrap_or_default();
        
        Self {
            config: RwLock::new(config),
            config_path,
        }
    }
    
    fn load_config(path: &PathBuf) -> Option<AppConfig> {
        let content = std::fs::read_to_string(path).ok()?;
        toml::from_str(&content).ok()
    }
    
    pub fn get_locale(&self) -> String {
        self.config.read().unwrap().app.language.clone()
    }
    
    pub fn set_locale(&self, locale: &str) -> Result<(), String> {
        let mut config = self.config.write().unwrap();
        config.app.language = locale.to_string();
        
        // Persist to file
        let content = toml::to_string_pretty(&*config)
            .map_err(|e| e.to_string())?;
        std::fs::write(&self.config_path, content)
            .map_err(|e| e.to_string())?;
        
        Ok(())
    }
}
```

- [ ] **Step 4: Write fluent_backend.rs**

```rust
use crate::i18n::traits::{Translate, TranslationProvider};
use crate::i18n::locale_manager::LocaleManager;
use fluent::{FluentBundle, FluentResource, FluentValue};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};

/// Fluent-based translation backend
pub struct FluentBackend {
    bundles: RwLock<HashMap<String, FluentBundle<FluentResource>>>,
    current_locale: RwLock<String>,
    locale_manager: Arc<LocaleManager>,
}

impl FluentBackend {
    pub fn new(locale_manager: Arc<LocaleManager>) -> Self {
        Self {
            bundles: RwLock::new(HashMap::new()),
            current_locale: RwLock::new(locale_manager.get_locale()),
            locale_manager,
        }
    }
    
    /// Load translation file for locale
    pub fn load_locale(&self, locale: &str, translations_dir: &Path) -> Result<(), String> {
        let file_path = translations_dir.join(format!("{}.ftl", locale));
        
        let source = std::fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to load {}: {}", locale, e))?;
        
        let resource = FluentResource::try_new(source)
            .map_err(|(_, errors)| {
                let messages: Vec<_> = errors.iter().map(|e| e.to_string()).collect();
                format!("Parse errors: {}", messages.join(", "))
            })?;
        
        let mut bundle = FluentBundle::new(vec![locale.parse().unwrap()]);
        bundle.add_resource(resource)
            .map_err(|e| format!("Bundle error: {}", e))?;
        
        self.bundles.write().unwrap().insert(locale.to_string(), bundle);
        
        Ok(())
    }
    
    /// Get FluentBundle for current locale
    fn get_bundle(&self) -> Option<fluent::FluentBundle<fluent::FluentResource>> {
        let locale = self.current_locale.read().unwrap().clone();
        self.bundles.read().unwrap().get(&locale).cloned()
    }
}

impl Translate for FluentBackend {
    fn t(&self, key: &str) -> String {
        self.t_with_args(key, &[])
    }
    
    fn t_with_args(&self, key: &str, args: &[(&str, &str)]) -> String {
        let bundle = self.get_bundle();
        
        let Some(bundle) = bundle else {
            return key.to_string();
        };
        
        let message = bundle.get_message(key);
        let Some(message) = message else {
            log::warn!("Translation key not found: {}", key);
            return key.to_string();
        };
        
        let pattern = message.value()?;
        let mut errors = vec![];
        let result = bundle.format_pattern(pattern, Some(args), &mut errors);
        
        if !errors.is_empty() {
            log::warn!("Translation errors for key {}: {:?}", key, errors);
        }
        
        result.into_owned()
    }
    
    fn current_locale(&self) -> &str {
        self.current_locale.read().unwrap().as_str()
    }
    
    fn available_locales(&self) -> Vec<String> {
        self.bundles.read().unwrap().keys().cloned().collect()
    }
    
    fn set_locale(&self, locale: &str) -> Result<(), String> {
        // Check if locale is loaded
        if !self.bundles.read().unwrap().contains_key(locale) {
            return Err(format!("Locale not loaded: {}", locale));
        }
        
        // Update current locale
        *self.current_locale.write().unwrap() = locale.to_string();
        
        // Persist preference
        self.locale_manager.set_locale(locale)?;
        
        Ok(())
    }
}
```

- [ ] **Step 5: Write mod.rs with factory function**

```rust
pub mod traits;
pub mod locale_manager;
pub mod fluent_backend;
pub mod bridge;

pub use traits::{Translate, TranslationProvider};
pub use locale_manager::LocaleManager;
pub use fluent_backend::FluentBackend;
pub use bridge::TranslationBridge;

use std::path::PathBuf;
use std::sync::Arc;

/// Create translation provider with default configuration
pub fn create_translation_provider(config_dir: PathBuf, translations_dir: PathBuf) -> Result<TranslationProvider, String> {
    let locale_manager = Arc::new(LocaleManager::new(config_dir));
    let backend = Arc::new(FluentBackend::new(locale_manager.clone()));
    
    // Load default locale
    let default_locale = locale_manager.get_locale();
    backend.load_locale(&default_locale, &translations_dir)?;
    
    // Also preload fallback locale if different
    if default_locale != "zh-CN" {
        let _ = backend.load_locale("zh-CN", &translations_dir);
    }
    
    Ok(backend)
}
```

- [ ] **Step 6: Write bridge.rs - cxx-qt exposure**

```rust
use crate::i18n::traits::TranslationProvider;
use cxx_qt_lib::QString;
use std::sync::Arc;

/// Bridge to expose translation to QML
#[cxx_qt::qobject]
pub struct TranslationBridge {
    provider: TranslationProvider,
    #[qproperty]
    current_locale: QString,
}

#[cxx_qt::qobject_impl]
impl TranslationBridge {
    /// Create new bridge with translation provider
    pub fn new(provider: TranslationProvider) -> Self {
        let locale = provider.current_locale().to_string();
        Self {
            provider,
            current_locale: QString::from(&locale),
        }
    }
    
    /// Translate key - exposed to QML
    #[qinvokable]
    pub fn translate(&self, key: &str) -> QString {
        let result = self.provider.t(key);
        QString::from(&result)
    }
    
    /// Translate with arguments - exposed to QML
    #[qinvokable]
    pub fn translate_with_args(&self, key: &str, args: &str) -> QString {
        // Parse args from JSON: {"key1": "value1", "key2": "value2"}
        let parsed: serde_json::Result<std::collections::HashMap<String, String>> = 
            serde_json::from_str(args);
        
        let args_vec: Vec<(&str, &str)> = match parsed {
            Ok(map) => map.iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect(),
            Err(_) => vec![],
        };
        
        let result = self.provider.t_with_args(key, &args_vec);
        QString::from(&result)
    }
    
    /// Get available locales
    #[qinvokable]
    pub fn available_locales(&self) -> Vec<QString> {
        self.provider.available_locales()
            .into_iter()
            .map(QString::from)
            .collect()
    }
    
    /// Switch locale
    #[qinvokable]
    pub fn set_locale(&mut self, locale: &str) -> bool {
        match self.provider.set_locale(locale) {
            Ok(()) => {
                self.current_locale = QString::from(locale);
                true
            }
            Err(e) => {
                log::error!("Failed to set locale: {}", e);
                false
            }
        }
    }
    
    /// Get current locale
    #[qinvokable]
    pub fn get_locale(&self) -> QString {
        QString::from(self.provider.current_locale())
    }
}
```

- [ ] **Step 7: Verify compilation**

Run: `cd /mnt/sdb1/qt4rust/.worktrees/i18n && cargo check -p qt-app`
Expected: No errors

- [ ] **Step 8: Commit**

```bash
cd /mnt/sdb1/qt4rust/.worktrees/i18n
git add crates/qt-app/src/i18n/
git commit -m "feat(i18n): create core i18n module with trait abstraction"
```

---

### Task 3: Create Translation Files

**Files:**
- Create: `qml/assets/translations/zh-CN.ftl`
- Create: `qml/assets/translations/en-US.ftl`

- [ ] **Step 1: Create translations directory**

```bash
mkdir -p /mnt/sdb1/qt4rust/.worktrees/i18n/qml/assets/translations
```

- [ ] **Step 2: Write zh-CN.ftl (Chinese translations)**

```ftl
## Main / Navigation
main.title = 起重机力矩监测系统
main.subtitle = Crane Monitoring System

nav.monitoring = 主界面
nav.charts = 数据曲线
nav.alarms = 报警记录
nav.settings = 设置

## Monitoring View
monitoring.title = 实时监控
monitoring.currentLoad = 当前载荷
monitoring.workingRadius = 工作半径
monitoring.boomAngle = 吊臂角度
monitoring.boomLength = 臂长
monitoring.ratedLoad = 额定载荷
monitoring.unit.ton = 吨
monitoring.unit.meter = 米
monitoring.unit.degree = 度
monitoring.sensorDisconnected = 传感器连接断开

## Danger Card
danger.title.warning = 力��预警
danger.title.danger = 危险报警
danger.title.angleAlarm = 角度报警
danger.message.warning = 力矩接近上限，请注意控制载荷
danger.message.danger = 力矩严重超限！立即停止作业
danger.message.angleAlarm = 吊臂角度超限！请立即调整角度
danger.craneStatus = 起重机臂架状态

## Alarm Records
alarm.title = 报警记录
alarm.subtitle = 系统报警历史与统计分析
alarm.totalCount = 总报警次数
alarm.warningCount = 预警次数
alarm.dangerCount = 危险次数
alarm.levels = 报警级别说明
alarm.level.normal = 正常
alarm.level.normalDesc = 力矩 0-75%
alarm.level.warning = 预警
alarm.level.warningDesc = 力矩 75-90%
alarm.level.danger = 危险
alarm.level.dangerDesc = 力矩 ≥90%

## Chart View
chart.title = 数据曲线分析
chart.subtitle = 实时监测数据变化趋势
chart.refresh = 刷新
chart.loadTrend = 载荷变化曲线
chart.momentTrend = 力矩百分比趋势
chart.multiParam = 多参数对比分析

## Settings View
settings.title = 设置
settings.systemStatus = 系统状态
settings.calibration = 参数校准
settings.momentCurve = 力矩曲线
settings.about = 关于系统
settings.language = 语言
settings.language.zhCN = 简体中文
settings.language.enUS = English
settings.reload = 重新加载配置
settings.save = 保存设置

## Calibration
calibration.loadSensor = 载荷传感器
calibration.angleSensor = 角度传感器
calibration.radiusSensor = 长度传感器
calibration.alarmThreshold = 报警阈值
calibration.restoreDefault = 恢复默认
calibration.adValue = AD值
calibration.physicalValue = 重量（吨）
calibration.angleValue = 角度（°）
calibration.radiusValue = 长度（m）
calibration.calibrating = 实时采集中
calibration.multiplier = 标定倍率
calibration.multiplierDesc = 标定倍率用于调整传感器灵敏度

## About System
about.title = 汽车吊力矩监测系统
about.subtitle = Crane Moment Monitoring System
about.version = 系统版本
about.releaseDate = 发布日期
about.firmware = 固件版本
about.hardware = 硬件版本
about.features = 系统特性
about.feature.realtime = 实时安全监控
about.feature.realtimeDesc = 24/7不间断监测起重机力矩状态，确保作业安全
about.feature.warning = 三级预警系统
about.feature.warningDesc = 正常、预警、危险三级报警，提前预防安全事故
about.feature.sensor = 高精度传感器
about.feature.sensorDesc = 采用工业级传感器，精度±0.5%，响应时间<50ms
about.techSpecs = 技术规格
about.certifications = 认证与标准
about.techSupport = 技术支持
about.hotline = 服务热线
about.email = 技术邮箱
about.address = 公司地址
about.copyright = © 2025 汽车吊力矩监测系统 版权所有

## Moment Card
moment.percentage = 力矩百分比
moment.warning = 预警
moment.danger = 危险

## Time Card
timeCard.time = 时间

## Boom Length Card
boomLength.title = 臂长
boomLength.unit = 米
boomLength.description = 吊臂总长度

## Moment Curve
momentCurve.title = 力矩曲线图说明
momentCurve.ratedCurve = 额定载荷曲线
momentCurve.importCurve = 导入曲线
momentCurve.boomLength = 臂长
momentCurve.currentLength = 当前臂长
momentCurve.maxLoad = 最大载荷
momentCurve.maxRadius = 最大幅度
momentCurve.dataPoints = 数据点数

## Alarm Threshold
alarmThreshold.momentTitle = 力矩报警阈值
alarmThreshold.momentDesc = 设置力矩百分比报警阈值
alarmThreshold.warningPercent = 预警阈值（%）
alarmThreshold.dangerPercent = 危险阈值（%）
alarmThreshold.angleTitle = 角度报警阈值
alarmThreshold.angleDesc = 设置吊臂角度报警范围
alarmThreshold.angleLower = 角度下限（度）
alarmThreshold.angleUpper = 角度上限（度）
alarmThreshold.mainHook = 主钩勾头开关报警
alarmThreshold.mainHookDesc = 设置主钩勾头开关状态报警触发条件
alarmThreshold.subHook = 副钩勾头开关报警
alarmThreshold.subHookDesc = 设置副钩勾头开关状态报警触发条件
alarmThreshold.mode = 报警模式
alarmThreshold.modeNO = 常开
alarmThreshold.modeNC = 常闭

## Dialogs
dialog.cancel = 取消
dialog.confirm = 确定
dialog.selectMultiplier = 选择自定义倍率
dialog.selectMultiplierDesc = 请选择标定倍率（0.5 - 10.0）
dialog.timeRange = 时间筛选设置
dialog.timeRangeDesc = 选择自定义时间范围以筛选曲线数据
dialog.startTime = 开始时间
dialog.endTime = 结束时间（可选）
dialog.resetFilter = 重置筛选
dialog.applyFilter = 应用筛选
dialog.importRatedLoad = 导入额定载荷表
dialog.importRatedLoadDesc = 请输入 CSV 文件路径
dialog.importHint = 提示：可以使用相对路径或绝对路径

## Filter Bar
filter.history = 历史记录:
filter.all = 全部
filter.today = 今天
filter.last7days = 最近7天
filter.last30days = 最近30天
filter.custom = 自定义

## Time Range
timeRange.1hour = 1小时
timeRange.2hours = 2小时
timeRange.5hours = 5小时
timeRange.custom = 自定义

## System Status
systemStatus.sensorConnection = 传感器连接状态
systemStatus.systemInfo = 系统信息
systemStatus.network = 网络与通信

## Common
common.version = v1.0.0
```

- [ ] **Step 3: Write en-US.ftl (English translations)**

```ftl
## Main / Navigation
main.title = Crane Moment Monitoring System
main.subtitle = Crane Monitoring System

nav.monitoring = Monitoring
nav.charts = Data Charts
nav.alarms = Alarm Records
nav.settings = Settings

## Monitoring View
monitoring.title = Real-time Monitoring
monitoring.currentLoad = Current Load
monitoring.workingRadius = Working Radius
monitoring.boomAngle = Boom Angle
monitoring.boomLength = Boom Length
monitoring.ratedLoad = Rated Load
monitoring.unit.ton = ton
monitoring.unit.meter = m
monitoring.unit.degree = °
monitoring.sensorDisconnected = Sensor Disconnected

## Danger Card
danger.title.warning = Moment Warning
danger.title.danger = Danger Alert
danger.title.angleAlarm = Angle Alarm
danger.message.warning = Moment approaching limit, please control load
danger.message.danger = Moment severely overloaded! Stop operation immediately
danger.message.angleAlarm = Boom angle exceeds limit! Adjust immediately
danger.craneStatus = Crane Boom Status

## Alarm Records
alarm.title = Alarm Records
alarm.subtitle = System Alarm History & Statistics
alarm.totalCount = Total Alarms
alarm.warningCount = Warning Count
alarm.dangerCount = Danger Count
alarm.levels = Alarm Level Description
alarm.level.normal = Normal
alarm.level.normalDesc = Moment 0-75%
alarm.level.warning = Warning
alarm.level.warningDesc = Moment 75-90%
alarm.level.danger = Danger
alarm.level.dangerDesc = Moment ≥90%

## Chart View
chart.title = Data Curve Analysis
chart.subtitle = Real-time Monitoring Data Trends
chart.refresh = Refresh
chart.loadTrend = Load Trend Curve
chart.momentTrend = Moment Percentage Trend
chart.multiParam = Multi-Parameter Comparison

## Settings View
settings.title = Settings
settings.systemStatus = System Status
settings.calibration = Calibration
settings.momentCurve = Moment Curve
settings.about = About
settings.language = Language
settings.language.zhCN = 简体中文
settings.language.enUS = English
settings.reload = Reload Config
settings.save = Save Settings

## Calibration
calibration.loadSensor = Load Sensor
calibration.angleSensor = Angle Sensor
calibration.radiusSensor = Radius Sensor
calibration.alarmThreshold = Alarm Threshold
calibration.restoreDefault = Restore Default
calibration.adValue = AD Value
calibration.physicalValue = Weight (ton)
calibration.angleValue = Angle (°)
calibration.radiusValue = Length (m)
calibration.calibrating = Calibrating...
calibration.multiplier = Multiplier
calibration.multiplierDesc = Multiplier adjusts sensor sensitivity for different ranges

## About System
about.title = Crane Moment Monitoring System
about.subtitle = Crane Moment Monitoring System
about.version = System Version
about.releaseDate = Release Date
about.firmware = Firmware Version
about.hardware = Hardware Version
about.features = System Features
about.feature.realtime = Real-time Safety Monitoring
about.feature.realtimeDesc = 24/7 continuous monitoring of crane moment status
about.feature.warning = Three-level Warning System
about.feature.warningDesc = Normal, Warning, Danger levels for accident prevention
about.feature.sensor = High-precision Sensors
about.feature.sensorDesc = Industrial-grade sensors, ±0.5% accuracy, <50ms response
about.techSpecs = Technical Specifications
about.certifications = Certifications
about.techSupport = Technical Support
about.hotline = Service Hotline
about.email = Technical Email
about.address = Company Address
about.copyright = © 2025 Crane Moment Monitoring System. All rights reserved.

## Moment Card
moment.percentage = Moment Percentage
moment.warning = Warning
moment.danger = Danger

## Time Card
timeCard.time = Time

## Boom Length Card
boomLength.title = Boom Length
boomLength.unit = m
boomLength.description = Total Boom Length

## Moment Curve
momentCurve.title = Moment Curve Description
momentCurve.ratedCurve = Rated Load Curve
momentCurve.importCurve = Import Curve
momentCurve.boomLength = Boom Length
momentCurve.currentLength = Current Length
momentCurve.maxLoad = Max Load
momentCurve.maxRadius = Max Radius
momentCurve.dataPoints = Data Points

## Alarm Threshold
alarmThreshold.momentTitle = Moment Alarm Threshold
alarmThreshold.momentDesc = Set moment percentage alarm threshold
alarmThreshold.warningPercent = Warning Threshold (%)
alarmThreshold.dangerPercent = Danger Threshold (%)
alarmThreshold.angleTitle = Angle Alarm Threshold
alarmThreshold.angleDesc = Set boom angle alarm range
alarmThreshold.angleLower = Lower Angle Limit (°)
alarmThreshold.angleUpper = Upper Angle Limit (°)
alarmThreshold.mainHook = Main Hook Switch Alarm
alarmThreshold.mainHookDesc = Set main hook switch alarm trigger condition
alarmThreshold.subHook = Sub Hook Switch Alarm
alarmThreshold.subHookDesc = Set sub hook switch alarm trigger condition
alarmThreshold.mode = Alarm Mode
alarmThreshold.modeNO = Normally Open
alarmThreshold.modeNC = Normally Closed

## Dialogs
dialog.cancel = Cancel
dialog.confirm = Confirm
dialog.selectMultiplier = Select Multiplier
dialog.selectMultiplierDesc = Please select multiplier (0.5 - 10.0)
dialog.timeRange = Time Range Filter
dialog.timeRangeDesc = Select custom time range for curve data
dialog.startTime = Start Time
dialog.endTime = End Time (Optional)
dialog.resetFilter = Reset Filter
dialog.applyFilter = Apply Filter
dialog.importRatedLoad = Import Rated Load Table
dialog.importRatedLoadDesc = Please enter CSV file path
dialog.importHint = Tip: You can use relative or absolute path

## Filter Bar
filter.history = History:
filter.all = All
filter.today = Today
filter.last7days = Last 7 Days
filter.last30days = Last 30 Days
filter.custom = Custom

## Time Range
timeRange.1hour = 1 Hour
timeRange.2hours = 2 Hours
timeRange.5hours = 5 Hours
timeRange.custom = Custom

## System Status
systemStatus.sensorConnection = Sensor Connection Status
systemStatus.systemInfo = System Information
systemStatus.network = Network & Communication

## Common
common.version = v1.0.0
```

- [ ] **Step 4: Commit**

```bash
cd /mnt/sdb1/qt4rust/.worktrees/i18n
git add qml/assets/translations/
git commit -m "feat(i18n): add translation files for zh-CN and en-US"
```

---

## Phase 2: QML Integration

### Task 4: Integrate i18n into Application

**Files:**
- Modify: `crates/qt-app/src/application.rs`

- [ ] **Step 1: Modify application.rs to initialize i18n**

Find the section where QQmlApplicationEngine is created and add:

```rust
// In application.rs, add after imports:
use crate::i18n::{create_translation_provider, TranslationBridge};

// In the application initialization, after engine is created:
// Load translations
let config_dir = std::path::PathBuf::from("config");
let translations_dir = std::path::PathBuf::from("qml/assets/translations");

let translation_provider = create_translation_provider(
    config_dir,
    translations_dir
).map_err(|e| format!("Failed to initialize translations: {}", e))?;

// Register QML type
let bridge = TranslationBridge::new(translation_provider);
engine.qc().register_unique_type::<TranslationBridge>();
engine.executor().queue(bridge.clone()).ok();
```

- [ ] **Step 2: Verify compilation**

Run: `cd /mnt/sdb1/qt4rust/.worktrees/i18n && cargo check -p qt-app`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
cd /mnt/sdb1/qt4rust/.worktrees/i18n
git add crates/qt-app/src/application.rs
git commit -m "feat(i18n): initialize translation provider in application"
```

---

### Task 5: Create QML Translation Helper

**Files:**
- Create: `qml/i18n/Tr.qml`

- [ ] **Step 1: Create Tr.qml singleton**

```qml
import QtQuick

/// Translation helper singleton for QML
/// Usage: Tr.t("key") or Tr.t("key", {"variable": "value"})
QtObject {
    id: root
    
    // Reference to the Rust TranslationBridge
    property var bridge: null
    
    /// Main translation function
    function t(key) {
        if (!bridge) {
            console.warn("TranslationBridge not initialized");
            return key;
        }
        return bridge.translate(key);
    }
    
    /// Translation with variables
    /// args: {"variable": "value"}
    function tWithArgs(key, args) {
        if (!bridge) {
            console.warn("TranslationBridge not initialized");
            return key;
        }
        var argsJson = JSON.stringify(args);
        return bridge.translateWithArgs(key, argsJson);
    }
    
    /// Get available locales
    function availableLocales() {
        if (!bridge) return [];
        return bridge.availableLocales();
    }
    
    /// Switch locale
    function setLocale(locale) {
        if (!bridge) {
            console.warn("TranslationBridge not initialized");
            return false;
        }
        return bridge.setLocale(locale);
    }
    
    /// Get current locale
    function getLocale() {
        if (!bridge) return "zh-CN";
        return bridge.getLocale();
    }
}
```

- [ ] **Step 2: Register Tr in main.qml**

Add to main.qml after imports:

```qml
import "i18n"

// At the Window level, add:
Tr {
    id: trHelper
    // Bridge will be set from Rust
}
```

- [ ] **Step 3: Expose Tr to QML context**

In application.rs, after engine is created:

```rust
// Register Tr singleton
engine.root_context().set_context_property("Tr", &tr_bridge);
```

- [ ] **Step 4: Commit**

```bash
cd /mnt/sdb1/qt4rust/.worktrees/i18n
git add qml/i18n/Tr.qml
git commit -m "feat(i18n): create QML translation helper"
```

---

### Task 6: Add Language Selector to Settings

**Files:**
- Modify: `qml/views/SettingsView.qml`

- [ ] **Step 1: Add language selector UI**

Find the settings list model and add language option:

```qml
// In SettingsView.qml, find the settingsModel list:
ListModel {
    id: settingsModel
    ListElement {
        text: "系统状态"
        icon: "icon-system-status.svg"
    },
    // Add this:
    ListElement {
        text: "语言"
        icon: "icon-language.svg"
    },
    // ... rest
}
```

- [ ] **Step 2: Create language selection UI**

Create new file: `qml/views/SettingsView/LanguageView.qml`

```qml
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../styles"

Item {
    Column {
        anchors.fill: parent
        anchors.margins: Theme.spacingLarge
        spacing: Theme.spacingLarge
        
        // Title
        Text {
            text: Tr.t("settings.language")
            font.pixelSize: Theme.fontSizeXLarge
            font.family: Theme.fontFamilyDefault
            color: Theme.textPrimary
        }
        
        // Language options
        Rectangle {
            width: parent.width
            height: 200
            color: Theme.darkSurface
            radius: Theme.radiusMedium
            
            Column {
                anchors.fill: parent
                anchors.margins: Theme.spacingMedium
                spacing: Theme.spacingMedium
                
                // Chinese option
                Button {
                    width: parent.width
                    height: 60
                    text: "简体中文"
                    onClicked: {
                        Tr.setLocale("zh-CN")
                        languageSelector.currentIndex = 0
                    }
                }
                
                // English option
                Button {
                    width: parent.width
                    height: 60
                    text: "English"
                    onClicked: {
                        Tr.setLocale("en-US")
                        languageSelector.currentIndex = 1
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 3: Integrate into SettingsView**

Add case in the currentIndexChanged handler:

```qml
// In SettingsView.qml, add to the switch/case:
case 3: // Language (new tab)
    loader.source = "LanguageView.qml"
    break
```

- [ ] **Step 4: Commit**

```bash
cd /mnt/sdb1/qt4rust/.worktrees/i18n
git add qml/views/SettingsView/
git commit -m "feat(i18n): add language selector to settings"
```

---

## Phase 3: String Externalization

### Task 7: Migrate MonitoringView Strings

**Files:**
- Modify: `qml/views/MonitoringView.qml`

- [ ] **Step 1: Replace hardcoded strings with Tr.t() calls**

In MonitoringView.qml, replace:
```qml
text: "当前载荷" 
```
with:
```qml
text: Tr.t("monitoring.currentLoad")
```

List of strings to replace:
- "传感器连接断开" → `monitoring.sensorDisconnected`
- "起重机臂架状态" → `danger.craneStatus`
- "当前载荷" → `monitoring.currentLoad`
- "工作半径" → `monitoring.workingRadius`
- "吊臂角度" → `monitoring.boomAngle`
- "臂长" → `monitoring.boomLength`
- "额定载荷" → `monitoring.ratedLoad`
- "吨" → `monitoring.unit.ton`
- "米" → `monitoring.unit.meter`
- "度" → `monitoring.unit.degree`
- Danger card titles and messages
- Data card labels

- [ ] **Step 2: Test the changes**

Run: `cd /mnt/sdb1/qt4rust/.worktrees/i18n && cargo run -p qt-app`
Expected: App starts, strings display in Chinese by default

- [ ] **Step 3: Commit**

```bash
cd /mnt/sdb1/qt4rust/.worktrees/i18n
git add qml/views/MonitoringView.qml
git commit -m "feat(i18n): migrate MonitoringView strings"
```

---

### Task 8: Migrate SettingsView Strings

**Files:**
- Modify: `qml/views/SettingsView/*.qml`

- [ ] **Step 1: Migrate all SettingsView strings**

Systematically replace hardcoded strings with Tr.t() calls in:
- SettingsView.qml (tab names)
- CalibrationView.qml (labels, buttons)
- AboutSystemView.qml (all text)
- MomentCurveView.qml (labels)
- SystemStatusView.qml (labels)
- CalibrationContents/*.qml (form labels)

- [ ] **Step 2: Commit**

```bash
cd /mnt/sdb1/qt4rust/.worktrees/i18n
git add qml/views/SettingsView/
git commit -m "feat(i18n): migrate SettingsView strings"
```

---

### Task 9: Migrate Remaining Views

**Files:**
- Modify: `qml/views/ChartView.qml`, `AlarmRecordView.qml`, `HomeView.qml`
- Modify: `qml/components/controls/*.qml`
- Modify: `qml/components/layouts/*.qml`
- Modify: `qml/components/dialogs/*.qml`

- [ ] **Step 1: Migrate ChartView strings**

- [ ] **Step 2: Migrate AlarmRecordView strings**

- [ ] **Step 3: Migrate Navigation and Header strings**

- [ ] **Step 4: Migrate component strings (cards, dialogs)**

- [ ] **Step 5: Commit**

```bash
cd /mnt/sdb1/qt4rust/.worktrees/i18n
git add qml/
git commit -m "feat(i18n): migrate remaining view strings"
```

---

## Phase 4: Testing & Verification

### Task 10: Integration Testing

- [ ] **Step 1: Test language switching**

1. Run the app
2. Go to Settings → Language
3. Switch to English
4. Verify all visible strings change to English
5. Switch back to Chinese
6. Verify all strings change back

- [ ] **Step 2: Test persistence**

1. Change language to English
2. Close the app
3. Reopen the app
4. Verify language is still English

- [ ] **Step 3: Test fallback behavior**

1. Use a non-loaded locale (should fail gracefully)
2. Verify fallback to default locale

- [ ] **Step 4: Commit test results**

```bash
cd /mnt/sdb1/qt4rust/.worktrees/i18n
git commit --allow-empty -m "test(i18n): integration tests completed"
```

---

## Summary

This implementation plan covers:

1. **Phase 1: Core Infrastructure**
   - Add fluent dependencies
   - Create i18n module with trait abstraction
   - Create translation files

2. **Phase 2: QML Integration**
   - Initialize i18n in application
   - Create QML translation helper
   - Add language selector to settings

3. **Phase 3: String Externalization**
   - Migrate all QML strings to use Tr.t()
   - ~195 strings across 30 files

4. **Phase 4: Testing**
   - Integration tests for switching, persistence, fallback

**Estimated total tasks:** ~30-40 individual steps
**Key files created:** 8 new files
**Key files modified:** ~35 QML files + 2 Rust files

---

## Execution Choice

**Plan complete and saved to `docs/superpowers/plans/2025-05-01-i18n-implementation-plan.md`.**

**Two execution options:**

1. **Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

2. **Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?