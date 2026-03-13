// AngleCalibrationContent.qml - 角度传感器校准内容
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../../../styles"
import "../../../components/controls"

Item {
    id: root
    
    Flickable {
        id: flickable
        anchors.fill: parent
        contentHeight: calibrationContent.height + Theme.spacingMedium * 2
        clip: true
        
        Column {
            id: calibrationContent
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.margins: Theme.spacingLarge
            spacing: Theme.spacingLarge
            
            // 角度范围设置
            Rectangle {
                width: parent.width
                height: 206
                color: Theme.darkSurface
                border.color: Theme.darkBorder
                border.width: Theme.borderThin
                radius: Theme.radiusMedium
                
                Column {
                    anchors.fill: parent
                    anchors.margins: 25
                    spacing: Theme.spacingMedium
                    
                    // 标题
                    Row {
                        width: parent.width
                        height: 28
                        spacing: Theme.spacingSmall
                        
                        Rectangle {
                            width: 4
                            height: 24
                            color: "#f0b100"
                            anchors.verticalCenter: parent.verticalCenter
                        }
                        
                        Text {
                            text: "角度测量范围"
                            font.pixelSize: Theme.fontSizeLarge
                            color: Theme.textPrimary
                            anchors.verticalCenter: parent.verticalCenter
                        }
                    }
                    
                    Row {
                        width: parent.width
                        spacing: Theme.spacingMedium
                        
                        Column {
                            width: (parent.width - Theme.spacingMedium) / 2
                            spacing: Theme.spacingSmall
                            
                            Text {
                                text: "最小角度（°）"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                            
                            CustomInput {
                                id: minAngleField
                                width: parent.width
                                height: 42
                                text: "0"
                                font.pixelSize: Theme.fontSizeMedium
                                font.family: Theme.fontFamilyMono
                                color: Theme.textPrimary
                                
                                background: Rectangle {
                                    color: "#314158"
                                    border.color: "#45556c"
                                    border.width: Theme.borderThin
                                    radius: Theme.radiusSmall
                                }
                            }
                        }
                        
                        Column {
                            width: (parent.width - Theme.spacingMedium) / 2
                            spacing: Theme.spacingSmall
                            
                            Text {
                                text: "最大角度（°）"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                            
                            CustomInput {
                                id: maxAngleField
                                width: parent.width
                                height: 42
                                text: "85"
                                font.pixelSize: Theme.fontSizeMedium
                                font.family: Theme.fontFamilyMono
                                color: Theme.textPrimary
                                
                                background: Rectangle {
                                    color: "#314158"
                                    border.color: "#45556c"
                                    border.width: Theme.borderThin
                                    radius: Theme.radiusSmall
                                }
                            }
                        }
                    }
                    
                    Text {
                        text: "设置角度传感器的有效测量范围，超出范围将触发报警"
                        font.pixelSize: Theme.fontSizeTiny
                        color: "#62748e"
                        wrapMode: Text.WordWrap
                        width: parent.width
                    }
                }
            }
            
            // 两点标定设置
            Rectangle {
                width: parent.width
                height: 286
                color: Theme.darkSurface
                border.color: Theme.darkBorder
                border.width: Theme.borderThin
                radius: Theme.radiusMedium
                
                Column {
                    anchors.fill: parent
                    anchors.margins: 25
                    spacing: Theme.spacingMedium
                    
                    // 标题
                    Row {
                        width: parent.width
                        height: 28
                        spacing: Theme.spacingSmall
                        
                        Rectangle {
                            width: 4
                            height: 24
                            color: Theme.darkAccent
                            anchors.verticalCenter: parent.verticalCenter
                        }
                        
                        Text {
                            text: "两点标定设置"
                            font.pixelSize: Theme.fontSizeLarge
                            font.weight: Font.Medium
                            color: Theme.textPrimary
                            anchors.verticalCenter: parent.verticalCenter
                        }
                    }
                    
                    // 标定点列表
                    Column {
                        width: parent.width
                        spacing: Theme.spacingSmall
                        
                        // 标定点 1
                        AngleCalibrationPoint {
                            width: parent.width
                            pointNumber: 1
                            adValue: "0"
                            angleValue: "0"
                        }
                        
                        // 标定点 2
                        AngleCalibrationPoint {
                            width: parent.width
                            pointNumber: 2
                            adValue: "20000"
                            angleValue: "85"
                        }
                    }
                }
            }
            
            // 校准说明
            Rectangle {
                width: parent.width
                height: 186
                color: "#2d2416"
                border.color: "#f0b100"
                border.width: Theme.borderThin
                radius: Theme.radiusMedium
                
                Row {
                    anchors.fill: parent
                    anchors.margins: 17
                    spacing: Theme.spacingSmall
                    
                    Image {
                        width: Theme.iconSizeMedium
                        height: Theme.iconSizeMedium
                        anchors.top: parent.top
                        anchors.topMargin: 4
                        source: "../../../assets/images/icon-angle.png"
                    }
                    
                    Column {
                        width: parent.width - Theme.iconSizeMedium - Theme.spacingSmall
                        spacing: Theme.spacingSmall
                        
                        Text {
                            text: "角度传感器标定说明"
                            font.pixelSize: Theme.fontSizeMedium
                            color: Theme.warningColor
                        }
                        
                        Column {
                            width: parent.width
                            spacing: 4
                            
                            Text {
                                text: "• 至少需要2个标定点，建议在0°和最大角度处标定"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                            
                            Text {
                                text: "• 使用精密角度测量仪器进行标定"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                            
                            Text {
                                text: "• 标定时确保臂架处于稳定状态"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                            
                            Text {
                                text: "• 角度传感器对温度敏感，建议在工作温度下标定"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                            
                            Text {
                                text: "• 系统使用线性插值算法计算两点之间的角度值"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                        }
                    }
                }
            }
        }
    }
    
    // 角度标定点组件
    component AngleCalibrationPoint: Rectangle {
        property int pointNumber: 1
        property string adValue: "0"
        property string angleValue: "0"
        
        height: 104
        color: Theme.darkBackground
        border.color: "#45556c"
        border.width: Theme.borderThin
        radius: Theme.radiusMedium
        
        Row {
            anchors.fill: parent
            anchors.margins: 17
            spacing: Theme.spacingMedium
            
            // 序号
            Rectangle {
                width: 32
                height: 32
                radius: 16
                color: "#f0b100"
                anchors.verticalCenter: parent.verticalCenter
                
                Text {
                    anchors.centerIn: parent
                    text: pointNumber
                    font.pixelSize: Theme.fontSizeSmall
                    font.family: Theme.fontFamilyMono
                    color: Theme.textPrimary
                }
            }
            
            // 输入字段
            Row {
                width: parent.width - 32 - Theme.spacingMedium * 2
                spacing: Theme.spacingMedium
                anchors.verticalCenter: parent.verticalCenter
                
                Column {
                    width: (parent.width - Theme.spacingMedium) / 2
                    spacing: Theme.spacingSmall
                    
                    Text {
                        text: "AD值"
                        font.pixelSize: Theme.fontSizeSmall
                        color: Theme.textSecondary
                    }
                    
                    CustomInput {
                        id: adValueField
                        width: parent.width
                        height: 42
                        text: adValue
                        font.pixelSize: Theme.fontSizeMedium
                        font.family: Theme.fontFamilyMono
                        color: Theme.textPrimary
                        
                        background: Rectangle {
                            color: "#314158"
                            border.color: "#45556c"
                            border.width: Theme.borderThin
                            radius: Theme.radiusSmall
                        }
                    }
                }
                
                Column {
                    width: (parent.width - Theme.spacingMedium) / 2
                    spacing: Theme.spacingSmall
                    
                    Text {
                        text: "角度（°）"
                        font.pixelSize: Theme.fontSizeSmall
                        color: Theme.textSecondary
                    }
                    
                    CustomInput {
                        id: angleValueField
                        width: parent.width
                        height: 42
                        text: angleValue
                        font.pixelSize: Theme.fontSizeMedium
                        font.family: Theme.fontFamilyMono
                        color: Theme.textPrimary
                        
                        background: Rectangle {
                            color: "#314158"
                            border.color: "#45556c"
                            border.width: Theme.borderThin
                            radius: Theme.radiusSmall
                        }
                    }
                }
            }
        }
    }
}
