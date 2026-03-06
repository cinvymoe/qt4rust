// AlarmRecordView.qml - 报警记录视图
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../styles"
import "../components/controls"

Item {
    id: alarmRecordView
    
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
                
                ColumnLayout {
                    anchors.centerIn: parent
                    spacing: Theme.spacingSmall
                    
                    Text {
                        text: "报警记录"
                        font.pixelSize: Theme.fontSizeXLarge
                        font.family: Theme.fontFamilyDefault
                        color: Theme.textPrimary
                    }
                    
                    Text {
                        text: "系统报警历史与统计分析"
                        font.pixelSize: Theme.fontSizeSmall
                        font.family: Theme.fontFamilyDefault
                        color: Theme.textTertiary
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
                                        text: "总报警次数"
                                        font.pixelSize: Theme.fontSizeMedium
                                        font.family: Theme.fontFamilyDefault
                                        color: Theme.textTertiary
                                    }
                                }
                                
                                Text {
                                    text: "14"
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
                                        text: "预警次数"
                                        font.pixelSize: Theme.fontSizeMedium
                                        font.family: Theme.fontFamilyDefault
                                        color: Theme.textTertiary
                                    }
                                }
                                
                                Text {
                                    text: "10"
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
                                        text: "危险次数"
                                        font.pixelSize: Theme.fontSizeMedium
                                        font.family: Theme.fontFamilyDefault
                                        color: Theme.textTertiary
                                    }
                                }
                                
                                Text {
                                    text: "4"
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
                                text: "报警级别说明"
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
                                                text: "正常"
                                                font.pixelSize: Theme.fontSizeMedium
                                                font.family: Theme.fontFamilyDefault
                                                color: Theme.successColor
                                            }
                                            
                                            Text {
                                                text: "力矩 0-75%"
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
                                                text: "预警"
                                                font.pixelSize: Theme.fontSizeMedium
                                                font.family: Theme.fontFamilyDefault
                                                color: Theme.warningColor
                                            }
                                            
                                            Text {
                                                text: "力矩 75-90%"
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
                                                text: "危险"
                                                font.pixelSize: Theme.fontSizeMedium
                                                font.family: Theme.fontFamilyDefault
                                                color: Theme.dangerLight
                                            }
                                            
                                            Text {
                                                text: "力矩 ≥90%"
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

                    // 历史记录标题卡片
                    Rectangle {
                        Layout.fillWidth: true
                        Layout.leftMargin: Theme.spacingMedium
                        Layout.rightMargin: Theme.spacingMedium
                        Layout.preferredHeight: 300
                        color: Theme.darkSurface
                        border.color: Theme.darkBorder
                        border.width: Theme.borderNormal
                        radius: Theme.radiusMedium
                        
                        ColumnLayout {
                            anchors.fill: parent
                            anchors.margins: 17
                            spacing: Theme.spacingMedium
                            
                            // 标题行
                            RowLayout {
                                Layout.fillWidth: true
                                
                                Text {
                                    text: "历史记录"
                                    font.pixelSize: Theme.fontSizeNormal
                                    font.family: Theme.fontFamilyDefault
                                    color: Theme.textPrimary
                                }
                                
                                Item { Layout.fillWidth: true }
                                
                                RowLayout {
                                    spacing: Theme.spacingSmall
                                    
                                    Image {
                                        Layout.preferredWidth: 16
                                        Layout.preferredHeight: 16
                                        source: "../assets/images/icon-chart.png"
                                        fillMode: Image.PreserveAspectFit
                                    }
                                    
                                    Text {
                                        text: "共 14 条记录"
                                        font.pixelSize: Theme.fontSizeSmall
                                        font.family: Theme.fontFamilyDefault
                                        color: Theme.textTertiary
                                    }
                                }
                            }
                            
                            // 筛选按钮行
                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: Theme.spacingMedium - 4
                                
                                RowLayout {
                                    spacing: Theme.spacingSmall
                                    
                                    Button {
                                        text: "全部"
                                        background: Rectangle {
                                            color: Theme.darkBorder
                                            radius: Theme.radiusMedium
                                        }
                                        contentItem: Text {
                                            text: parent.text
                                            font.pixelSize: Theme.fontSizeMedium
                                            font.family: Theme.fontFamilyDefault
                                            color: Theme.textSecondary
                                            horizontalAlignment: Text.AlignHCenter
                                            verticalAlignment: Text.AlignVCenter
                                        }
                                        implicitWidth: 64
                                        implicitHeight: 40
                                    }
                                    
                                    Button {
                                        text: "今天"
                                        background: Rectangle {
                                            color: Theme.darkBorder
                                            radius: Theme.radiusMedium
                                        }
                                        contentItem: Text {
                                            text: parent.text
                                            font.pixelSize: Theme.fontSizeMedium
                                            font.family: Theme.fontFamilyDefault
                                            color: Theme.textSecondary
                                            horizontalAlignment: Text.AlignHCenter
                                            verticalAlignment: Text.AlignVCenter
                                        }
                                        implicitWidth: 64
                                        implicitHeight: 40
                                    }
                                    
                                    Button {
                                        text: "最近7天"
                                        background: Rectangle {
                                            color: Theme.darkBorder
                                            radius: Theme.radiusMedium
                                        }
                                        contentItem: Text {
                                            text: parent.text
                                            font.pixelSize: Theme.fontSizeMedium
                                            font.family: Theme.fontFamilyDefault
                                            color: Theme.textSecondary
                                            horizontalAlignment: Text.AlignHCenter
                                            verticalAlignment: Text.AlignVCenter
                                        }
                                        implicitWidth: 90
                                        implicitHeight: 40
                                    }
                                    
                                    Button {
                                        text: "最近30天"
                                        background: Rectangle {
                                            color: Theme.darkBorder
                                            radius: Theme.radiusMedium
                                        }
                                        contentItem: Text {
                                            text: parent.text
                                            font.pixelSize: Theme.fontSizeMedium
                                            font.family: Theme.fontFamilyDefault
                                            color: Theme.textSecondary
                                            horizontalAlignment: Text.AlignHCenter
                                            verticalAlignment: Text.AlignVCenter
                                        }
                                        implicitWidth: 99
                                        implicitHeight: 40
                                    }
                                    
                                    Button {
                                        text: "自定义"
                                        background: Rectangle {
                                            color: "#155dfc"
                                            radius: Theme.radiusMedium
                                        }
                                        contentItem: RowLayout {
                                            spacing: Theme.spacingSmall
                                            
                                            Image {
                                                Layout.preferredWidth: 16
                                                Layout.preferredHeight: 16
                                                source: "../assets/images/icon-settings.png"
                                                fillMode: Image.PreserveAspectFit
                                            }
                                            
                                            Text {
                                                text: "自定义"
                                                font.pixelSize: Theme.fontSizeMedium
                                                font.family: Theme.fontFamilyDefault
                                                color: Theme.textPrimary
                                            }
                                        }
                                        implicitWidth: 104
                                        implicitHeight: 40
                                    }
                                }
                                
                                // 日期选择器
                                Rectangle {
                                    Layout.fillWidth: true
                                    Layout.preferredHeight: 104
                                    color: Theme.darkBorder
                                    border.color: "#45556c"
                                    border.width: Theme.borderNormal
                                    radius: Theme.radiusMedium
                                    
                                    RowLayout {
                                        anchors.fill: parent
                                        anchors.margins: 17
                                        spacing: Theme.spacingMedium
                                        
                                        ColumnLayout {
                                            Layout.fillWidth: true
                                            spacing: Theme.spacingSmall
                                            
                                            Text {
                                                text: "开始日期"
                                                font.pixelSize: Theme.fontSizeSmall
                                                font.family: Theme.fontFamilyDefault
                                                color: Theme.textSecondary
                                            }
                                            
                                            Rectangle {
                                                Layout.fillWidth: true
                                                Layout.preferredHeight: 42
                                                color: Theme.darkSurface
                                                border.color: "#45556c"
                                                border.width: Theme.borderNormal
                                                radius: Theme.radiusMedium
                                            }
                                        }
                                        
                                        ColumnLayout {
                                            Layout.fillWidth: true
                                            spacing: Theme.spacingSmall
                                            
                                            Text {
                                                text: "结束日期"
                                                font.pixelSize: Theme.fontSizeSmall
                                                font.family: Theme.fontFamilyDefault
                                                color: Theme.textSecondary
                                            }
                                            
                                            Rectangle {
                                                Layout.fillWidth: true
                                                Layout.preferredHeight: 42
                                                color: Theme.darkSurface
                                                border.color: "#45556c"
                                                border.width: Theme.borderNormal
                                                radius: Theme.radiusMedium
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // 报警记录列表 - 使用 Repeater 在 ColumnLayout 中
                    Repeater {
                        model: ListModel {
                            id: alarmRecordModel
                            
                            ListElement {
                                alarmType: "danger"
                                title: "危险报警"
                                message: "危险！力矩已达 95%，请立即减载！"
                                date: "2026/3/6"
                                time: "15:48:32"
                                momentValue: "95.0%"
                            }
                            
                            ListElement {
                                alarmType: "warning"
                                title: "预警提示"
                                message: "预警：力矩达到 75.9%，注意安全！"
                                date: "2026/3/6"
                                time: "15:47:36"
                                momentValue: "75.9%"
                            }
                            
                            ListElement {
                                alarmType: "danger"
                                title: "危险报警"
                                message: "危险！力矩已达 94%，请立即减载！"
                                date: "2026/3/6"
                                time: "15:47:19"
                                momentValue: "94.0%"
                            }
                            
                            ListElement {
                                alarmType: "warning"
                                title: "预警提示"
                                message: "预警：力矩达到 82.3%，注意安全！"
                                date: "2026/3/6"
                                time: "15:46:54"
                                momentValue: "82.3%"
                            }
                            
                            ListElement {
                                alarmType: "danger"
                                title: "危险报警"
                                message: "危险！力矩已达 91%，请立即减载！"
                                date: "2026/3/6"
                                time: "15:45:28"
                                momentValue: "91.0%"
                            }
                            
                            ListElement {
                                alarmType: "warning"
                                title: "预警提示"
                                message: "预警：力矩达到 78.5%，注意安全！"
                                date: "2026/3/6"
                                time: "15:44:12"
                                momentValue: "78.5%"
                            }
                            
                            ListElement {
                                alarmType: "warning"
                                title: "预警提示"
                                message: "预警：力矩达到 76.2%，注意安全！"
                                date: "2026/3/6"
                                time: "15:43:05"
                                momentValue: "76.2%"
                            }
                            
                            ListElement {
                                alarmType: "danger"
                                title: "危险报警"
                                message: "危险！力矩已达 93%，请立即减载！"
                                date: "2026/3/6"
                                time: "15:42:18"
                                momentValue: "93.0%"
                            }
                        }
                        
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
