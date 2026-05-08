// AlarmThresholdContent.qml - 报警阈值设置内容
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qt.rust.demo
import "../../../styles"
import "../../../components/controls"

Item {
    id: root

    Tr { id: tr }

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
                            text: tr.t("alarmThreshold.momentTitle")
                            font.pixelSize: Theme.fontSizeLarge
                            color: Theme.textPrimary
                            anchors.verticalCenter: parent.verticalCenter
                        }
                    }
                    
                    Column {
                        width: parent.width
                        spacing: Theme.spacingSmall
                        
                                Text {
                                    text: tr.t("alarmThreshold.mode")
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
                                    text: tr.t("alarmThreshold.warningPercent")
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
                                    text: tr.t("alarmThreshold.dangerPercent")
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
                            text: tr.t("alarmThreshold.momentNote")
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
                            text: tr.t("alarmThreshold.angleTitle")
                            font.pixelSize: Theme.fontSizeLarge
                            color: Theme.textPrimary
                            anchors.verticalCenter: parent.verticalCenter
                        }
                    }
                    
                    Column {
                        width: parent.width
                        spacing: Theme.spacingSmall
                        
                        Text {
                            text: tr.t("alarmThreshold.angleDesc")
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
                                    text: tr.t("alarmThreshold.angleLower")
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
                                    text: tr.t("alarmThreshold.angleUpper")
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
                            text: tr.t("alarmThreshold.angleNote")
                            font.pixelSize: Theme.fontSizeTiny
                            color: "#62748e"
                            wrapMode: Text.WordWrap
                            width: parent.width
                        }
                    }
                }
            }
            
            // 主钩勾头开关报警
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
                            text: tr.t("alarmThreshold.mainHook")
                            font.pixelSize: Theme.fontSizeLarge
                            color: Theme.textPrimary
                            anchors.verticalCenter: parent.verticalCenter
                        }
                    }
                    
                    Column {
                        width: parent.width
                        spacing: Theme.spacingSmall
                        
                        Text {
                            text: tr.t("alarmThreshold.mainHookDesc")
                            font.pixelSize: Theme.fontSizeSmall
                            color: Theme.textSecondary
                        }
                        
                        Row {
                            width: parent.width
                            spacing: Theme.spacingMedium
                            
                            // 常开/常闭 二选一
                            Column {
                                width: parent.width
                                spacing: Theme.spacingSmall
                                
                                Text {
                                    text: tr.t("alarmThreshold.mode")
                                    font.pixelSize: Theme.fontSizeSmall
                                    color: Theme.textSecondary
                                }
                                 

                                Row {
                                    spacing: Theme.spacingMedium
                                    
                                    // 常开
                                    RadioButton {
                                        id: mainHookNormallyOpenRadio
                                        text: tr.t("alarmThreshold.modeNO")
                                        checked: alarmViewModel.main_hook_mode === 1
                                        font.pixelSize: Theme.fontSizeMedium
                                        indicator: Rectangle {
                                            width: 18
                                            height: 18
                                            anchors.verticalCenter: parent.verticalCenter
                                            radius: 9
                                            border.color: mainHookNormallyOpenRadio.checked ? Theme.dangerColor : "#45556c"
                                            border.width: 2
                                            Rectangle {
                                                anchors.centerIn: parent
                                                width: 10
                                                height: 10
                                                radius: 5
                                                color: mainHookNormallyOpenRadio.checked ? Theme.dangerColor : "transparent"
                                            }
                                        }
                                        contentItem: Text {
                                            leftPadding: 26
                                            text: mainHookNormallyOpenRadio.text
                                            font.pixelSize: Theme.fontSizeMedium
                                            color: mainHookNormallyOpenRadio.checked ? Theme.dangerColor : Theme.textSecondary
                                            verticalAlignment: Text.AlignVCenter
                                        }
                                        onClicked: {
                                            alarmViewModel.main_hook_mode = 1
                                        }
                                    }
                                    
                                    // 常闭
                                    RadioButton {
                                        id: mainHookNormallyClosedRadio
                                        text: tr.t("alarmThreshold.modeNC")
                                        checked: alarmViewModel.main_hook_mode === 2
                                        font.pixelSize: Theme.fontSizeMedium
                                        indicator: Rectangle {
                                            width: 18
                                            height: 18
                                            anchors.verticalCenter: parent.verticalCenter
                                            radius: 9
                                            border.color: mainHookNormallyClosedRadio.checked ? Theme.dangerColor : "#45556c"
                                            border.width: 2
                                            Rectangle {
                                                anchors.centerIn: parent
                                                width: 10
                                                height: 10
                                                radius: 5
                                                color: mainHookNormallyClosedRadio.checked ? Theme.dangerColor : "transparent"
                                            }
                                        }
                                        contentItem: Text {
                                            leftPadding: 26
                                            text: mainHookNormallyClosedRadio.text
                                            font.pixelSize: Theme.fontSizeMedium
                                            color: mainHookNormallyClosedRadio.checked ? Theme.dangerColor : Theme.textSecondary
                                            verticalAlignment: Text.AlignVCenter
                                        }
                                        onClicked: {
                                            alarmViewModel.main_hook_mode = 2
                                        }
                                    }
                                }
                            }
                        }
                        
                        Text {
                            text: tr.t("alarmThreshold.mainHookNote")
                            font.pixelSize: Theme.fontSizeTiny
                            color: "#62748e"
                            wrapMode: Text.WordWrap
                            width: parent.width
                        }
                    }
                }
            }
            
            // 副钩勾头开关报警
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
                            text: tr.t("alarmThreshold.subHook")
                            font.pixelSize: Theme.fontSizeLarge
                            color: Theme.textPrimary
                            anchors.verticalCenter: parent.verticalCenter
                        }
                    }
                    
                    Column {
                        width: parent.width
                        spacing: Theme.spacingSmall
                        
                        Text {
                            text: tr.t("alarmThreshold.subHookDesc")
                            font.pixelSize: Theme.fontSizeSmall
                            color: Theme.textSecondary
                        }
                        
                        Row {
                            width: parent.width
                            spacing: Theme.spacingMedium
                            
                            // 常开/常闭 二选一
                            Column {
                                width: parent.width
                                spacing: Theme.spacingSmall
                                
                                Text {
                                    text: tr.t("alarmThreshold.mode")
                                    font.pixelSize: Theme.fontSizeSmall
                                    color: Theme.textSecondary
                                }
                                 

                                Row {
                                    spacing: Theme.spacingMedium
                                    
                                    // 常开
                                    RadioButton {
                                        id: auxHookNormallyOpenRadio
                                        text: tr.t("alarmThreshold.modeNO")
                                        checked: alarmViewModel.aux_hook_mode === 1
                                        font.pixelSize: Theme.fontSizeMedium
                                        indicator: Rectangle {
                                            width: 18
                                            height: 18
                                            anchors.verticalCenter: parent.verticalCenter
                                            radius: 9
                                            border.color: auxHookNormallyOpenRadio.checked ? Theme.dangerColor : "#45556c"
                                            border.width: 2
                                            Rectangle {
                                                anchors.centerIn: parent
                                                width: 10
                                                height: 10
                                                radius: 5
                                                color: auxHookNormallyOpenRadio.checked ? Theme.dangerColor : "transparent"
                                            }
                                        }
                                        contentItem: Text {
                                            leftPadding: 26
                                            text: auxHookNormallyOpenRadio.text
                                            font.pixelSize: Theme.fontSizeMedium
                                            color: auxHookNormallyOpenRadio.checked ? Theme.dangerColor : Theme.textSecondary
                                            verticalAlignment: Text.AlignVCenter
                                        }
                                        onClicked: {
                                            alarmViewModel.aux_hook_mode = 1
                                        }
                                    }
                                    
                                    // 常闭
                                    RadioButton {
                                        id: auxHookNormallyClosedRadio
                                        text: tr.t("alarmThreshold.modeNC")
                                        checked: alarmViewModel.aux_hook_mode === 2
                                        font.pixelSize: Theme.fontSizeMedium
                                        indicator: Rectangle {
                                            width: 18
                                            height: 18
                                            anchors.verticalCenter: parent.verticalCenter
                                            radius: 9
                                            border.color: auxHookNormallyClosedRadio.checked ? Theme.dangerColor : "#45556c"
                                            border.width: 2
                                            Rectangle {
                                                anchors.centerIn: parent
                                                width: 10
                                                height: 10
                                                radius: 5
                                                color: auxHookNormallyClosedRadio.checked ? Theme.dangerColor : "transparent"
                                            }
                                        }
                                        contentItem: Text {
                                            leftPadding: 26
                                            text: auxHookNormallyClosedRadio.text
                                            font.pixelSize: Theme.fontSizeMedium
                                            color: auxHookNormallyClosedRadio.checked ? Theme.dangerColor : Theme.textSecondary
                                            verticalAlignment: Text.AlignVCenter
                                        }
                                        onClicked: {
                                            alarmViewModel.aux_hook_mode = 2
                                        }
                                    }
                                }
                            }
                        }
                        
                        Text {
                            text: tr.t("alarmThreshold.subHookNote")
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
                            text: tr.t("alarmThreshold.notes")
                            font.pixelSize: Theme.fontSizeMedium
                            color: Theme.dangerLight
                        }
                        
                        Column {
                            width: parent.width
                            spacing: 4
                            
                            Text {
                                text: "• " + tr.t("alarmThreshold.note1")
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }

                            Text {
                                text: "• " + tr.t("alarmThreshold.note2")
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }

                            Text {
                                text: "• " + tr.t("alarmThreshold.note3")
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }

                            Text {
                                text: "• " + tr.t("alarmThreshold.note4")
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
