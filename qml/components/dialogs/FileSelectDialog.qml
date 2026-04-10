// FileSelectDialog.qml - 文件路径输入对话框
import QtQuick
import QtQuick.Controls
import "../../styles"
import "../controls"

Dialog {
    id: root
    
    // 对外属性
    property string selectedFilePath: ""
    
    // 信号
    signal fileSelected(string filePath)
    
    modal: true
    closePolicy: Popup.CloseOnEscape
    width: 600
    height: 280
    
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
                text: "导入额定载荷表"
                font.pixelSize: Theme.fontSizeLarge
                font.family: Theme.fontFamilyDefault
                font.bold: true
                color: Theme.textPrimary
            }
            
            // 副标题
            Text {
                text: "请输入 CSV 文件路径"
                font.pixelSize: Theme.fontSizeSmall
                font.family: Theme.fontFamilyDefault
                color: Theme.textSecondary
            }
        }
    }
    
    // 内容区域
    contentItem: Item {
        anchors.fill: parent
        anchors.margins: Theme.spacingMedium
        
        Column {
            anchors.fill: parent
            spacing: Theme.spacingMedium
            
            CustomInput {
                id: filePathInput
                width: parent.width
                placeholderText: "例如: test_import_load_table.csv"
                text: root.selectedFilePath
                
                Keys.onReturnPressed: {
                    if (filePathInput.text.trim() !== "") {
                        root.selectedFilePath = filePathInput.text.trim()
                        root.fileSelected(root.selectedFilePath)
                        root.close()
                    }
                }
            }
            
            Text {
                text: "提示：可以使用相对路径或绝对路径"
                font.pixelSize: Theme.fontSizeSmall
                font.family: Theme.fontFamilyDefault
                color: Theme.textSecondary
                opacity: 0.7
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
                    text: "取消"
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
                    text: "确定"
                    font.pixelSize: Theme.fontSizeSmall
                    color: Theme.textPrimary
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }
                
                onClicked: {
                    if (filePathInput.text.trim() !== "") {
                        root.selectedFilePath = filePathInput.text.trim()
                        root.fileSelected(root.selectedFilePath)
                        root.close()
                    }
                }
            }
        }
    }
    
    onOpened: {
        filePathInput.forceActiveFocus()
    }
}
