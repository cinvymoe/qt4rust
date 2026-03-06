// SettingsView.qml - 设置页面视图
import QtQuick
import QtQuick.Controls

Item {
    id: settingsView
    
    Column {
        anchors.centerIn: parent
        spacing: 20
        
        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: "设置页面"
            font.pixelSize: 32
            font.bold: true
        }
        
        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: "功能开发中..."
            font.pixelSize: 18
            color: "#666"
        }
    }
}
