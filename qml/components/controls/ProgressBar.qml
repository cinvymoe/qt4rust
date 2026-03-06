// ProgressBar.qml - 进度条组件
import QtQuick
import "../../styles"

Item {
    id: root
    
    property real value: 0.0  // 0.0 到 1.0
    property color barColor: Theme.dangerLight
    property var labels: []  // [{text: "0%", position: 0.0}, ...]
    
    height: 44
    
    Column {
        anchors.fill: parent
        spacing: Theme.spacingSmall
        
        // 进度条
        Rectangle {
            width: parent.width
            height: 16
            color: Theme.darkBorder
            radius: Theme.radiusLarge
            
            Rectangle {
                width: parent.width * root.value
                height: parent.height
                color: root.barColor
                radius: parent.radius
            }
        }
        
        // 标签行
        Row {
            width: parent.width
            height: 20
            
            Repeater {
                model: root.labels
                
                Text {
                    text: modelData.text
                    font.pixelSize: Theme.fontSizeSmall
                    color: modelData.color || Theme.textTertiary
                    width: {
                        if (index === 0) return implicitWidth
                        if (index === root.labels.length - 1) return implicitWidth
                        return (parent.width - root.labels[0].implicitWidth - root.labels[root.labels.length-1].implicitWidth) / (root.labels.length - 2)
                    }
                    horizontalAlignment: {
                        if (index === 0) return Text.AlignLeft
                        if (index === root.labels.length - 1) return Text.AlignRight
                        return Text.AlignHCenter
                    }
                }
            }
        }
    }
}
