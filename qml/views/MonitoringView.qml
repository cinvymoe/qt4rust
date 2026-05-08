// MonitoringView.qml - 监控主视图
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qt.rust.demo
import "../components/controls"
import "../components/layouts"
import "../components/dialogs"
import "../styles"

Item {
    id: monitoringView
    
    property bool headerVisible: true
    property int pageIndex: 0
    property int currentIndex: 0
    property bool isActivePage: currentIndex === pageIndex
    
    onIsActivePageChanged: {
        if (isActivePage) {
            console.log("[QML] MonitoringView: Page activated, starting timer")
            displayTimer.start()
        } else {
            console.log("[QML] MonitoringView: Page deactivated, stopping timer")
            displayTimer.stop()
        }
    }
    
    MonitoringViewModel {
        id: viewModel

        Component.onCompleted: {
            console.log("[QML] MonitoringViewModel created")
            viewModel.init_display_pipeline_from_global()
            console.log("[QML] Display pipeline initialization called")
        }

        Component.onDestruction: {
            displayTimer.stop()
        }
    }

    AlarmThresholdViewModel {
        id: alarmThresholdViewModel
    }

    Timer {
        id: displayTimer
        interval: 500
        running: false
        repeat: true
        onTriggered: {
            var updated = viewModel.tick_display()
        }
    }
    

    
    Rectangle {
        anchors.fill: parent
        color: Theme.darkBackground
        
        Image {
            id: backgroundImage
            anchors.centerIn: parent
            source: "qrc:/qt/qml/qt/rust/demo/qml/assets/images/background-monitoring.png"
            fillMode: Image.PreserveAspectFit
            horizontalAlignment: Image.AlignHCenter
            verticalAlignment: Image.AlignVCenter
            width: parent.width * 1.1
            height: parent.width * 1.1 * 9 / 16
            z: 0
        }
        
        Rectangle {
            anchors.bottom: parent.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            height: parent.height * 0.35
            z: 0
            gradient: Gradient {
                orientation: Gradient.Vertical
                GradientStop { position: 0.0; color: "transparent" }
                GradientStop { position: 1.0; color: Theme.darkBackground }
            }
        }
        
        Rectangle {
            anchors.left: parent.left
            anchors.top: parent.top
            anchors.bottom: parent.bottom
            width: parent.width * 0.15
            z: 0
            gradient: Gradient {
                orientation: Gradient.Horizontal
                GradientStop { position: 0.0; color: Theme.darkBackground }
                GradientStop { position: 1.0; color: "transparent" }
            }
        }
        
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
                    text: "传感器连接断开"
                    font.pixelSize: Theme.fontSizeMedium
                    font.family: Theme.fontFamilyDefault
                    color: Theme.textPrimary
                    anchors.verticalCenter: parent.verticalCenter
                }
            }
        }
        
        // 顶部栏
        Header {
            id: header
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            height: monitoringView.headerVisible ? Theme.headerHeight : 0
            visible: height > 0
            alertActive: viewModel.is_warning || viewModel.is_danger
            isWarning: viewModel.is_warning && !viewModel.is_danger
            isDanger: viewModel.is_danger
            isAngleAlarm: viewModel.is_angle_alarm
            clip: true
            z: 2
            
            Behavior on height {
                NumberAnimation {
                    duration: Theme.animationDuration
                    easing.type: Easing.InOutQuad
                }
            }
        }
        
        // 主内容区域 - 直接使用 Item，不嵌套 Column
        Item {
            id: contentArea
            anchors.top: header.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.margins: Theme.spacingMedium
            z: 1
            
            // 预警/报警状态卡片 - 左上角
            DangerCard {
                id: dangerCard
                anchors.left: parent.left
                anchors.top: parent.top
                width: parent.width * 0.35
                height: 96
                visible: viewModel.is_warning || viewModel.is_danger
                isWarning: viewModel.is_warning && !viewModel.is_danger
                isAngleAlarm: viewModel.is_angle_alarm
                title: {
                    if (viewModel.is_angle_alarm) return "角度报警"
                    return viewModel.is_danger ? "危险报警" : "力矩预警"
                }
                message: {
                    if (viewModel.is_angle_alarm) return "吊臂角度超限！请立即调整角度"
                    return viewModel.is_danger ?
                        "力矩严重超限！立即停止作业" :
                        "力矩接近上限，请注意控制载荷"
                }

                Behavior on visible {
                    NumberAnimation {
                        duration: Theme.animationDuration
                        easing.type: Easing.InOutQuad
                    }
                }
            }

            // 时间卡片 - 左上角
            TimeCard {
                id: timeCard
                anchors.left: parent.left
                anchors.top: parent.top
                width: parent.width * 0.35
                height: 96
                visible: !viewModel.is_warning && !viewModel.is_danger
                
                Behavior on visible {
                    NumberAnimation {
                        duration: Theme.animationDuration
                        easing.type: Easing.InOutQuad
                    }
                }
            }

            // 力矩百分比卡片 - 左下角
            MomentCard {
                id: momentCard
                anchors.left: parent.left
                anchors.bottom: parent.bottom
                width: parent.width * 0.35
                height: 216
                percentage: viewModel.moment_percentage
                warningThreshold: alarmThresholdViewModel.moment_warning_threshold
                dangerThreshold: alarmThresholdViewModel.moment_danger_threshold
            }
            
            // 当前载荷卡片 - 左侧
            DataCard {
                id: currentLoadCard
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width * 0.28
                height: 150

                iconSource: "qrc:/qt/qml/qt/rust/demo/qml/assets/images/icon-weight.png"
                label: "当前载荷"
                value: (viewModel.current_load || 0).toFixed(1)
                unit: "吨"
                description: "额定: " + (viewModel.rated_load || 25).toFixed(1) + "吨"
                showProgress: true
                progress: (viewModel.current_load || 0) / (viewModel.rated_load || 25)
            }

            // 工作半径卡片 - 右侧
            DataCard {
                id: workingRadiusCard
                anchors.bottom: parent.bottom
                anchors.horizontalCenter: parent.horizontalCenter
                anchors.bottomMargin: 60
                anchors.horizontalCenterOffset: 100
                width: parent.width * 0.28
                height: 150

                iconSource: "qrc:/qt/qml/qt/rust/demo/qml/assets/images/icon-radius.png"
                label: "工作半径"
                value: (viewModel.working_radius || 0).toFixed(1)
                unit: "米"
                description: "水平工作距离"
                showProgress: false
            }

            // 吊臂角度卡片 - 中间
            DataCard {
                id: boomAngleCard
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width * 0.28
                height: 150

                iconSource: "qrc:/qt/qml/qt/rust/demo/qml/assets/images/icon-angle.png"
                label: "吊臂角度"
                value: (viewModel.boom_angle || 0).toFixed(1)
                unit: "度"
                description: "与水平面夹角"
                showProgress: false
            }

            // 臂长卡片 - 底部
            BoomLengthCard {
                id: boomLengthCard
                anchors.top: parent.top
                anchors.horizontalCenter: parent.horizontalCenter
                width: parent.width * 0.28
                height: 150

                boomLength: viewModel.boom_length || 0
            }
        }
    }
}
