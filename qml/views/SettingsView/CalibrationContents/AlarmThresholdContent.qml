// AlarmThresholdContent.qml - 报警阈值设置内容
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
            
            // 力矩报警阈值
            Rectangle {
                width: parent.width
                height: 246
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
                            color: Theme.dangerColor
                            anchors.verticalCenter: parent.verticalCenter
                        }
                        
                        Text {
                            text: "力矩报警阈值"
                            font.pixelSize: Theme.fontSizeLarge
                            color: Theme.textPrimary
                            anchors.verticalCenter: parent.verticalCenter
                        }
                    }
                    
                    Column {
                        width: parent.width
                        spacing: Theme.spacingSmall
                        
                        Text {
                            text: "设置力矩百分比报警阈值"
                            font.pixelSize: Theme.fontSizeSmall
                            color: Theme.textSecondary
                        }
                        
                        Row {
                            width: parent.width
                            spacing: Theme.spacingMedium
                            
                            Column {
                                width: (parent.width - Theme.spacingMedium) / 2
                                spacing: Theme.spacingSmall
                                
                                Text {
                                    text: "预警阈值（%）"
                                    font.pixelSize: Theme.fontSizeSmall
                                    color: Theme.textSecondary
                                }
                                
                                CustomInput {
                                    id: warningThresholdField
                                    width: parent.width
                                    height: 42
                                    text: "80"
                                    font.pixelSize: Theme.fontSizeMedium
                                    font.family: Theme.fontFamilyMono
                                    color: Theme.warningColor
                                    
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
                                    text: "危险阈值（%）"
                                    font.pixelSize: Theme.fontSizeSmall
                                    color: Theme.textSecondary
                                }
                                
                                CustomInput {
                                    id: dangerThresholdField
                                    width: parent.width
                                    height: 42
                                    text: "100"
                                    font.pixelSize: Theme.fontSizeMedium
                                    font.family: Theme.fontFamilyMono
                                    color: Theme.dangerColor
                                    
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
                            text: "当力矩百分比超过预警阈值时显示黄色警告，超过危险阈值时显示红色报警"
                            font.pixelSize: Theme.fontSizeTiny
                            color: "#62748e"
                            wrapMode: Text.WordWrap
                            width: parent.width
                        }
                    }
                }
            }
            
            // 载荷报警阈值
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
                            color: "#ad46ff"
                            anchors.verticalCenter: parent.verticalCenter
                        }
                        
                        Text {
                            text: "载荷报警阈值"
                            font.pixelSize: Theme.fontSizeLarge
                            color: Theme.textPrimary
                            anchors.verticalCenter: parent.verticalCenter
                        }
                    }
                    
                    Column {
                        width: parent.width
                        spacing: Theme.spacingSmall
                        
                        Text {
                            text: "最大载荷限制（吨）"
                            font.pixelSize: Theme.fontSizeSmall
                            color: Theme.textSecondary
                        }
                        
                        CustomInput {
                            id: maxLoadField
                            width: parent.width
                            height: 42
                            text: "50.0"
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
                        
                        Text {
                            text: "当实际载荷超过此值时触发报警"
                            font.pixelSize: Theme.fontSizeTiny
                            color: "#62748e"
                            wrapMode: Text.WordWrap
                            width: parent.width
                        }
                    }
                }
            }
            
            // 角度报警阈值
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
                            text: "角度报警阈值"
                            font.pixelSize: Theme.fontSizeLarge
                            color: Theme.textPrimary
                            anchors.verticalCenter: parent.verticalCenter
                        }
                    }
                    
                    Column {
                        width: parent.width
                        spacing: Theme.spacingSmall
                        
                        Text {
                            text: "最大角度限制（°）"
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
                        
                        Text {
                            text: "当臂架角度超过此值时触发报警"
                            font.pixelSize: Theme.fontSizeTiny
                            color: "#62748e"
                            wrapMode: Text.WordWrap
                            width: parent.width
                        }
                    }
                }
            }
            
            // 报警说明
            Rectangle {
                width: parent.width
                height: 206
                color: "#2d0d0f"
                border.color: Theme.dangerColor
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
                        source: "../../../assets/images/icon-alert.png"
                    }
                    
                    Column {
                        width: parent.width - Theme.iconSizeMedium - Theme.spacingSmall
                        spacing: Theme.spacingSmall
                        
                        Text {
                            text: "报警阈值设置说明"
                            font.pixelSize: Theme.fontSizeMedium
                            color: Theme.dangerLight
                        }
                        
                        Column {
                            width: parent.width
                            spacing: 4
                            
                            Text {
                                text: "• 力矩报警是最重要的安全保护，建议预警阈值设为80%"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                            
                            Text {
                                text: "• 危险阈值达到100%时系统将强制停止作业"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                            
                            Text {
                                text: "• 载荷阈值应根据起重机额定载荷设置"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                            
                            Text {
                                text: "• 角度阈值应考虑起重机的最大工作角度"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                            
                            Text {
                                text: "• 报警触发后会记录到报警记录中，可在报警记录页面查看"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                            
                            Text {
                                text: "• 修改阈值后需要保存设置才能生效"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                        }
                    }
                }
            }
        }
    }
}
