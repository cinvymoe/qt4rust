## Main / Navigation
main-title = 起重机力矩监测系统
main-subtitle = Crane Monitoring System

nav-monitoring = 主界面
nav-charts = 数据曲线
nav-alarms = 报警记录
nav-settings = 设置

## Monitoring View
monitoring-title = 实时监控
monitoring-currentLoad = 当前载荷
monitoring-workingRadius = 工作半径
monitoring-boomAngle = 吊臂角度
monitoring-boomLength = 臂长
monitoring-ratedLoad = 额定载荷
monitoring-unit-ton = 吨
monitoring-unit-meter = 米
monitoring-unit-degree = 度
monitoring-sensorDisconnected = 传感器连接断开
monitoring-horizontalDistance = 水平工作距离

## Danger Card
danger-title-warning = 力矩预警
danger-title-danger = 危险报警
danger-title-angleAlarm = 角度报警
danger-message-warning = 力矩接近上限，请注意控制载荷
danger-message-danger = 力矩严重超限！立即停止作业
danger-message-angleAlarm = 吊臂角度超限！请立即调整角度
danger-craneStatus = 起重机臂架状态

## Alarm Records
alarm-title = 报警记录
alarm-subtitle = 系统报警历史与统计分析
alarm-totalCount = 总报警次数
alarm-warningCount = 预警次数
alarm-dangerCount = 危险次数
alarm-levels = 报警级别说明
alarm-level-normal = 正常
alarm-level-normalDesc = 力矩 0-75%
alarm-level-warning = 预警
alarm-level-warningDesc = 力矩 75-90%
alarm-level-danger = 危险
alarm-level-dangerDesc = 力矩 ≥90%
alarm-momentValue = 力矩

## Chart View
chart-title = 数据曲线分析
chart-subtitle = 实时监测数据变化趋势
chart-refresh = 刷新
chart-loadTrend = 载荷变化曲线
chart-momentTrend = 力矩百分比趋势
chart-multiParam = 多参数对比分析
chart-workingRadius = 工作半径 (m)
chart-boomAngle = 吊臂角度 (°)

## Settings View
settings-title = 设置
settings-systemStatus = 系统状态
settings-calibration = 参数校准
settings-momentCurve = 力矩曲线
settings-about = 关于系统
settings-language = 语言
settings-language-zhCN = 简体中文
settings-language-enUS = English
settings-reload = 重新加载配置
settings-save = 保存设置

## Calibration
calibration-loadSensor = 载荷传感器
calibration-angleSensor = 角度传感器
calibration-radiusSensor = 长度传感器
calibration-alarmThreshold = 报警阈值
calibration-restoreDefault = 恢复默认
calibration-adValue = AD值
calibration-physicalValue = 重量（吨）
calibration-angleValue = 角度（°）
calibration-radiusValue = 长度（m）
calibration-calibrating = 实时采集中
calibration-multiplier = 标定倍率
calibration-multiplierDesc = 标定倍率用于调整传感器灵敏度，适用于不同测量范围的应用场景
calibration-selectMultiplier = 选择自定义倍率
calibration-selectMultiplierDesc = 请选择标定倍率（0.5 - 10.0）
calibration-loadNote1 = 至少需要2个标定点，建议使用3-5个标定点
calibration-loadNote2 = 标定点应均匀覆盖整个测量范围
calibration-loadNote3 = AD值是传感器的原始输出值（模拟数字转换值）
calibration-loadNote4 = 使用标准砝码进行标定，确保重量准确
calibration-loadNote5 = 标定时应确保设备稳定，避免振动干扰
calibration-loadNote6 = 系统使用线性插值算法计算两点之间的重量值
calibration-angleNote1 = 至少需要2个标定点，建议在0°和最大角度处标定
calibration-angleNote2 = 使用精密角度测量仪器进行标定
calibration-angleNote3 = 标定时确保臂架处于稳定状态
calibration-angleNote4 = 角度传感器对温度敏感，建议在工作温度下标定
calibration-angleNote5 = 系统使用线性插值算法计算两点之间的角度值
calibration-radiusNote1 = 至少需要2个标定点，建议在最小和最大工作长度处标定
calibration-radiusNote2 = 使用精确测量工具确定实际工作长度
calibration-radiusNote3 = 标定时确保臂架完全伸展或收缩到位
calibration-radiusNote4 = 长度传感器通常为拉绳式或角度换算，需定期检查
calibration-radiusNote5 = 系统使用线性插值算法计算两点之间的长度值

