// CalibrationView.qml - 参数校准子页面
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qt.rust.demo
import "../../styles"
import "../../i18n"
import "CalibrationContents"

Item {
    id: calibrationView
    
    property int currentSensorTab: 0  // 0: 载荷, 1: 角度, 2: 长度, 3: 报警阈值
    
    Tr { id: tr }
    
    // 绑定 CalibrationViewModel
    CalibrationViewModel {
        id: viewModel
    }
    
    // 定时器：每 500ms 更新传感器数据
    Timer {
        id: sensorUpdateTimer
        interval: 500
        running: true
        repeat: true
        onTriggered: {
            viewModel.update_sensor_data()
        }
    }
    
    Component.onCompleted: {
        console.log("[CalibrationView] Initialized with timer interval:", sensorUpdateTimer.interval, "ms")
    }
    
    Row {
        anchors.fill: parent
        spacing: 0
        
        // 左侧：传感器实时数据面板
        Rectangle {
            width: Math.max(240, Math.min(280, parent.width * 0.35))
            height: parent.height
            color: Theme.darkSurface
            border.color: Theme.darkBorder
            border.width: Theme.borderThin
            
            Flickable {
                anchors.fill: parent
                contentHeight: sensorColumn.height
                clip: true
                
                Column {
                    id: sensorColumn
                    width: parent.width
                    spacing: Theme.spacingMedium
                    topPadding: Theme.spacingMedium
                    leftPadding: Theme.spacingMedium
                    rightPadding: Theme.spacingMedium
                    
                    // 重量传感器卡片
                    SensorDataCard {
                        width: parent.width - parent.leftPadding - parent.rightPadding
                        sensorName: tr.t("calibration.loadSensor")
                        adValue: (viewModel.ad1_load || 0).toFixed(2)
                        calculatedValue: (viewModel.calculated_load || 0).toFixed(2)
                        unit: tr.t("monitoring.unit.ton")
                        isOnline: viewModel.sensor_connected
                    }
                    
                    // 角度传感器卡片
                    SensorDataCard {
                        width: parent.width - parent.leftPadding - parent.rightPadding
                        sensorName: tr.t("calibration.angleSensor")
                        adValue: (viewModel.ad3_angle || 0).toFixed(2)
                        calculatedValue: (viewModel.calculated_angle || 0).toFixed(1)
                        unit: "°"
                        isOnline: viewModel.sensor_connected
                    }
                    
                    // 侧长传感器卡片
                    SensorDataCard {
                        width: parent.width - parent.leftPadding - parent.rightPadding
                        sensorName: tr.t("calibration.radiusSensor")
                        adValue: (viewModel.ad2_radius || 0).toFixed(2)
                        calculatedValue: (viewModel.calculated_radius || 0).toFixed(2)
                        unit: "m"
                        isOnline: viewModel.sensor_connected
                    }
                }
            }
        }
        
        // 右侧：校准设置区域
        Rectangle {
            width: parent.width - Math.max(240, Math.min(280, parent.width * 0.35))
            height: parent.height
            color: Theme.darkBackground
            
            Column {
                anchors.fill: parent
                spacing: 0
                
                // 传感器类型 Tab 栏
                Rectangle {
                    width: parent.width
                    height: Math.max(36, Math.min(48, parent.height * 0.09))
                    color: Theme.darkSurface
                    border.color: Theme.darkBorder
                    border.width: Theme.borderThin
                    
                    Row {
                        anchors.fill: parent
                        spacing: 0
                        
                        Repeater {
                            model: [
                                {text: tr.t("calibration.loadSensor")},
                                {text: tr.t("calibration.angleSensor")},
                                {text: tr.t("calibration.radiusSensor")},
                                {text: tr.t("calibration.alarmThreshold")}
                            ]
                            
                            Rectangle {
                                width: parent.width / 4
                                height: parent.height - 1
                                color: currentSensorTab === index ? Theme.darkBackground : "transparent"
                                
                                Text {
                                    anchors.centerIn: parent
                                    text: modelData.text
                                    font.pixelSize: Theme.fontSizeSmall
                                    color: currentSensorTab === index ? Theme.textAccent : Theme.textTertiary
                                    font.family: Theme.fontFamilyDefault
                                    elide: Text.ElideRight
                                    width: parent.width - Theme.spacingSmall * 2
                                    horizontalAlignment: Text.AlignHCenter
                                }
                                
                                Rectangle {
                                    visible: currentSensorTab === index
                                    width: parent.width
                                    height: 2
                                    color: Theme.darkAccent
                                    anchors.bottom: parent.bottom
                                }
                                
                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: currentSensorTab = index
                                }
                            }
                        }
                    }
                }
                
                // 校准内容区域
                Item {
                    width: parent.width
                    height: parent.height - Math.max(36, Math.min(48, parent.height * 0.09)) - Math.max(52, Math.min(60, parent.height * 0.12))
                    
                    // 载荷传感器校准
                    LoadCalibrationContent {
                        id: loadCalibrationContent
                        anchors.fill: parent
                        visible: currentSensorTab === 0
                    }
                    
                    // 角度传感器校准
                    AngleCalibrationContent {
                        id: angleCalibrationContent
                        anchors.fill: parent
                        visible: currentSensorTab === 1
                    }
                    
                    // 长度传感器校准
                    RadiusCalibrationContent {
                        id: radiusCalibrationContent
                        anchors.fill: parent
                        visible: currentSensorTab === 2
                    }
                    
                    // 报警阈值设置
                    AlarmThresholdContent {
                        id: alarmThresholdContent
                        anchors.fill: parent
                        visible: currentSensorTab === 3
                    }
                }
                
                // 底部操作按钮
                Rectangle {
                    width: parent.width
                    height: Math.max(52, Math.min(60, parent.height * 0.12))
                    color: Theme.darkSurface
                    border.color: Theme.darkBorder
                    border.width: Theme.borderThin
                    
                    Row {
                        anchors.left: parent.left
                        anchors.leftMargin: Theme.spacingMedium
                        anchors.right: parent.right
                        anchors.rightMargin: Theme.spacingMedium
                        anchors.verticalCenter: parent.verticalCenter
                        spacing: Theme.spacingSmall
                        
                        Button {
                            width: Math.max(100, 120)
                            height: Math.max(36, Math.min(44, parent.parent.height * 0.7))
                            
                            background: Rectangle {
                                color: "#314158"
                                radius: Theme.radiusMedium
                            }
                            
                            contentItem: Row {
                                spacing: Theme.spacingSmall
                                anchors.centerIn: parent
                                
                                Image {
                                    width: Theme.iconSizeSmall
                                    height: Theme.iconSizeSmall
                                    anchors.verticalCenter: parent.verticalCenter
                                    source: "../../assets/images/icon-settings.svg"
                                }
                                
                                Text {
                                    text: tr.t("calibration.restoreDefault")
                                    font.pixelSize: Theme.fontSizeSmall
                                    color: Theme.textPrimary
                                    anchors.verticalCenter: parent.verticalCenter
                                }
                            }
                            
                            onClicked: {
                                switch(currentSensorTab) {
                                    case 0:
                                        loadCalibrationContent.resetToDefault()
                                        console.log("载荷传感器恢复默认")
                                        break
                                    case 1:
                                        if (angleCalibrationContent.resetToDefault) {
                                            angleCalibrationContent.resetToDefault()
                                        }
                                        console.log("角度传感器恢复默认")
                                        break
                                    case 2:
                                        if (radiusCalibrationContent.resetToDefault) {
                                            radiusCalibrationContent.resetToDefault()
                                        }
                                        console.log("长度传感器恢复默认")
                                        break
                                    case 3:
                                        if (alarmThresholdContent.resetToDefault) {
                                            alarmThresholdContent.resetToDefault()
                                        }
                                        console.log("报警阈值恢复默认")
                                        break
                                }
                            }
                        }
                        
                        Button {
                            width: parent.width - Math.max(100, 120) - Theme.spacingSmall
                            height: Math.max(36, Math.min(44, parent.parent.height * 0.7))
                            
                            background: Rectangle {
                                color: "#155dfc"
                                radius: Theme.radiusMedium
                            }
                            
                            contentItem: Row {
                                spacing: Theme.spacingSmall
                                anchors.centerIn: parent
                                
                                Image {
                                    width: Theme.iconSizeSmall
                                    height: Theme.iconSizeSmall
                                    anchors.verticalCenter: parent.verticalCenter
                                    source: "../../assets/images/icon-chart.svg"
                                }
                                
                                Text {
                                    text: tr.t("settings.save")
                                    font.pixelSize: Theme.fontSizeSmall
                                    color: Theme.textPrimary
                                    anchors.verticalCenter: parent.verticalCenter
                                }
                            }
                            
                            onClicked: {
                                var success = false
                                switch(currentSensorTab) {
                                    case 0:
                                        success = loadCalibrationContent.saveCalibration()
                                        if (success) {
                                            console.log("载荷传感器校准参数保存成功")
                                        } else {
                                            console.error("载荷传感器校准参数保存失败")
                                        }
                                        break
                                    case 1:
                                        if (angleCalibrationContent.saveCalibration) {
                                            success = angleCalibrationContent.saveCalibration()
                                            if (success) {
                                                console.log("角度传感器校准参数保存成功")
                                            } else {
                                                console.error("角度传感器校准参数保存失败")
                                            }
                                        }
                                        break
                                    case 2:
                                        if (radiusCalibrationContent.saveCalibration) {
                                            success = radiusCalibrationContent.saveCalibration()
                                            if (success) {
                                                console.log("长度传感器校准参数保存成功")
                                            } else {
                                                console.error("长度传感器校准参数保存失败")
                                            }
                                        }
                                        break
                                    case 3:
                                        if (alarmThresholdContent.saveCalibration) {
                                            success = alarmThresholdContent.saveCalibration()
                                            if (success) {
                                                console.log("报警阈值保存成功")
                                            } else {
                                                console.error("报警阈值保存失败")
                                            }
                                        }
                                        break
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // 传感器数据卡片组件
    component SensorDataCard: Rectangle {
        property string sensorName: ""
        property string adValue: "0"
        property string calculatedValue: "0"
        property string unit: ""
        property bool isOnline: true
        
        Tr { id: sensorTr }
        
        height: Math.max(140, Math.min(180, 203))
        color: Theme.darkBackground
        border.color: "#45556c"
        border.width: Theme.borderThin
        radius: Theme.radiusMedium
        
        Column {
            anchors.fill: parent
            anchors.margins: Theme.spacingSmall
            spacing: Theme.spacingTiny
            
            // 传感器名称和状态
            Row {
                width: parent.width
                height: 20
                
                Text {
                    text: sensorName
                    font.pixelSize: Theme.fontSizeSmall
                    color: Theme.textTertiary
                }
                
                Item { width: parent.width - 70 - 76 }
                
                Row {
                    spacing: Theme.spacingSmall
                    anchors.verticalCenter: parent.verticalCenter
                    
                    Rectangle {
                        width: 8
                        height: 8
                        radius: 4
                        color: isOnline ? Theme.successColor : "#62748e"
                        opacity: 0.93
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    
                    Text {
                        text: sensorTr.t("calibration.calibrating")
                        font.pixelSize: Theme.fontSizeTiny
                        color: "#62748e"
                        anchors.verticalCenter: parent.verticalCenter
                    }
                }
            }
            
            Column {
                width: parent.width
                spacing: Theme.spacingSmall
                
                // AD值
                Column {
                    width: parent.width
                    spacing: 4
                    
                    Text {
                        text: sensorTr.t("calibration.adValue")
                        font.pixelSize: Theme.fontSizeTiny
                        color: "#62748e"
                    }
                    
                    Text {
                        text: adValue
                        font.pixelSize: Theme.fontSizeLarge
                        font.family: Theme.fontFamilyMono
                        color: Theme.textAccent
                    }
                }
                
                Rectangle {
                    width: parent.width
                    height: 1
                    color: Theme.darkBorder
                }
                
                // 计算值
                Column {
                    width: parent.width
                    spacing: 4
                    
                    Text {
                        text: unit === sensorTr.t("monitoring.unit.ton") ? sensorTr.t("calibration.physicalValue") : (unit === "°" ? sensorTr.t("calibration.angleValue") : sensorTr.t("calibration.radiusValue"))
                        font.pixelSize: Theme.fontSizeTiny
                        color: "#62748e"
                    }
                    
                    Row {
                        spacing: 0
                        
                        Text {
                            text: calculatedValue
                            font.pixelSize: Theme.fontSizeLarge
                            font.family: Theme.fontFamilyMono
                            color: "#05df72"
                        }
                        
                        Text {
                            text: unit
                            font.pixelSize: Theme.fontSizeMedium
                            font.family: Theme.fontFamilyMono
                            color: "#05df72"
                            anchors.baseline: parent.children[0].baseline
                        }
                    }
                }
            }
        }
    }
}
