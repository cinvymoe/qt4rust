// CalibrationView.qml - 参数校准子页面
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../../styles"
import "CalibrationContents"

Item {
    id: calibrationView
    
    property int currentSensorTab: 0  // 0: 载荷, 1: 角度, 2: 半径, 3: 报警阈值
    
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
                        sensorName: "重量传感器"
                        adValue: "21611"
                        calculatedValue: "5.00"
                        unit: "吨"
                        isOnline: true
                    }
                    
                    // 角度传感器卡片
                    SensorDataCard {
                        width: parent.width - parent.leftPadding - parent.rightPadding
                        sensorName: "角度传感器"
                        adValue: "14309"
                        calculatedValue: "64.4"
                        unit: "°"
                        isOnline: true
                    }
                    
                    // 侧长传感器卡片
                    SensorDataCard {
                        width: parent.width - parent.leftPadding - parent.rightPadding
                        sensorName: "侧长传感器"
                        adValue: "17459"
                        calculatedValue: "17.46"
                        unit: "m"
                        isOnline: true
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
                                {text: "载荷传感器"},
                                {text: "角度传感器"},
                                {text: "半径传感器"},
                                {text: "报警阈值"}
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
                        anchors.fill: parent
                        visible: currentSensorTab === 0
                    }
                    
                    // 角度传感器校准
                    AngleCalibrationContent {
                        anchors.fill: parent
                        visible: currentSensorTab === 1
                    }
                    
                    // 半径传感器校准
                    RadiusCalibrationContent {
                        anchors.fill: parent
                        visible: currentSensorTab === 2
                    }
                    
                    // 报警阈值设置
                    AlarmThresholdContent {
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
                                    text: "恢复默认"
                                    font.pixelSize: Theme.fontSizeSmall
                                    color: Theme.textPrimary
                                    anchors.verticalCenter: parent.verticalCenter
                                }
                            }
                            
                            onClicked: console.log("恢复默认")
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
                                    text: "保存设置"
                                    font.pixelSize: Theme.fontSizeSmall
                                    color: Theme.textPrimary
                                    anchors.verticalCenter: parent.verticalCenter
                                }
                            }
                            
                            onClicked: console.log("保存设置")
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
                        text: "实时采集中"
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
                        text: "AD值"
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
                        text: unit === "吨" ? "计算重量值" : (unit === "°" ? "计算角度值" : "计算长度值")
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
