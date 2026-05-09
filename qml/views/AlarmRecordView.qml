// AlarmRecordView.qml - 报警记录视图
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qt.rust.demo
import "../styles"
import "../components/controls"

Item {
    id: alarmRecordView
    
    
    // HistoryViewModel 实例
    HistoryViewModel {
        id: historyViewModel
        
        Component.onCompleted: {
            historyViewModel.init_with_repository()
        }
    }
    
    // 监听 ViewModel 属性变化
    Connections {
        target: historyViewModel
        function onAlarm_records_jsonChanged() { parseAlarmRecords() }
        function onTotal_alarm_countChanged() { /* 统计数据会通过属性绑定自动更新 */ }
    }
    
    // 报警记录列表模型
    ListModel {
        id: alarmRecordModel
    }
    
    // 解析报警记录 JSON
    function parseAlarmRecords() {
        try {
            var records = JSON.parse(historyViewModel.alarm_records_json)
            alarmRecordModel.clear()
            for (var i = 0; i < records.length; i++) {
                var r = records[i]
                alarmRecordModel.append({
                    alarmType: r.alarmType,
                    title: r.title,
                    message: r.message,
                    date: r.date,
                    time: r.time,
                    momentValue: r.momentValue
                })
            }
        } catch (e) {
            console.log("解析报警记录失败:", e)
        }
    }
    
    // 主容器
    Rectangle {
        anchors.fill: parent
        color: Theme.darkBackground
        
        ColumnLayout {
            anchors.fill: parent
            spacing: 0
            
            // 顶部标题栏
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 93
                color: Theme.darkSurface
                border.color: Theme.darkBorder
                border.width: Theme.borderNormal
                
                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: Theme.spacingLarge
                    anchors.rightMargin: Theme.spacingLarge
                    spacing: Theme.spacingLarge
                    
                    // 左侧标题区域
                    ColumnLayout {
                        spacing: Theme.spacingSmall
                        
                        Text {
                            text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("alarm.title") }
                            font.pixelSize: Theme.fontSizeXLarge
                            font.family: Theme.fontFamilyDefault
                            color: Theme.textPrimary
                        }
                        
                        Text {
                            text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("alarm.subtitle") }
                            font.pixelSize: Theme.fontSizeSmall
                            font.family: Theme.fontFamilyDefault
                            color: Theme.textTertiary
                        }
                    }
                    
                    Item { Layout.fillWidth: true }
                    
                    // 右侧历史记录筛选区域
                    RowLayout {
                        spacing: Theme.spacingMedium
                        
                        HistoryFilterBar {
                            id: historyFilter
                            onFilterChanged: function(filter) {
                                historyViewModel.set_filter(filter)
                                historyViewModel.refresh_alarm_records()
                            }
                            onCustomTimeRangeChanged: function(startTimestamp, endTimestamp) {
                                historyViewModel.set_custom_time_range(startTimestamp, endTimestamp)
                                historyViewModel.refresh_alarm_records()
                            }
                        }
                        
                        // 刷新按钮
                        Button {
                            text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("chart.refresh") }
                            implicitWidth: 60
                            implicitHeight: 32
                            
                            background: Rectangle {
                                color: parent.down ? Theme.darkAccent : Theme.darkSurface
                                radius: Theme.radiusMedium
                                border.color: Theme.darkBorder
                                border.width: Theme.borderNormal
                            }
                            
                            contentItem: Text {
                                text: parent.text
                                font.pixelSize: Theme.fontSizeSmall
                                font.family: Theme.fontFamilyDefault
                                color: Theme.textPrimary
                                horizontalAlignment: Text.AlignHCenter
                                verticalAlignment: Text.AlignVCenter
                            }
                            
                            onClicked: {
                                historyViewModel.refresh_alarm_records()
                            }
                        }
                    }
                }
            }
            
            // 主内容区域 - 可滚动
            ScrollView {
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                contentWidth: availableWidth
                
                ColumnLayout {
                    width: parent.parent.width
                    spacing: Theme.spacingMedium
                    
                    Item { height: Theme.spacingMedium } // 顶部间距

                    // 统计卡片行
                    RowLayout {
                        Layout.fillWidth: true
                        Layout.leftMargin: Theme.spacingMedium
                        Layout.rightMargin: Theme.spacingMedium
                        Layout.preferredHeight: 102
                        spacing: Theme.spacingMedium
                        
                        // 总报警次数卡片
                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 102
                            color: Theme.darkSurface
                            border.color: Theme.darkBorder
                            border.width: Theme.borderNormal
                            radius: Theme.radiusMedium
                            
                            ColumnLayout {
                                anchors.fill: parent
                                anchors.margins: 17
                                spacing: Theme.spacingSmall
                                
                                RowLayout {
                                    spacing: Theme.spacingSmall
                                    
                                    Image {
                                        Layout.preferredWidth: Theme.iconSizeSmall
                                        Layout.preferredHeight: Theme.iconSizeSmall
                                        source: "../assets/images/icon-alarm-record.png"
                                        fillMode: Image.PreserveAspectFit
                                    }
                                    
                                    Text {
                                        text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("alarm.totalCount") }
                                        font.pixelSize: Theme.fontSizeMedium
                                        font.family: Theme.fontFamilyDefault
                                        color: Theme.textTertiary
                                    }
                                }
                                
                                Text {
                                    text: historyViewModel.total_alarm_count.toString()
                                    font.pixelSize: Theme.fontSizeXXLarge
                                    font.family: Theme.fontFamilyMono
                                    color: Theme.textPrimary
                                }
                            }
                        }
                        
                        // 预警次数卡片
                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 102
                            color: Theme.darkSurface
                            border.color: "#d08700"
                            border.width: Theme.borderNormal
                            radius: Theme.radiusMedium
                            
                            ColumnLayout {
                                anchors.fill: parent
                                anchors.margins: 17
                                spacing: Theme.spacingSmall
                                
                                RowLayout {
                                    spacing: Theme.spacingSmall
                                    
                                    Image {
                                        Layout.preferredWidth: Theme.iconSizeSmall
                                        Layout.preferredHeight: Theme.iconSizeSmall
                                        source: "../assets/images/icon-alert.png"
                                        fillMode: Image.PreserveAspectFit
                                    }
                                    
                                    Text {
                                        text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("alarm.warningCount") }
                                        font.pixelSize: Theme.fontSizeMedium
                                        font.family: Theme.fontFamilyDefault
                                        color: Theme.textTertiary
                                    }
                                }
                                
                                Text {
                                    text: historyViewModel.warning_count.toString()
                                    font.pixelSize: Theme.fontSizeXXLarge
                                    font.family: Theme.fontFamilyMono
                                    color: Theme.warningColor
                                }
                            }
                        }
                        
                        // 危险次数卡片
                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 102
                            color: Theme.darkSurface
                            border.color: Theme.dangerColor
                            border.width: Theme.borderNormal
                            radius: Theme.radiusMedium
                            
                            ColumnLayout {
                                anchors.fill: parent
                                anchors.margins: 17
                                spacing: Theme.spacingSmall
                                
                                RowLayout {
                                    spacing: Theme.spacingSmall
                                    
                                    Image {
                                        Layout.preferredWidth: Theme.iconSizeSmall
                                        Layout.preferredHeight: Theme.iconSizeSmall
                                        source: "../assets/images/icon-danger.png"
                                        fillMode: Image.PreserveAspectFit
                                    }
                                    
                                    Text {
                                        text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("alarm.dangerCount") }
                                        font.pixelSize: Theme.fontSizeMedium
                                        font.family: Theme.fontFamilyDefault
                                        color: Theme.textTertiary
                                    }
                                }
                                
                                Text {
                                    text: historyViewModel.danger_count.toString()
                                    font.pixelSize: Theme.fontSizeXXLarge
                                    font.family: Theme.fontFamilyMono
                                    color: Theme.dangerLight
                                }
                            }
                        }
                    }

                    // 报警级别说明卡片
                    Rectangle {
                        Layout.fillWidth: true
                        Layout.leftMargin: Theme.spacingMedium
                        Layout.rightMargin: Theme.spacingMedium
                        Layout.preferredHeight: 144
                        color: Theme.darkSurface
                        border.color: Theme.darkBorder
                        border.width: Theme.borderNormal
                        radius: Theme.radiusMedium
                        
                        ColumnLayout {
                            anchors.fill: parent
                            anchors.margins: 17
                            spacing: Theme.spacingMedium - 4
                            
                            Text {
                                text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("alarm.levels") }
                                font.pixelSize: Theme.fontSizeNormal
                                font.family: Theme.fontFamilyDefault
                                color: Theme.textPrimary
                            }
                            
                            RowLayout {
                                Layout.fillWidth: true
                                spacing: Theme.spacingMedium
                                
                                // 正常级别
                                Rectangle {
                                    Layout.fillWidth: true
                                    Layout.preferredHeight: 70
                                    color: Qt.rgba(3/255, 46/255, 21/255, 0.3)
                                    border.color: "#008236"
                                    border.width: Theme.borderNormal
                                    radius: Theme.radiusMedium
                                    
                                    RowLayout {
                                        anchors.fill: parent
                                        anchors.leftMargin: 13
                                        spacing: Theme.spacingMedium - 4
                                        
                                        Image {
                                            Layout.preferredWidth: Theme.iconSizeMedium
                                            Layout.preferredHeight: Theme.iconSizeMedium
                                            source: "../assets/images/icon-gauge.png"
                                            fillMode: Image.PreserveAspectFit
                                        }
                                        
                                        ColumnLayout {
                                            spacing: 0
                                            
                                            Text {
                                                text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("alarm.level.normal") }
                                                font.pixelSize: Theme.fontSizeMedium
                                                font.family: Theme.fontFamilyDefault
                                                color: Theme.successColor
                                            }
                                            
                                            Text {
                                                text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("alarm.level.normalDesc") }
                                                font.pixelSize: Theme.fontSizeSmall
                                                font.family: Theme.fontFamilyDefault
                                                color: Theme.textTertiary
                                            }
                                        }
                                    }
                                }
                                
                                // 预警级别
                                Rectangle {
                                    Layout.fillWidth: true
                                    Layout.preferredHeight: 70
                                    color: Qt.rgba(67/255, 32/255, 4/255, 0.3)
                                    border.color: "#a65f00"
                                    border.width: Theme.borderNormal
                                    radius: Theme.radiusMedium
                                    
                                    RowLayout {
                                        anchors.fill: parent
                                        anchors.leftMargin: 13
                                        spacing: Theme.spacingMedium - 4
                                        
                                        Image {
                                            Layout.preferredWidth: Theme.iconSizeMedium
                                            Layout.preferredHeight: Theme.iconSizeMedium
                                            source: "../assets/images/icon-alert.png"
                                            fillMode: Image.PreserveAspectFit
                                        }
                                        
                                        ColumnLayout {
                                            spacing: 0
                                            
                                            Text {
                                                text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("alarm.level.warning") }
                                                font.pixelSize: Theme.fontSizeMedium
                                                font.family: Theme.fontFamilyDefault
                                                color: Theme.warningColor
                                            }
                                            
                                            Text {
                                                text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("alarm.level.warningDesc") }
                                                font.pixelSize: Theme.fontSizeSmall
                                                font.family: Theme.fontFamilyDefault
                                                color: Theme.textTertiary
                                            }
                                        }
                                    }
                                }
                                
                                // 危险级别
                                Rectangle {
                                    Layout.fillWidth: true
                                    Layout.preferredHeight: 70
                                    color: Qt.rgba(70/255, 8/255, 9/255, 0.3)
                                    border.color: "#c10007"
                                    border.width: Theme.borderNormal
                                    radius: Theme.radiusMedium
                                    
                                    RowLayout {
                                        anchors.fill: parent
                                        anchors.leftMargin: 13
                                        spacing: Theme.spacingMedium - 4
                                        
                                        Image {
                                            Layout.preferredWidth: Theme.iconSizeMedium
                                            Layout.preferredHeight: Theme.iconSizeMedium
                                            source: "../assets/images/icon-danger.png"
                                            fillMode: Image.PreserveAspectFit
                                        }
                                        
                                        ColumnLayout {
                                            spacing: 0
                                            
                                            Text {
                                                text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("alarm.level.danger") }
                                                font.pixelSize: Theme.fontSizeMedium
                                                font.family: Theme.fontFamilyDefault
                                                color: Theme.dangerLight
                                            }
                                            
                                            Text {
                                                text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("alarm.level.dangerDesc") }
                                                font.pixelSize: Theme.fontSizeSmall
                                                font.family: Theme.fontFamilyDefault
                                                color: Theme.textTertiary
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // 报警记录列表 - 使用 Repeater 在 ColumnLayout 中
                    Repeater {
                        model: alarmRecordModel
                        
                        AlarmRecordItem {
                            Layout.fillWidth: true
                            Layout.leftMargin: Theme.spacingMedium
                            Layout.rightMargin: Theme.spacingMedium
                            alarmType: model.alarmType
                            title: model.title
                            message: model.message
                            date: model.date
                            time: model.time
                            momentValue: model.momentValue
                        }
                    }
                    
                    Item { height: Theme.spacingMedium } // 底部间距
                }
            }
        }
    }
}
