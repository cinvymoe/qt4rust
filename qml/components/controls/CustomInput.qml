// CustomInput.qml - 自定义输入框组件
import QtQuick
import QtQuick.Controls

TextField {
    id: control
    
    background: Rectangle {
        implicitWidth: 200
        implicitHeight: 40
        color: control.enabled ? "white" : "#f0f0f0"
        border.color: control.activeFocus ? "#2196F3" : "#cccccc"
        border.width: control.activeFocus ? 2 : 1
        radius: 4
    }
}
