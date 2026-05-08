// CustomMultiplierDialog.qml - 自定义倍率选择对话框
import qt.rust.demo
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../../styles"

Dialog {
    id: root
    
    TranslationBridge { id: tr }
    
    // 对外属性
    property real currentMultiplier: 1.0
    
    // 信号
    signal multiplierSelected(real multiplier)
    
    modal: true
    closePolicy: Popup.CloseOnEscape
    width: 500
    height: 520
    
    // 居中显示
    x: (parent.width - width) / 2
    y: (parent.height - height) / 2
    
    // 背景
    background: Rectangle {
        color: Theme.darkSurface
        radius: Theme.radiusMedium
        border.color: Theme.darkBorder
        border.width: Theme.borderNormal
    }
    
    // 标题区域
    header: Item {
        width: parent.width
        height: 64
        
        // 关闭按钮
        Button {
            id: closeButton
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.margins: Theme.spacingMedium
            width: 32
            height: 32
            
            background: Rectangle {
                color: "transparent"
                radius: Theme.radiusSmall
            }
            
            contentItem: Text {
                text: "✕"
                font.pixelSize: Theme.fontSizeLarge
                color: Theme.textSecondary
                horizontalAlignment: Text.AlignHCenter
                verticalAlignment: Text.AlignVCenter
            }
            
            onClicked: root.close()
        }
        
        Column {
            anchors.left: parent.left
            anchors.top: parent.top
            anchors.margins: Theme.spacingMedium
            spacing: Theme.spacingTiny
            
            // 主标题
            Text {
                text: tr.translate("dialog.selectMultiplier")
                font.pixelSize: Theme.fontSizeLarge
                font.family: Theme.fontFamilyDefault
                font.bold: true
                color: Theme.textPrimary
            }
            
            // 副标题
            Text {
                text: tr.translate("dialog.selectMultiplierDesc")
                font.pixelSize: Theme.fontSizeSmall
                font.family: Theme.fontFamilyDefault
                color: Theme.textSecondary
            }
        }
    }
    
    // 内容区域
    contentItem: Item {
        // 倍率选项网格（可滚动）
        Flickable {
            anchors.fill: parent
            anchors.margins: Theme.spacingMedium
            contentHeight: multiplierGrid.height
            clip: true
            
            Grid {
                id: multiplierGrid
                width: parent.width
                columns: 5
                spacing: Theme.spacingSmall
                
                Repeater {
                    model: ListModel {
                        id: multiplierOptions
                        Component.onCompleted: {
                            // 生成 0.5 到 10 的选项，步长 0.5
                            for (var i = 0.5; i <= 10; i += 0.5) {
                                append({value: i, label: i.toFixed(1) + "x"})
                            }
                        }
                    }
                    
                    delegate: Rectangle {
                        width: (multiplierGrid.width - 4 * Theme.spacingSmall) / 5
                        height: 50
                        color: root.currentMultiplier === model.value ? "#155dfc" : "#314158"
                        border.color: root.currentMultiplier === model.value ? "#4d8fff" : "#45556c"
                        border.width: 2
                        radius: Theme.radiusSmall
                        
                        Text {
                            anchors.centerIn: parent
                            text: model.label
                            font.pixelSize: Theme.fontSizeMedium
                            font.family: Theme.fontFamilyMono
                            color: root.currentMultiplier === model.value ? Theme.textPrimary : Theme.textSecondary
                        }
                        
                        MouseArea {
                            anchors.fill: parent
                            onClicked: {
                                root.currentMultiplier = model.value
                            }
                        }
                    }
                }
            }
        }
    }
    
    // 底部按钮区域
    footer: Item {
        width: parent.width
        height: 64
        
        Row {
            anchors.centerIn: parent
            spacing: Theme.spacingMedium
            
            Button {
                width: 120
                height: 42
                
                background: Rectangle {
                    color: "#314158"
                    radius: Theme.radiusMedium
                }
                
                contentItem: Text {
                    text: tr.translate("dialog.cancel")
                    font.pixelSize: Theme.fontSizeSmall
                    color: Theme.textPrimary
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }
                
                onClicked: root.close()
            }
            
            Button {
                width: 120
                height: 42
                
                background: Rectangle {
                    color: "#155dfc"
                    radius: Theme.radiusMedium
                }
                
                contentItem: Text {
                    text: tr.translate("dialog.confirm")
                    font.pixelSize: Theme.fontSizeSmall
                    color: Theme.textPrimary
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }
                
                onClicked: {
                    root.multiplierSelected(root.currentMultiplier)
                    root.close()
                }
            }
        }
    }
}