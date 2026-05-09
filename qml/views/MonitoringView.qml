// MonitoringView.qml - 监控主视图
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qt.rust.demo  // 导入 Rust ViewModel
import "../components/controls"
import "../components/layouts"
import "../components/dialogs"  // 导入对话框组件
import "../styles"

Item {
    id: monitoringView
    
    // i18n 翻译对象
    
    // 顶部栏显示状态（从父组件传递）
    property bool headerVisible: true
    
    // 页面索引和当前索引（用于判断是否为活动页面）
    property int pageIndex: 0
    property int currentIndex: 0
    
    // 是否为当前活动页面
    property bool isActivePage: currentIndex === pageIndex
    
    // 当页面激活/失活时控制定时器
    onIsActivePageChanged: {
        if (isActivePage) {
            console.log("[QML] MonitoringView: Page activated, starting timer")
            displayTimer.start()
        } else {
            console.log("[QML] MonitoringView: Page deactivated, stopping timer")
            displayTimer.stop()
        }
    }
    
    // 绑定 MonitoringViewModel
    MonitoringViewModel {
        id: viewModel

        // 组件创建完成后初始化显示管道
        Component.onCompleted: {
            console.log("[QML] MonitoringViewModel created")

            // 初始化显示管道（从全局共享缓冲区）
            viewModel.init_display_pipeline_from_global()
            console.log("[QML] Display pipeline initialization called")

            // 注意：定时器由 isActivePage 控制，不在初始化时启动
            console.log("[QML] Available methods:")
            for (var prop in viewModel) {
                if (typeof viewModel[prop] === "function") {
                    console.log("[QML]   -", prop)
                }
            }
        }

        Component.onDestruction: {
            displayTimer.stop()
            console.log("[QML] Display timer stopped (destruction)")
        }
    }

    // 绑定 AlarmThresholdViewModel - 用于读取报警阈值配置
    AlarmThresholdViewModel {
        id: alarmThresholdViewModel
    }

    // 显示更新定时器（管道三：主线程显示管道）
    Timer {
        id: displayTimer
        interval: 500  // 100ms = 10Hz，与采集管道频率匹配
        running: false
        repeat: true
        onTriggered: {
            // 调用 ViewModel 的 tick_display() 方法
            // 从 DisplayPipeline 获取最新数据并更新 UI
            var updated = viewModel.tick_display()
        }
    }
    

    
    // 数据模型 - 使用 ViewModel 数据动态更新
    ListModel {
        id: monitoringDataModel
    }
    
    // 初始化数据模型
    Component.onCompleted: {
        updateDataModel()
    }
    
    // 监听 ViewModel 属性变化，更新数据模型
    Connections {
        target: viewModel
        function onCurrent_loadChanged() { updateDataModel() }
        function onWorking_radiusChanged() { updateDataModel() }
        function onBoom_angleChanged() { updateDataModel() }
        function onBoom_lengthChanged() { updateDataModel() }
    }
    
    // 更新数据模型函数
    function updateDataModel() {
        monitoringDataModel.clear()

        // 确保值不为 undefined（注意：属性名使用 snake_case）
        var currentLoad = viewModel.current_load || 0
        var ratedLoad = viewModel.rated_load || 25
        var workingRadius = viewModel.working_radius || 0
        var boomAngle = viewModel.boom_angle || 0
        var boomLength = viewModel.boom_length || 22.6

        monitoringDataModel.append({
            type: "dataCard",
            iconSource: "qrc:/qt/qml/qt/rust/demo/qml/assets/images/icon-weight.png",
            label: TranslationBridge.translate("monitoring.currentLoad"),
            unit: TranslationBridge.translate("monitoring.unit.ton"),
            description: TranslationBridge.translate("monitoring.ratedLoad"),
            showProgress: true,
            value: currentLoad,
            maxValue: ratedLoad
        })

        monitoringDataModel.append({
            type: "dataCard",
            iconSource: "qrc:/qt/qml/qt/rust/demo/qml/assets/images/icon-radius.png",
            label: TranslationBridge.translate("monitoring.workingRadius"),
            unit: TranslationBridge.translate("monitoring.unit.meter"),
            description: TranslationBridge.translate("monitoring.horizontalDistance"),
            showProgress: false,
            value: workingRadius,
            maxValue: 0.0
        })

        monitoringDataModel.append({
            type: "dataCard",
            iconSource: "qrc:/qt/qml/qt/rust/demo/qml/assets/images/icon-angle.png",
            label: TranslationBridge.translate("monitoring.boomAngle"),
            unit: TranslationBridge.translate("monitoring.unit.degree"),
            description: TranslationBridge.translate("monitoring.angleWithHorizontal"),
            showProgress: false,
            value: boomAngle,
            maxValue: 0.0
        })

        monitoringDataModel.append({
            type: "boomLength",
            label: TranslationBridge.translate("monitoring.boomLength"),
            value: boomLength
        })
    }
    
    Rectangle {
        anchors.fill: parent
        color: "transparent"
        
        // 传感器断连提示
        Rectangle {
            id: sensorDisconnectedBanner
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            height: 40
            color: Theme.dangerBackground
            border.color: Theme.dangerColor
            border.width: Theme.borderNormal
            visible: !viewModel.sensor_connected
            z: 100
            
            Row {
                anchors.centerIn: parent
                spacing: Theme.spacingMedium
                
                Rectangle {
                    width: Theme.iconSizeSmall
                    height: Theme.iconSizeSmall
                    color: Theme.dangerColor
                    radius: width / 2
                    anchors.verticalCenter: parent.verticalCenter
                }
                
                Text {
                    text: TranslationBridge.translate("monitoring.sensorDisconnected")
                    font.pixelSize: Theme.fontSizeMedium
                    font.family: Theme.fontFamilyDefault
                    color: Theme.textPrimary
                    anchors.verticalCenter: parent.verticalCenter
                }
            }
        }
        
        Column {
            anchors.fill: parent
            spacing: 0
            
            // 顶部栏 - 绑定预警或危险状态
            Header {
                id: header
                width: parent.width
                height: monitoringView.headerVisible ? Theme.headerHeight : 0
                visible: height > 0
                alertActive: viewModel.is_warning || viewModel.is_danger
                isWarning: viewModel.is_warning && !viewModel.is_danger
                isDanger: viewModel.is_danger
                isAngleAlarm: viewModel.is_angle_alarm
                clip: true
                
                Behavior on height {
                    NumberAnimation {
                        duration: Theme.animationDuration
                        easing.type: Easing.InOutQuad
                    }
                }
            }
            
            // 主内容区域
            Rectangle {
                width: parent.width
                height: parent.height - header.height
                color: "transparent"
                
                Row {
                    anchors.fill: parent
                    anchors.margins: Theme.spacingMedium
                    spacing: Theme.spacingMedium
            
            // 左列
            Column {
                width: (parent.width - Theme.spacingMedium) / 2
                height: parent.height
                spacing: Theme.spacingMedium
                
                // 预警/报警状态卡片 - 预警或报警时显示
                DangerCard {
                    id: dangerCard
                    width: parent.width
                    height: 96
                    visible: viewModel.is_warning || viewModel.is_danger
                    // 预警状态为黄色，报警状态为红色
                    isWarning: viewModel.is_warning && !viewModel.is_danger
                    isAngleAlarm: viewModel.is_angle_alarm
                    // 根据状态显示不同的消息：角度报警、危险报警或力矩预警
                    title: {
                        if (viewModel.is_angle_alarm) return TranslationBridge.translate("danger.title.angleAlarm")
                        return viewModel.is_danger ? TranslationBridge.translate("danger.title.danger") : TranslationBridge.translate("danger.title.warning")
                    }
                    message: {
                        if (viewModel.is_angle_alarm) return TranslationBridge.translate("danger.message.angleAlarm")
                        return viewModel.is_danger ?
                            TranslationBridge.translate("danger.message.danger") :
                            TranslationBridge.translate("danger.message.warning")
                    }

                    Behavior on visible {
                        NumberAnimation {
                            duration: Theme.animationDuration
                            easing.type: Easing.InOutQuad
                        }
                    }
                }

                // 时间卡片 - 预警或报警时隐藏
                TimeCard {
                    id: timeCard
                    width: parent.width
                    height: 96
                    visible: !viewModel.is_warning && !viewModel.is_danger
                    
                    Behavior on visible {
                        NumberAnimation {
                            duration: Theme.animationDuration
                            easing.type: Easing.InOutQuad
                        }
                    }
                }
                
                // 起重机臂架状态卡片
                Rectangle {
                    width: parent.width
                    height: parent.height - 96 - Theme.spacingMedium
                    color: Theme.darkSurface
                    border.color: Theme.darkBorder
                    border.width: Theme.borderThin
                    radius: Theme.radiusMedium
                    
                    Column {
                        anchors.fill: parent
                        anchors.margins: Theme.spacingMedium
                        spacing: Theme.spacingMedium
                        
                        // 标题
                        Row {
                            spacing: Theme.spacingMedium
                            
                            Rectangle {
                                width: 4
                                height: Theme.fontSizeLarge
                                color: Theme.darkAccent
                                anchors.verticalCenter: parent.verticalCenter
                            }
                            
Text {
                                 text: TranslationBridge.translate("danger.craneStatus")
                                 font.pixelSize: Theme.fontSizeLarge
                                 font.family: Theme.fontFamilyDefault
                                 font.weight: Font.Medium
                                 color: Theme.textPrimary
                                 anchors.verticalCenter: parent.verticalCenter
                             }
                        }
                        
                        // 臂架图
                        Rectangle {
                            width: parent.width
                            height: parent.height - Theme.fontSizeLarge - Theme.spacingMedium * 2
                            color: Theme.darkBackground
                            radius: Theme.radiusMedium
                            
                            Image {
                                anchors.centerIn: parent
                                source: "qrc:/qt/qml/qt/rust/demo/qml/assets/images/canvas-crane.png"
                                fillMode: Image.PreserveAspectFit
                                width: parent.width - Theme.spacingMedium * 2
                                height: parent.height - Theme.spacingMedium * 2
                            }
                        }
                    }
                }
            }
            
            // 右列
            Column {
                width: (parent.width - Theme.spacingMedium) / 2
                height: parent.height
                spacing: Theme.spacingMedium
                
                // 力矩百分比卡片 - 绑定 ViewModel 和阈值配置
                MomentCard {
                    width: parent.width
                    height: 216
                    percentage: viewModel.moment_percentage
                    warningThreshold: alarmThresholdViewModel.moment_warning_threshold
                    dangerThreshold: alarmThresholdViewModel.moment_danger_threshold
                }
                
                // 数据网格 - 使用 GridView (每行2列，可滑动)
                GridView {
                    id: dataGridView
                    width: parent.width
                    height: parent.height - 216 - Theme.spacingMedium
                    cellWidth: parent.width / 2
                    cellHeight: 150 + Theme.spacingMedium
                    clip: true
                    
                    // 使用外部定义的数据模型
                    model: monitoringDataModel
                    
                    // 网格项目
                    delegate: Item {
                        width: dataGridView.cellWidth
                        height: dataGridView.cellHeight
                        
                        // 直接在 delegate 中渲染，不使用 Loader
                        DataCard {
                            anchors.fill: parent
                            anchors.margins: Theme.spacingMedium / 2
                            visible: model.type === "dataCard"
                            
                            iconSource: model.iconSource || ""
                            label: model.label || ""
                            value: (model.value || 0).toFixed(1)
                            unit: model.unit || ""
                            description: {
                                if (model.showProgress) {
                                    return TranslationBridge.translate("common.rated") + ": " + (model.maxValue || 0).toFixed(1) + (model.unit || "")
                                }
                                return model.description || ""
                            }
                            showProgress: model.showProgress || false
                            progress: model.showProgress ? ((model.value || 0) / (model.maxValue || 1)) : 0
                        }
                        
                        BoomLengthCard {
                            anchors.fill: parent
                            anchors.margins: Theme.spacingMedium / 2
                            visible: model.type === "boomLength"
                            
                            boomLength: model.value || 0
                        }
                    }
                }
            }
        }
            }
        }
    }
}
