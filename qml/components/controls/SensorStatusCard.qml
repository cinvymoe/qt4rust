// SensorStatusCard.qml - 传感器状态卡片组件
import qt.rust.demo
import QtQuick
import QtQuick.Controls
import "../../styles"

Rectangle {
    id: sensorCard


    // 公开属性
    property string sensorName: TranslationBridge.translate("systemStatus.sensorName") || "传感器"
    property string statusText: TranslationBridge.translate("systemStatus.online") || "在线 - 正常"
    property bool isOnline: true
    
    // 卡片样式
    width: 399.333
    height: 89.333
    color: isOnline ? Qt.rgba(3/255, 46/255, 21/255, 0.2) : Qt.rgba(70/255, 8/255, 9/255, 0.2)
    border.color: isOnline ? "#008236" : "#e7000b"
    border.width: Theme.borderThin
    radius: Theme.radiusMedium
    
    Column {
        anchors.fill: parent
        anchors.topMargin: 16.667
        anchors.bottomMargin: Theme.borderThin
        anchors.leftMargin: 16.667
        anchors.rightMargin: 16.667
        spacing: Theme.spacingSmall
        
        // 传感器名称 + 图标
        Item {
            width: parent.width
            height: 28
            
            Text {
                text: sensorCard.sensorName
                font.pixelSize: Theme.fontSizeNormal
                font.family: Theme.fontFamilyDefault
                color: "#e2e8f0"
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
            }
            
            Image {
                source: "../../assets/images/icon-sensor.svg"
                width: Theme.iconSizeMedium
                height: Theme.iconSizeMedium
                sourceSize.width: Theme.iconSizeMedium
                sourceSize.height: Theme.iconSizeMedium
                fillMode: Image.PreserveAspectFit
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
            }
        }
        
        // 状态指示器 + 状态文本
        Row {
            spacing: Theme.spacingSmall
            height: 20
            
            Image {
                source: "../../assets/images/icon-status-online.svg"
                width: 16
                height: 16
                sourceSize.width: 16
                sourceSize.height: 16
                fillMode: Image.PreserveAspectFit
                anchors.verticalCenter: parent.verticalCenter
                visible: sensorCard.isOnline
            }
            
            Rectangle {
                width: 16
                height: 16
                color: Theme.dangerColor
                radius: 8
                anchors.verticalCenter: parent.verticalCenter
                visible: !sensorCard.isOnline
            }
            
            Text {
                text: sensorCard.statusText
                font.pixelSize: Theme.fontSizeSmall
                font.family: Theme.fontFamilyDefault
                color: sensorCard.isOnline ? Theme.successColor : Theme.dangerColor
                anchors.verticalCenter: parent.verticalCenter
            }
        }
    }
}
