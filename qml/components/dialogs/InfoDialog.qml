// InfoDialog.qml - 信息对话框组件
import qt.rust.demo
import QtQuick
import QtQuick.Controls

Dialog {
    id: infoDialog

    TranslationBridge { id: tr }

    property string message: ""

    title: tr.translate("dialog.info") || "信息"
    modal: true
    standardButtons: Dialog.Ok
    
    contentItem: Text {
        text: infoDialog.message
        wrapMode: Text.WordWrap
    }
}