## About System
about-title = 汽车吊力矩监测系统
about-subtitle = Crane Moment Monitoring System
about-version = 系统版本
about-releaseDate = 发布日期
about-firmware = 固件版本
about-hardware = 硬件版本
about-features = 系统特性
about-feature-realtime = 实时安全监控
about-feature-realtimeDesc = 24/7不间断监测起重机力矩状态，确保作业安全
about-feature-warning = 三级预警系统
about-feature-warningDesc = 正常、预警、危险三级报警，提前预防安全事故
about-feature-sensor = 高精度传感器
about-feature-sensorDesc = 采用工业级传感器，精度±0.5%，响应时间<50ms
about-techSpecs = 技术规格
about-certifications = 认证与标准
about-techSupport = 技术支持
about-hotline = 服务热线
about-email = 技术邮箱
about-address = 公司地址
about-addressValue = 北京市海淀区中关村科技园区
about-copyright = © 2025 汽车吊力矩监测系统 版权所有
about-copyrightNotice = 未经授权禁止复制、传播或使用本系统的任何内容

## Moment Card
moment-percentage = 力矩百分比
moment-warning = 预警
moment-danger = 危险

## Time Card
timeCard-time = 时间

## Boom Length Card
boomLength-title = 臂长
boomLength-unit = 米
boomLength-description = 吊臂总长度

## Load Comparison Card
loadComparison-title = 载荷对比
loadComparison-actual = 实际
loadComparison-rated = 额定

## Moment Curve
momentCurve-title = 力矩曲线图说明：
momentCurve-ratedCurve = 额定载荷曲线
momentCurve-importCurve = 导入曲线
momentCurve-boomLength = 臂长
momentCurve-currentLength = 当前臂长
momentCurve-maxLoad = 最大载荷
momentCurve-maxRadius = 最大幅度
momentCurve-dataPoints = 数据点数

## Alarm Threshold
alarmThreshold-momentTitle = 力矩报警阈值
alarmThreshold-momentDesc = 设置力矩百分比报警阈值
alarmThreshold-warningPercent = 预警阈值（%）
alarmThreshold-dangerPercent = 危险阈值（%）
alarmThreshold-momentNote = 当力矩百分比超过预警阈值时显示黄色警告，超过危险阈值时显示红色报警
alarmThreshold-angleTitle = 角度报警阈值
alarmThreshold-angleDesc = 设置吊臂角度报警范围
alarmThreshold-angleLower = 角度下限（度）
alarmThreshold-angleUpper = 角度上限（度）
alarmThreshold-angleNote = 当吊臂角度低于下限或高于上限时，系统将发出报警
alarmThreshold-mainHook = 主钩勾头开关报警
alarmThreshold-mainHookDesc = 设置主钩勾头开关状态报警触发条件
alarmThreshold-subHook = 副钩勾头开关报警
alarmThreshold-subHookDesc = 设置副钩勾头开关状态报警触发条件
alarmThreshold-mode = 报警模式
alarmThreshold-modeNO = 常开
alarmThreshold-modeNC = 常闭
alarmThreshold-mainHookNote = 选择主钩勾头开关报警触发条件：常开或常闭（二者只能选其一）
alarmThreshold-subHookNote = 选择副钩勾头开关报警触发条件：常开或常闭（二者只能选其一）
alarmThreshold-notes = 报警阈值设置说明
alarmThreshold-note1 = 力矩报警是最重要的安全保护，建议预警阈值设为80%
alarmThreshold-note2 = 危险阈值达到100%时系统将强制停止作业
alarmThreshold-note3 = 报警触发后会记录到报警记录中，可在报警记录页面查看
alarmThreshold-note4 = 修改阈值后需要保存设置才能生效

## Dialogs
dialog-cancel = 取消
dialog-confirm = 确定
dialog-selectMultiplier = 选择自定义倍率
dialog-selectMultiplierDesc = 请选择标定倍率（0.5 - 10.0）
dialog-timeRange = 时间筛选设置
dialog-timeRangeDesc = 选择自定义时间范围以筛选曲线数据
dialog-startTime = 开始时间
dialog-endTime = 结束时间（可选）
dialog-resetFilter = 重置筛选
dialog-applyFilter = 应用筛选
dialog-importRatedLoad = 导入额定载荷表
dialog-importRatedLoadDesc = 请输入 CSV 文件路径
dialog-importHint = 提示：可以使用相对路径或绝对路径
dialog-info = 信息

## Filter Bar
filter-history = 历史记录:
filter-all = 全部
filter-today = 今天
filter-last7days = 最近7天
filter-last30days = 最近30天
filter-custom = 自定义

## Time Range
timeRange-1hour = 1小时
timeRange-2hours = 2小时
timeRange-5hours = 5小时
timeRange-custom = 自定义
timeRange-label = 时间范围:

## System Status
systemStatus-sensorConnection = 传感器连接状态
systemStatus-systemInfo = 系统信息
systemStatus-network = 网络与通信

## Common
common-version = v1.0.0
common-rated = 额定
