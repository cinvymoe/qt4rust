// LoadCalibrationContent.qml - 载荷传感器校准内容
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qt.rust.demo
import "../../../styles"
import "../../../components/controls"
import "../../../components/dialogs"

Item {
    id: root

    
    // 暴露保存和重置函数给外部调用
    function saveCalibration() {
        return loadViewModel.save_calibration()
    }
    
    function resetToDefault() {
        loadViewModel.reset_to_default()
    }
    
    // 绑定 LoadCalibrationViewModel
    LoadCalibrationViewModel {
        id: loadViewModel
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
            
            // 标定倍率设置
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
                            color: "#ad46ff"
                            anchors.verticalCenter: parent.verticalCenter
                        }
                        
                        Text {
                            text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("calibration.multiplier") }
                            font.pixelSize: Theme.fontSizeLarge
                            color: Theme.textPrimary
                            anchors.verticalCenter: parent.verticalCenter
                        }
                    }
                    
                    Column {
                        width: parent.width
                        spacing: Theme.spacingSmall
                        
                        Text {
                            text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("calibration.selectMultiplier") }
                            font.pixelSize: Theme.fontSizeSmall
                            color: Theme.textSecondary
                        }
                        
                        // 倍率选择按钮
                        Grid {
                            width: parent.width
                            columns: 4
                            spacing: Theme.spacingSmall
                            
                            Repeater {
                                model: [
                                    {value: 1, label: "标准倍率"},
                                    {value: 2, label: "2倍放大"},
                                    {value: 4, label: "4倍放大"},
                                    {value: 0, label: "自定义"}
                                ]
                                
                                Rectangle {
                                    width: (parent.width - 3 * Theme.spacingSmall) / 4
                                    height: 92
                                    color: {
                                        if (modelData.value === 0) {
                                            // 自定义：检查是否是非标准值
                                            return (loadViewModel.calibration_multiplier !== 1 && 
                                                    loadViewModel.calibration_multiplier !== 2 && 
                                                    loadViewModel.calibration_multiplier !== 4) ? "#9810fa" : "#314158"
                                        } else {
                                            return loadViewModel.calibration_multiplier === modelData.value ? "#9810fa" : "#314158"
                                        }
                                    }
                                    border.color: {
                                        if (modelData.value === 0) {
                                            return (loadViewModel.calibration_multiplier !== 1 && 
                                                    loadViewModel.calibration_multiplier !== 2 && 
                                                    loadViewModel.calibration_multiplier !== 4) ? "#ad46ff" : "#45556c"
                                        } else {
                                            return loadViewModel.calibration_multiplier === modelData.value ? "#ad46ff" : "#45556c"
                                        }
                                    }
                                    border.width: 2
                                    radius: Theme.radiusMedium
                                    
                                    Column {
                                        anchors.centerIn: parent
                                        spacing: 4
                                        
                                        Text {
                                            text: {
                                                if (modelData.value === 0) {
                                                    // 自定义：显示当前值或"自定义"
                                                    if (loadViewModel.calibration_multiplier !== 1 && 
                                                        loadViewModel.calibration_multiplier !== 2 && 
                                                        loadViewModel.calibration_multiplier !== 4) {
                                                        return loadViewModel.calibration_multiplier + "x"
                                                    } else {
                                                        return "自定义"
                                                    }
                                                } else {
                                                    return modelData.value + "x"
                                                }
                                            }
                                            font.pixelSize: modelData.value === 0 ? Theme.fontSizeLarge : Theme.fontSizeXXLarge
                                            font.family: Theme.fontFamilyMono
                                            color: {
                                                if (modelData.value === 0) {
                                                    return (loadViewModel.calibration_multiplier !== 1 && 
                                                            loadViewModel.calibration_multiplier !== 2 && 
                                                            loadViewModel.calibration_multiplier !== 4) ? Theme.textPrimary : Theme.textSecondary
                                                } else {
                                                    return loadViewModel.calibration_multiplier === modelData.value ? Theme.textPrimary : Theme.textSecondary
                                                }
                                            }
                                            anchors.horizontalCenter: parent.horizontalCenter
                                        }
                                        
                                        Text {
                                            text: modelData.label
                                            font.pixelSize: Theme.fontSizeTiny
                                            color: {
                                                if (modelData.value === 0) {
                                                    return (loadViewModel.calibration_multiplier !== 1 && 
                                                            loadViewModel.calibration_multiplier !== 2 && 
                                                            loadViewModel.calibration_multiplier !== 4) ? Theme.textPrimary : Theme.textSecondary
                                                } else {
                                                    return loadViewModel.calibration_multiplier === modelData.value ? Theme.textPrimary : Theme.textSecondary
                                                }
                                            }
                                            anchors.horizontalCenter: parent.horizontalCenter
                                        }
                                    }
                                    
                                    MouseArea {
                                        anchors.fill: parent
                                        onClicked: {
                                            if (modelData.value === 0) {
                                                // 自定义倍率，弹出对话框
                                                customMultiplierDialog.open()
                                            } else {
                                                loadViewModel.set_multiplier(modelData.value)
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        
                        Text {
                            text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("calibration.multiplierDesc") }
                            font.pixelSize: Theme.fontSizeTiny
                            color: "#62748e"
                            wrapMode: Text.WordWrap
                            width: parent.width
                        }
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
                            text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("calibration.loadTitle") || "两点标定设置" }
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
                        CalibrationPoint {
                            width: parent.width
                            pointNumber: 1
                            adValue: loadViewModel.point1_ad.toFixed(2)
                            weightValue: loadViewModel.point1_weight.toFixed(2)
                            onAdValueEdited: function(newValue) {
                                var value = parseFloat(newValue)
                                if (!isNaN(value)) {
                                    loadViewModel.point1_ad = value
                                }
                            }
                            onWeightValueEdited: function(newValue) {
                                var value = parseFloat(newValue)
                                if (!isNaN(value)) {
                                    loadViewModel.point1_weight = value
                                }
                            }
                        }
                        
                        // 标定点 2
                        CalibrationPoint {
                            width: parent.width
                            pointNumber: 2
                            adValue: loadViewModel.point2_ad.toFixed(2)
                            weightValue: loadViewModel.point2_weight.toFixed(2)
                            onAdValueEdited: function(newValue) {
                                var value = parseFloat(newValue)
                                if (!isNaN(value)) {
                                    loadViewModel.point2_ad = value
                                }
                            }
                            onWeightValueEdited: function(newValue) {
                                var value = parseFloat(newValue)
                                if (!isNaN(value)) {
                                    loadViewModel.point2_weight = value
                                }
                            }
                        }
                    }
                }
            }
            
            // 校准说明
            Rectangle {
                width: parent.width
                height: 206
                color: "#162456"
                border.color: "#155dfc"
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
                        source: "../../../assets/images/icon-chart.svg"
                    }
                    
                    Column {
                        width: parent.width - Theme.iconSizeMedium - Theme.spacingSmall
                        spacing: Theme.spacingSmall
                        
                        Text {
                            text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("calibration.loadNote1") }
                            font.pixelSize: Theme.fontSizeMedium
                            color: Theme.textAccent
                        }

                        Column {
                            width: parent.width
                            spacing: 4

                            Text {
                                text: { const _ = TranslationBridge.locale_version; return "• " + TranslationBridge.translate("calibration.loadNote1") }
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }

                            Text {
                                text: { const _ = TranslationBridge.locale_version; return "• " + TranslationBridge.translate("calibration.loadNote2") }
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }

                            Text {
                                text: { const _ = TranslationBridge.locale_version; return "• " + TranslationBridge.translate("calibration.loadNote3") }
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }

                            Text {
                                text: { const _ = TranslationBridge.locale_version; return "• " + TranslationBridge.translate("calibration.loadNote4") }
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }

                            Text {
                                text: { const _ = TranslationBridge.locale_version; return "• " + TranslationBridge.translate("calibration.loadNote5") }
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }

                            Text {
                                text: { const _ = TranslationBridge.locale_version; return "• " + TranslationBridge.translate("calibration.loadNote6") }
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                        }
                    }
                }
            }
        }
    }
    
    // 自定义倍率选择对话框
    CustomMultiplierDialog {
        id: customMultiplierDialog
        currentMultiplier: loadViewModel.calibration_multiplier
        
        onMultiplierSelected: function(multiplier) {
            loadViewModel.set_multiplier(multiplier)
        }
    }

    // 标定点组件
    component CalibrationPoint: Rectangle {
        property int pointNumber: 1
        property string adValue: "0"
        property string weightValue: "0"
        signal adValueEdited(string newValue)
        signal weightValueEdited(string newValue)
        
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
                color: "#155dfc"
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
                        text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("calibration.adValue") }
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
                        text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("calibration.physicalValue") }
                        font.pixelSize: Theme.fontSizeSmall
                        color: Theme.textSecondary
                    }
                    
                    CustomInput {
                        id: weightValueField
                        width: parent.width
                        height: 42
                        text: weightValue
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
                            weightValueEdited(text)
                        }
                    }
                }
            }
        }
    }
}
