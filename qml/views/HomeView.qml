// HomeView.qml - 主页面视图
import QtQuick
import QtQuick.Controls
import qt.rust.demo
import "../components/controls"

Item {
    id: homeView
    
    
    property alias counter: counter
    
    Counter {
        id: counter
    }
    
    Column {
        id: content
        anchors.centerIn: parent
        spacing: 20
        
        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: "Platform: " + counter.platformInfo
            font.pixelSize: 50
        }
        
        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: "Count: " + counter.count
            font.pixelSize: 24
            font.bold: true
        }
        
        Button {
            anchors.horizontalCenter: parent.horizontalCenter
            text: "Increment"
            onClicked: counter.increment()
        }
        
        Button {
            anchors.horizontalCenter: parent.horizontalCenter
            text: "Reset"
            onClicked: counter.reset()
        }
        
        CustomInput {
            id: textField
            anchors.horizontalCenter: parent.horizontalCenter
            width: 400
            placeholderText: TranslationBridge.translate("home.placeholder")
            font.pixelSize: 20
            
            onActiveFocusChanged: {
                if (activeFocus) {
                    Qt.inputMethod.show()
                }
            }
        }
        
        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: TranslationBridge.translate("home.inputContent") + ": " + textField.text
            font.pixelSize: 16
            color: "#666"
        }
    }
}
