// MainLayout.qml - 主布局组件
import QtQuick
import QtQuick.Controls

Item {
    id: mainLayout
    
    default property alias content: contentArea.data
    property alias backgroundColor: background.color
    
    Rectangle {
        id: background
        anchors.fill: parent
        color: "#f0f0f0"
    }
    
    Item {
        id: contentArea
        anchors.fill: parent
        anchors.margins: 20
    }
}
