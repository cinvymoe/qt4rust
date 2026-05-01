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
monitoring.horizontalDistance = Horizontal Working Distance

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
alarm.momentValue = Moment

## Chart View
chart.title = Data Curve Analysis
chart.subtitle = Real-time Monitoring Data Trends
chart.refresh = Refresh
chart.loadTrend = Load Trend Curve
chart.momentTrend = Moment Percentage Trend
chart.multiParam = Multi-Parameter Comparison
chart.workingRadius = Working Radius (m)
chart.boomAngle = Boom Angle (°)

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
calibration.multiplierDesc = Multiplier adjusts sensor sensitivity for different measurement ranges
calibration.selectMultiplier = Select Multiplier
calibration.selectMultiplierDesc = Please select multiplier (0.5 - 10.0)
calibration.multiplierNote = At least 2 calibration points required
calibration.loadNote1 = At least 2 calibration points needed, 3-5 recommended
calibration.loadNote2 = Calibration points should evenly cover the measurement range
calibration.loadNote3 = AD value is the raw sensor output (analog-to-digital value)
calibration.loadNote4 = Use standard weights for calibration to ensure accuracy
calibration.loadNote5 = Ensure device stability during calibration, avoid vibration
calibration.loadNote6 = System uses linear interpolation between calibration points
calibration.angleNote1 = At least 2 points, calibrate at 0° and max angle
calibration.angleNote2 = Use precision angle measurement instruments
calibration.angleNote3 = Ensure boom is stable during calibration
calibration.angleNote4 = Angle sensors are temperature-sensitive, calibrate at working temperature
calibration.angleNote5 = System uses linear interpolation between calibration points
calibration.radiusNote1 = At least 2 points, calibrate at min and max working length
calibration.radiusNote2 = Use precise measuring tools for actual working length
calibration.radiusNote3 = Ensure boom is fully extended or retracted during calibration
calibration.radiusNote4 = Radius sensors are typically draw-wire or angle-derived, check regularly
calibration.radiusNote5 = System uses linear interpolation between calibration points

## About System
about.title = Crane Moment Monitoring System
about.subtitle = Crane Moment Monitoring System
about.version = System Version
about.releaseDate = Release Date
about.firmware = Firmware Version
about.hardware = Hardware Version
about.features = System Features
about.feature.realtime = Real-time Safety Monitoring
about.feature.realtimeDesc = 24/7 continuous monitoring of crane moment status, ensuring safety
about.feature.warning = Three-level Warning System
about.feature.warningDesc = Normal, Warning, Danger levels for accident prevention
about.feature.sensor = High-precision Sensors
about.feature.sensorDesc = Industrial-grade sensors, ±0.5% accuracy, <50ms response time
about.techSpecs = Technical Specifications
about.certifications = Certifications & Standards
about.techSupport = Technical Support
about.hotline = Service Hotline
about.email = Technical Email
about.address = Company Address
about.addressValue = Zhongguancun Science Park, Haidian District, Beijing
about.copyright = © 2025 Crane Moment Monitoring System. All rights reserved.
about.copyrightNotice = Unauthorized copying, distribution, or use of any content is prohibited

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

## Load Comparison Card
loadComparison.title = Load Comparison
loadComparison.actual = Actual
loadComparison.rated = Rated

## Moment Curve
momentCurve.title = Moment Curve Description:
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
alarmThreshold.momentNote = Yellow warning when exceeding warning threshold, red alarm when exceeding danger threshold
alarmThreshold.angleTitle = Angle Alarm Threshold
alarmThreshold.angleDesc = Set boom angle alarm range
alarmThreshold.angleLower = Lower Angle Limit (°)
alarmThreshold.angleUpper = Upper Angle Limit (°)
alarmThreshold.angleNote = Alarm triggers when boom angle is below lower limit or above upper limit
alarmThreshold.mainHook = Main Hook Switch Alarm
alarmThreshold.mainHookDesc = Set main hook switch alarm trigger condition
alarmThreshold.subHook = Sub Hook Switch Alarm
alarmThreshold.subHookDesc = Set sub hook switch alarm trigger condition
alarmThreshold.mode = Alarm Mode
alarmThreshold.modeNO = Normally Open
alarmThreshold.modeNC = Normally Closed
alarmThreshold.mainHookNote = Select main hook switch trigger: Normally Open or Normally Closed (mutually exclusive)
alarmThreshold.subHookNote = Select sub hook switch trigger: Normally Open or Normally Closed (mutually exclusive)
alarmThreshold.notes = Alarm Threshold Notes
alarmThreshold.note1 = Moment alarm is the most important safety protection, recommend warning threshold at 80%
alarmThreshold.note2 = System will force stop operation when danger threshold reaches 100%
alarmThreshold.note3 = Triggered alarms are recorded and can be viewed in Alarm Records
alarmThreshold.note4 = Modified thresholds require saving to take effect

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
dialog.info = Info

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
timeRange.label = Time Range:

## System Status
systemStatus.sensorConnection = Sensor Connection Status
systemStatus.systemInfo = System Information
systemStatus.network = Network & Communication

## Home View
home.placeholder = Click to test virtual keyboard...
home.inputContent = Input content

## Common
common.version = v1.0.0
common.rated = Rated
