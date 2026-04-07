// RadiusCalibrationContent.qml - 长度传感器校准内容
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
        return radiusViewModel.save_calibration()
    }
    
    function resetToDefault() {
        radiusViewModel.reset_to_default()
    }
    
    // 绑定 RadiusCalibrationViewModel
    RadiusCalibrationViewModel {
        id: radiusViewModel
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
                        RadiusCalibrationPoint {
                            width: parent.width
                            pointNumber: 1
                            adValue: radiusViewModel.point1_ad.toFixed(2)
                            lengthValue: radiusViewModel.point1_radius.toFixed(2)
                            onAdValueEdited: function(newValue) {
                                var value = parseFloat(newValue)
                                if (!isNaN(value)) {
                                    radiusViewModel.point1_ad = value
                                }
                            }
                            onLengthValueEdited: function(newValue) {
                                var value = parseFloat(newValue)
                                if (!isNaN(value)) {
                                    radiusViewModel.point1_radius = value
                                }
                            }
                        }
                        
                        // 标定点 2
                        RadiusCalibrationPoint {
                            width: parent.width
                            pointNumber: 2
                            adValue: radiusViewModel.point2_ad.toFixed(2)
                            lengthValue: radiusViewModel.point2_radius.toFixed(2)
                            onAdValueEdited: function(newValue) {
                                var value = parseFloat(newValue)
                                if (!isNaN(value)) {
                                    radiusViewModel.point2_ad = value
                                }
                            }
                            onLengthValueEdited: function(newValue) {
                                var value = parseFloat(newValue)
                                if (!isNaN(value)) {
                                    radiusViewModel.point2_radius = value
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
                color: "#0d2d1f"
                border.color: "#00c950"
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
                        source: "../../../assets/images/icon-radius.png"
                    }
                    
                    Column {
                        width: parent.width - Theme.iconSizeMedium - Theme.spacingSmall
                        spacing: Theme.spacingSmall
                        
                        Text {
                            text: "长度传感器标定说明"
                            font.pixelSize: Theme.fontSizeMedium
                            color: Theme.successColor
                        }
                        
                        Column {
                            width: parent.width
                            spacing: 4
                            
                            Text {
                                text: "• 至少需要2个标定点，建议在最小和最大工作长度处标定"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                            
                            Text {
                                text: "• 使用精确测量工具确定实际工作长度"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                            
                            Text {
                                text: "• 标定时确保臂架完全伸展或收缩到位"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                            
                            Text {
                                text: "• 长度传感器通常为拉绳式或角度换算，需定期检查"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                            
                            Text {
                                text: "• 系统使用线性插值算法计算两点之间的长度值"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                        }
                    }
                }
            }
        }
    }
    
    // 长度标定点组件
    component RadiusCalibrationPoint: Rectangle {
        property int pointNumber: 1
        property string adValue: "0"
        property string lengthValue: "0"
        signal adValueEdited(string newValue)
        signal lengthValueEdited(string newValue)
        
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
                color: "#00c950"
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
                        
                        onEditingFinished: {
                            adValueEdited(text)
                        }
                    }
                }
                
                Column {
                    width: (parent.width - Theme.spacingMedium) / 2
                    spacing: Theme.spacingSmall
                    
                    Text {
                        text: "长度（m）"
                        font.pixelSize: Theme.fontSizeSmall
                        color: Theme.textSecondary
                    }
                    
                    CustomInput {
                        id: lengthValueField
                        width: parent.width
                        height: 42
                        text: lengthValue
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
                            lengthValueEdited(text)
                        }
                    }
                }
            }
        }
    }
}