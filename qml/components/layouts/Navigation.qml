// Navigation.qml - 底部导航组件
import qt.rust.demo
import QtQuick
import QtQuick.Controls
import "../../styles"
import "../controls"

Rectangle {
    id: navigation
    height: Theme.navigationHeight
    color: Theme.darkSurface
    border.color: "#000000"
    border.width: Theme.borderThin
    
    // i18n 翻译对象
    Tr { id: tr }
    
    property int currentIndex: 0
    signal tabChanged(int index)
    
    Row {
        anchors.fill: parent
        spacing: 0
        
        // 主界面
        NavigationButton {
            width: parent.width / 4
            height: parent.height
            iconSource: "qrc:/qt/qml/qt/rust/demo/qml/assets/images/icon-home.svg"
            text: tr.t("nav.monitoring")
            active: navigation.currentIndex === 0
            onClicked: {
                navigation.currentIndex = 0
                navigation.tabChanged(0)
            }
        }
        
        // 数据曲线
        NavigationButton {
            width: parent.width / 4
            height: parent.height
            iconSource: "qrc:/qt/qml/qt/rust/demo/qml/assets/images/icon-chart.svg"
            text: tr.t("nav.charts")
            active: navigation.currentIndex === 1
            onClicked: {
                navigation.currentIndex = 1
                navigation.tabChanged(1)
            }
        }
        
        // 报警记录
        NavigationButton {
            width: parent.width / 4
            height: parent.height
            iconSource: "qrc:/qt/qml/qt/rust/demo/qml/assets/images/icon-alarm-record.svg"
            text: tr.t("nav.alarms")
            active: navigation.currentIndex === 2
            showBadge: true
            onClicked: {
                navigation.currentIndex = 2
                navigation.tabChanged(2)
            }
        }
        
        // 设置
        NavigationButton {
            width: parent.width / 4
            height: parent.height
            iconSource: "qrc:/qt/qml/qt/rust/demo/qml/assets/images/icon-settings.svg"
            text: tr.t("nav.settings")
            active: navigation.currentIndex === 3
            onClicked: {
                navigation.currentIndex = 3
                navigation.tabChanged(3)
            }
        }
    }
}
