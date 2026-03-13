// BoomLengthCard.qml - 臂长卡片组件
import QtQuick
import QtQuick.Controls
import "../../styles"

Rectangle {
    id: root
    
    // 公开属性
    property real boomLength: 0.0    // 臂长（米）
    
    // 样式
    color: Theme.darkSurface
    border.color: Theme.darkBorder
    border.width: Theme.borderThin
    radius: Theme.radiusMedium
    
    Column {
        anchors.fill: parent
        anchors.margins: Theme.spacingLarge
        spacing: Theme.spacingSmall
        
        // 图标和标签
        Row {
            spacing: Theme.spacingSmall
            
            // 旋转45度的图标容器
            Item {
                width: Theme.iconSizeSmall
                height: Theme.iconSizeSmall
                
                IconImage {
                    anchors.centerIn: parent
                    source: "qrc:/qt/qml/qt/rust/demo/qml/assets/images/icon-boom-length.svg"
                    width: Theme.iconSizeSmall
                    height: Theme.iconSizeSmall
                    rotation: 45
                    color: Theme.textSecondary
                }
            }
            
            Text {
                text: "臂长"
                font.pixelSize: Theme.fontSizeSmall
                font.family: Theme.fontFamilyDefault
                color: Theme.textSecondary
                anchors.verticalCenter: parent.verticalCenter
            }
        }
        
        // 数值显示
        Row {
            spacing: 4
            
            Text {
                text: root.boomLength.toFixed(1)
                font.pixelSize: Theme.fontSizeHuge
                font.family: Theme.fontFamilyMono
                color: Theme.textPrimary
            }
            
            Text {
                text: "米"
                font.pixelSize: Theme.fontSizeLarge
                font.family: Theme.fontFamilyDefault
                color: Theme.textTertiary
                anchors.baseline: parent.children[0].baseline
                anchors.baselineOffset: -8
            }
        }
        
        // 描述文本
        Text {
            text: "吊臂总长度"
            font.pixelSize: Theme.fontSizeTiny
            font.family: Theme.fontFamilyDefault
            color: Theme.textTertiary
        }
    }
}
