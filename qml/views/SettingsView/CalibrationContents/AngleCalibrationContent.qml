// AngleCalibrationContent.qml - 角度传感器校准内容
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
        return angleViewModel.save_calibration()
    }
    
    function resetToDefault() {
        angleViewModel.reset_to_default()
    }
    
    // 绑定 AngleCalibrationViewModel
    AngleCalibrationViewModel {
        id: angleViewModel
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
                            text: TranslationBridge.translate("calibration.angleTitle") || "两点标定设置"
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
                            adValue: angleViewModel.point1_ad.toFixed(2)
                            angleValue: angleViewModel.point1_angle.toFixed(2)
                            onAdValueEdited: function(newValue) {
                                var value = parseFloat(newValue)
                                if (!isNaN(value)) {
                                    angleViewModel.point1_ad = value
                                }
                            }
                            onAngleValueEdited: function(newValue) {
                                var value = parseFloat(newValue)
                                if (!isNaN(value)) {
                                    angleViewModel.point1_angle = value
                                }
                            }
                        }
                        
                        // 标定点 2
                        AngleCalibrationPoint {
                            width: parent.width
                            pointNumber: 2
                            adValue: angleViewModel.point2_ad.toFixed(2)
                            angleValue: angleViewModel.point2_angle.toFixed(2)
                            onAdValueEdited: function(newValue) {
                                var value = parseFloat(newValue)
                                if (!isNaN(value)) {
                                    angleViewModel.point2_ad = value
                                }
                            }
                            onAngleValueEdited: function(newValue) {
                                var value = parseFloat(newValue)
                                if (!isNaN(value)) {
                                    angleViewModel.point2_angle = value
                                }
                            }
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
                            text: TranslationBridge.translate("calibration.angleNote1")
                            font.pixelSize: Theme.fontSizeMedium
                            color: Theme.warningColor
                        }

                        Column {
                            width: parent.width
                            spacing: 4

                            Text {
                                text: "• " + TranslationBridge.translate("calibration.angleNote1")
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }

                            Text {
                                text: "• " + TranslationBridge.translate("calibration.angleNote2")
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }

                            Text {
                                text: "• " + TranslationBridge.translate("calibration.angleNote3")
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }

                            Text {
                                text: "• " + TranslationBridge.translate("calibration.angleNote4")
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }

                            Text {
                                text: "• " + TranslationBridge.translate("calibration.angleNote5")
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
        signal adValueEdited(string newValue)
        signal angleValueEdited(string newValue)
        
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
                        text: TranslationBridge.translate("calibration.adValue")
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
                        
                        onEditingFinished: {
                            adValueEdited(text)
                        }
                    }
                }
                
                Column {
                    width: (parent.width - Theme.spacingMedium) / 2
                    spacing: Theme.spacingSmall
                    
                    Text {
                        text: TranslationBridge.translate("calibration.angleValue")
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
                        
                        onEditingFinished: {
                            angleValueEdited(text)
                        }
                    }
                }
            }
        }
    }
}
