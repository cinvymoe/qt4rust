// Qt Rust Demo - 主界面 QML 文件
// 实现完整的 UI 布局，绑定 Rust Counter 对象

import QtQuick
import QtQuick.Controls
import QtQuick.Window
import QtQuick.VirtualKeyboard

Window {
    id: root
    width: 1024
    height: 600
    visible: true
    title: "Rust QT Demo"
    
    // 模拟 Counter 对象用于预览
    QtObject {
        id: counter
        property int count: 0
        property string platformInfo: "Preview Mode (x86_64)"
        
        function increment() {
            count++
        }
        
        function reset() {
            count = 0
        }
    }
    
    Rectangle {
        anchors.fill: parent
        color: "#f0f0f0"
        
        Column {
            id: content
            x: (parent.width - width) / 2
            y: (parent.height - height) / 2 + (inputPanel.active ? -inputPanel.height / 2 : 0)
            spacing: 20
            
            Behavior on y {
                NumberAnimation {
                    duration: 250
                    easing.type: Easing.InOutQuad
                }
            }
            
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
            
            // 虚拟键盘测试输入框
            TextField {
                id: textField
                anchors.horizontalCenter: parent.horizontalCenter
                width: 400
                placeholderText: "点击测试虚拟键盘..."
                font.pixelSize: 20
                
                onActiveFocusChanged: {
                    if (activeFocus) {
                        Qt.inputMethod.show()
                    }
                }
            }
            
            Text {
                anchors.horizontalCenter: parent.horizontalCenter
                text: "输入内容: " + textField.text
                font.pixelSize: 16
                color: "#666"
            }
        }
        
        // 覆盖整个内容区域的 MouseArea，用于隐藏键盘
        MouseArea {
            anchors.fill: parent
            anchors.bottomMargin: inputPanel.active ? inputPanel.height : 0
            enabled: textField.activeFocus
            onClicked: {
                textField.focus = false
            }
        }
    }
    
    // 虚拟键盘
    InputPanel {
        id: inputPanel
        z: 99
        x: 0
        y: root.height
        width: root.width
        
        states: State {
            name: "visible"
            when: inputPanel.active
            PropertyChanges {
                target: inputPanel
                y: root.height - inputPanel.height
            }
        }
        transitions: Transition {
            from: ""
            to: "visible"
            reversible: true
            NumberAnimation {
                properties: "y"
                duration: 250
                easing.type: Easing.InOutQuad
            }
        }
    }
}
