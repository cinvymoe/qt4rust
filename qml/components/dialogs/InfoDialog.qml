// InfoDialog.qml - 信息对话框组件
import QtQuick
import QtQuick.Controls

Dialog {
    id: infoDialog
    
    property string message: ""
    
    title: "信息"
    modal: true
    standardButtons: Dialog.Ok
    
    contentItem: Text {
        text: infoDialog.message
        wrapMode: Text.WordWrap
    }
}
