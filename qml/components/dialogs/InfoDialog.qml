// InfoDialog.qml - 信息对话框组件
import QtQuick
import QtQuick.Controls
import "../../i18n"

Dialog {
    id: infoDialog
    
    property string message: ""
    
    Tr { id: tr }
    
    title: tr.t("dialog.info")
    modal: true
    standardButtons: Dialog.Ok
    
    contentItem: Text {
        text: infoDialog.message
        wrapMode: Text.WordWrap
    }
}
