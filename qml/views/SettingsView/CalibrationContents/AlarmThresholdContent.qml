// AlarmThresholdContent.qml - 报警阈值设置内容
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qt.rust.demo
import "../../../styles"
import "../../../components/controls"

Item {
    id: root
    
    // 暴露保存和重置函数给外部调用
    function saveCalibration() {
        return alarmViewModel.save_thresholds()
    }
    
    function resetToDefault() {
        alarmViewModel.reset_to_default()
    }
    
    // 绑定 AlarmThresholdViewModel
    AlarmThresholdViewModel {
        id: alarmViewModel
    }
    
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
                                    text: alarmViewModel.moment_warning_threshold.toFixed(1)
                                    font.pixelSize: Theme.fontSizeMedium
                                    font.family: Theme.fontFamilyMono
                                    color: Theme.warningColor
                                    
                                    background: Rectangle {
                                        color: "#314158"
                                        border.color: "#45556c"
                                        border.width: Theme.borderThin
                                        radius: Theme.radiusSmall
                                    }
                                    
                                    onEditingFinished: {
                                        var value = parseFloat(text)
                                        if (!isNaN(value)) {
                                            alarmViewModel.moment_warning_threshold = value
                                        }
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
                                    text: alarmViewModel.moment_danger_threshold.toFixed(1)
                                    font.pixelSize: Theme.fontSizeMedium
                                    font.family: Theme.fontFamilyMono
                                    color: Theme.dangerColor
                                    
                                    background: Rectangle {
                                        color: "#314158"
                                        border.color: "#45556c"
                                        border.width: Theme.borderThin
                                        radius: Theme.radiusSmall
                                    }
                                    
                                    onEditingFinished: {
                                        var value = parseFloat(text)
                                        if (!isNaN(value)) {
                                            alarmViewModel.moment_danger_threshold = value
                                        }
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
            
            // 角度报警阈值
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
                            color: Theme.warningColor
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
                            text: "设置吊臂角度报警范围"
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
                                    text: "角度下限（度）"
                                    font.pixelSize: Theme.fontSizeSmall
                                    color: Theme.textSecondary
                                }
                                
                                CustomInput {
                                    id: angleMinField
                                    width: parent.width
                                    height: 42
                                    text: alarmViewModel.min_angle.toFixed(1)
                                    font.pixelSize: Theme.fontSizeMedium
                                    font.family: Theme.fontFamilyMono
                                    color: Theme.warningColor
                                    
                                    background: Rectangle {
                                        color: "#314158"
                                        border.color: "#45556c"
                                        border.width: Theme.borderThin
                                        radius: Theme.radiusSmall
                                    }
                                    
                                    onEditingFinished: {
                                        var value = parseFloat(text)
                                        if (!isNaN(value)) {
                                            alarmViewModel.min_angle = value
                                        }
                                    }
                                }
                            }
                            
                            Column {
                                width: (parent.width - Theme.spacingMedium) / 2
                                spacing: Theme.spacingSmall
                                
                                Text {
                                    text: "角度上限（度）"
                                    font.pixelSize: Theme.fontSizeSmall
                                    color: Theme.textSecondary
                                }
                                
                                CustomInput {
                                    id: angleMaxField
                                    width: parent.width
                                    height: 42
                                    text: alarmViewModel.max_angle.toFixed(1)
                                    font.pixelSize: Theme.fontSizeMedium
                                    font.family: Theme.fontFamilyMono
                                    color: Theme.dangerColor
                                    
                                    background: Rectangle {
                                        color: "#314158"
                                        border.color: "#45556c"
                                        border.width: Theme.borderThin
                                        radius: Theme.radiusSmall
                                    }
                                    
                                    onEditingFinished: {
                                        var value = parseFloat(text)
                                        if (!isNaN(value)) {
                                            alarmViewModel.max_angle = value
                                        }
                                    }
                                }
                            }
                        }
                        
                        Text {
                            text: "当吊臂角度低于下限或高于上限时，系统将发出报警"
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
