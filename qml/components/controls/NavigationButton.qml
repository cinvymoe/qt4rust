// NavigationButton.qml - 导航按钮组件
import QtQuick
import QtQuick.Controls
import "../../styles"

Rectangle {
    id: navButton
    
    property string iconSource: ""
    property string text: ""
    property bool active: false
    property bool showBadge: false
    
    signal clicked()
    
    color: active ? Theme.darkBorder : "transparent"
    
    Column {
        anchors.centerIn: parent
        spacing: Theme.spacingTiny
        
        // 图标容器
        Item {
            width: Theme.iconSizeMedium
            height: Theme.iconSizeMedium
            anchors.horizontalCenter: parent.horizontalCenter
            
            // 使用 IconImage 组件 - 原生支持 SVG 颜色变化
            IconImage {
                id: icon
                source: navButton.iconSource
                width: Theme.iconSizeMedium
                height: Theme.iconSizeMedium
                anchors.centerIn: parent
                color: navButton.active ? Theme.textAccent : Theme.textTertiary
                
                Behavior on color {
                    ColorAnimation {
                        duration: Theme.animationDuration
                    }
                }
            }
            
            // 徽章
            Rectangle {
                visible: navButton.showBadge
                width: 12
                height: 12
                radius: Theme.radiusLarge
                color: Theme.dangerLight
                opacity: Theme.opacityMedium
                anchors.right: icon.right
                anchors.top: icon.top
                anchors.rightMargin: -4
                anchors.topMargin: -4
            }
        }
        
        Text {
            text: navButton.text
            font.pixelSize: Theme.fontSizeTiny
            font.family: Theme.fontFamilyDefault
            color: navButton.active ? Theme.textAccent : Theme.textTertiary
            anchors.horizontalCenter: parent.horizontalCenter
        }
    }
    
    // 顶部指示条
    Rectangle {
        visible: navButton.active
        width: parent.width
        height: 4
        color: Theme.darkAccent
        anchors.top: parent.top
    }
    
    MouseArea {
        anchors.fill: parent
        onClicked: navButton.clicked()
    }
}
