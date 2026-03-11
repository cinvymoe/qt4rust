// SensorConfigView.qml - 传感器配置子页面
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../../styles"

Item {
    id: sensorConfigView
    
    Rectangle {
        anchors.fill: parent
        color: Theme.darkBackground
        
        Column {
            anchors.centerIn: parent
            spacing: Theme.spacingLarge
            
            Image {
                width: Theme.iconSizeLarge
                height: Theme.iconSizeLarge
                source: "../../assets/images/icon-sensor.svg"
                anchors.horizontalCenter: parent.horizontalCenter
            }
            
            Text {
                text: "传感器配置"
                font.pixelSize: Theme.fontSizeXLarge
                color: Theme.textPrimary
                anchors.horizontalCenter: parent.horizontalCenter
            }
            
            Text {
                text: "此功能正在开发中..."
                font.pixelSize: Theme.fontSizeMedium
                color: Theme.textSecondary
                anchors.horizontalCenter: parent.horizontalCenter
            }
        }
    }
}
