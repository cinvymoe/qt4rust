// CustomTimeRangeDialog.qml - 自定义时间范围选择对话框
import qt.rust.demo
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../../styles"

Dialog {
    id: root

    TranslationBridge { id: tr }

    // 对外暴露的属性
    property string startTime: ""
    property string endTime: ""
    
    // 信号
    signal timeRangeApplied(string startTime, string endTime)
    signal timeRangeReset()
    
    modal: true
    closePolicy: Popup.CloseOnEscape
    width: 590
    height: 380
    
    // 居中显示 - 使用 parent 的尺寸动态计算
    x: (parent.width - width) / 2
    y: (parent.height - height) / 2
    
    // 背景
    background: Rectangle {
        color: Theme.darkSurface
        radius: Theme.radiusMedium
        border.color: Theme.darkBorder
        border.width: Theme.borderNormal
    }
    
    // 标题区域
    header: Item {
        width: parent.width
        height: 64
        
        // 关闭按钮
        Button {
            id: closeButton
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.margins: Theme.spacingMedium
            width: 32
            height: 32
            
            background: Rectangle {
                color: "transparent"
                radius: Theme.radiusSmall
            }
            
            contentItem: Text {
                text: "✕"
                font.pixelSize: Theme.fontSizeLarge
                color: Theme.textSecondary
                horizontalAlignment: Text.AlignHCenter
                verticalAlignment: Text.AlignVCenter
            }
            
            onClicked: root.close()
        }
        
        Column {
            anchors.left: parent.left
            anchors.top: parent.top
            anchors.margins: Theme.spacingMedium
            spacing: Theme.spacingTiny
            
            // 主标题
            Text {
                text: tr.translate("dialog.timeRange")
                font.pixelSize: Theme.fontSizeLarge
                font.family: Theme.fontFamilyDefault
                font.bold: true
                color: Theme.textPrimary
            }

            // 副标题
            Text {
                text: tr.translate("dialog.timeRangeDesc")
                font.pixelSize: Theme.fontSizeSmall
                font.family: Theme.fontFamilyDefault
                color: Theme.textSecondary
            }
            
            // 副标题
            Text {
                text: tr.translate("dialog.timeRangeDesc")
                font.pixelSize: Theme.fontSizeSmall
                font.family: Theme.fontFamilyDefault
                color: Theme.textSecondary
            }
        }
    }
    
    // 内容区域
    contentItem: Item {
        Column {
            anchors.fill: parent
            anchors.margins: Theme.spacingMedium
            spacing: Theme.spacingMedium
            
            // 开始时间
            Column {
                width: parent.width
                spacing: Theme.spacingSmall
                
                Text {
                    text: tr.translate("dialog.startTime")
                    font.pixelSize: Theme.fontSizeMedium
                    font.family: Theme.fontFamilyDefault
                    color: Theme.textSecondary
                }
                
                Row {
                    width: parent.width
                    spacing: Theme.spacingMedium
                    
                    // 年
                    ComboBox {
                        id: startYearCombo
                        width: 110
                        height: 48
                        model: {
                            var years = []
                            var currentYear = new Date().getFullYear()
                            for (var i = currentYear; i >= currentYear - 5; i--) {
                                years.push(i)
                            }
                            return years
                        }
                        currentIndex: 0
                        
                        background: Rectangle {
                            color: Theme.darkBackground
                            border.color: startYearCombo.activeFocus ? Theme.darkAccent : Theme.darkBorder
                            border.width: Theme.borderNormal
                            radius: Theme.radiusMedium
                        }
                        
                        contentItem: Text {
                            text: startYearCombo.displayText + tr.translate("timeUnit.year")
                            font.pixelSize: Theme.fontSizeSmall
                            font.family: Theme.fontFamilyDefault
                            color: Theme.textPrimary
                            verticalAlignment: Text.AlignVCenter
                            leftPadding: Theme.spacingSmall
                        }
                        
                        delegate: ItemDelegate {
                            width: startYearCombo.width
                            height: 40
                            
                            background: Rectangle {
                                color: highlighted ? Theme.darkAccent : Theme.darkSurface
                            }
                            
                            contentItem: Text {
                                text: modelData + tr.translate("timeUnit.year")
                                font.pixelSize: Theme.fontSizeSmall
                                font.family: Theme.fontFamilyDefault
                                color: Theme.textPrimary
                                verticalAlignment: Text.AlignVCenter
                                leftPadding: Theme.spacingSmall
                            }
                        }
                        
                        popup: Popup {
                            y: startYearCombo.height
                            width: startYearCombo.width
                            height: Math.min(contentItem.implicitHeight, 240)
                            
                            background: Rectangle {
                                color: Theme.darkSurface
                                border.color: Theme.darkBorder
                                border.width: Theme.borderNormal
                                radius: Theme.radiusMedium
                            }
                            
                            contentItem: ListView {
                                clip: true
                                implicitHeight: contentHeight
                                model: startYearCombo.delegateModel
                                currentIndex: startYearCombo.highlightedIndex
                                
                                ScrollBar.vertical: ScrollBar {
                                    active: true
                                }
                            }
                        }
                    }
                    
                    // 月
                    ComboBox {
                        id: startMonthCombo
                        width: 90
                        height: 48
                        model: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]
                        currentIndex: new Date().getMonth()
                        
                        background: Rectangle {
                            color: Theme.darkBackground
                            border.color: startMonthCombo.activeFocus ? Theme.darkAccent : Theme.darkBorder
                            border.width: Theme.borderNormal
                            radius: Theme.radiusMedium
                        }
                        
                        contentItem: Text {
                            text: startMonthCombo.displayText + tr.translate("timeUnit.month")
                            font.pixelSize: Theme.fontSizeSmall
                            font.family: Theme.fontFamilyDefault
                            color: Theme.textPrimary
                            verticalAlignment: Text.AlignVCenter
                            leftPadding: Theme.spacingSmall
                        }
                        
                        delegate: ItemDelegate {
                            width: startMonthCombo.width
                            height: 40
                            
                            background: Rectangle {
                                color: highlighted ? Theme.darkAccent : Theme.darkSurface
                            }
                            
                            contentItem: Text {
                                text: modelData + tr.translate("timeUnit.month")
                                font.pixelSize: Theme.fontSizeSmall
                                font.family: Theme.fontFamilyDefault
                                color: Theme.textPrimary
                                verticalAlignment: Text.AlignVCenter
                                leftPadding: Theme.spacingSmall
                            }
                        }
                        
                        popup: Popup {
                            y: startMonthCombo.height
                            width: startMonthCombo.width
                            height: Math.min(contentItem.implicitHeight, 240)
                            
                            background: Rectangle {
                                color: Theme.darkSurface
                                border.color: Theme.darkBorder
                                border.width: Theme.borderNormal
                                radius: Theme.radiusMedium
                            }
                            
                            contentItem: ListView {
                                clip: true
                                implicitHeight: contentHeight
                                model: startMonthCombo.delegateModel
                                currentIndex: startMonthCombo.highlightedIndex
                                
                                ScrollBar.vertical: ScrollBar {
                                    active: true
                                }
                            }
                        }
                    }
                    
                    // 日
                    ComboBox {
                        id: startDayCombo
                        width: 90
                        height: 48
                        model: {
                            var days = []
                            for (var i = 1; i <= 31; i++) {
                                days.push(i)
                            }
                            return days
                        }
                        currentIndex: new Date().getDate() - 1
                        
                        background: Rectangle {
                            color: Theme.darkBackground
                            border.color: startDayCombo.activeFocus ? Theme.darkAccent : Theme.darkBorder
                            border.width: Theme.borderNormal
                            radius: Theme.radiusMedium
                        }
                        
                        contentItem: Text {
                            text: startDayCombo.displayText + tr.translate("timeUnit.day")
                            font.pixelSize: Theme.fontSizeSmall
                            font.family: Theme.fontFamilyDefault
                            color: Theme.textPrimary
                            verticalAlignment: Text.AlignVCenter
                            leftPadding: Theme.spacingSmall
                        }
                        
                        delegate: ItemDelegate {
                            width: startDayCombo.width
                            height: 40
                            
                            background: Rectangle {
                                color: highlighted ? Theme.darkAccent : Theme.darkSurface
                            }
                            
                            contentItem: Text {
                                text: modelData + tr.translate("timeUnit.day")
                                font.pixelSize: Theme.fontSizeSmall
                                font.family: Theme.fontFamilyDefault
                                color: Theme.textPrimary
                                verticalAlignment: Text.AlignVCenter
                                leftPadding: Theme.spacingSmall
                            }
                        }
                        
                        popup: Popup {
                            y: startDayCombo.height
                            width: startDayCombo.width
                            height: Math.min(contentItem.implicitHeight, 240)
                            
                            background: Rectangle {
                                color: Theme.darkSurface
                                border.color: Theme.darkBorder
                                border.width: Theme.borderNormal
                                radius: Theme.radiusMedium
                            }
                            
                            contentItem: ListView {
                                clip: true
                                implicitHeight: contentHeight
                                model: startDayCombo.delegateModel
                                currentIndex: startDayCombo.highlightedIndex
                                
                                ScrollBar.vertical: ScrollBar {
                                    active: true
                                }
                            }
                        }
                    }
                    
                    // 小时
                    ComboBox {
                        id: startHourCombo
                        width: 90
                        height: 48
                        model: {
                            var hours = []
                            for (var i = 0; i < 24; i++) {
                                hours.push(i)
                            }
                            return hours
                        }
                        currentIndex: new Date().getHours()
                        
                        background: Rectangle {
                            color: Theme.darkBackground
                            border.color: startHourCombo.activeFocus ? Theme.darkAccent : Theme.darkBorder
                            border.width: Theme.borderNormal
                            radius: Theme.radiusMedium
                        }
                        
                        contentItem: Text {
                            text: (startHourCombo.displayText < 10 ? "0" : "") + startHourCombo.displayText + tr.translate("timeUnit.hour")
                            font.pixelSize: Theme.fontSizeSmall
                            font.family: Theme.fontFamilyDefault
                            color: Theme.textPrimary
                            verticalAlignment: Text.AlignVCenter
                            leftPadding: Theme.spacingSmall
                        }
                        
                        delegate: ItemDelegate {
                            width: startHourCombo.width
                            height: 40
                            
                            background: Rectangle {
                                color: highlighted ? Theme.darkAccent : Theme.darkSurface
                            }
                            
                            contentItem: Text {
                                text: (modelData < 10 ? "0" : "") + modelData + tr.translate("timeUnit.hour")
                                font.pixelSize: Theme.fontSizeSmall
                                font.family: Theme.fontFamilyDefault
                                color: Theme.textPrimary
                                verticalAlignment: Text.AlignVCenter
                                leftPadding: Theme.spacingSmall
                            }
                        }
                        
                        popup: Popup {
                            y: startHourCombo.height
                            width: startHourCombo.width
                            height: Math.min(contentItem.implicitHeight, 240)
                            
                            background: Rectangle {
                                color: Theme.darkSurface
                                border.color: Theme.darkBorder
                                border.width: Theme.borderNormal
                                radius: Theme.radiusMedium
                            }
                            
                            contentItem: ListView {
                                clip: true
                                implicitHeight: contentHeight
                                model: startHourCombo.delegateModel
                                currentIndex: startHourCombo.highlightedIndex
                                
                                ScrollBar.vertical: ScrollBar {
                                    active: true
                                }
                            }
                        }
                    }
                    
                    // 分钟
                    ComboBox {
                        id: startMinuteCombo
                        width: 90
                        height: 48
                        model: {
                            var minutes = []
                            for (var i = 0; i < 60; i++) {
                                minutes.push(i)
                            }
                            return minutes
                        }
                        currentIndex: new Date().getMinutes()
                        
                        background: Rectangle {
                            color: Theme.darkBackground
                            border.color: startMinuteCombo.activeFocus ? Theme.darkAccent : Theme.darkBorder
                            border.width: Theme.borderNormal
                            radius: Theme.radiusMedium
                        }
                        
                        contentItem: Text {
                            text: (startMinuteCombo.displayText < 10 ? "0" : "") + startMinuteCombo.displayText + tr.translate("timeUnit.minute")
                            font.pixelSize: Theme.fontSizeSmall
                            font.family: Theme.fontFamilyDefault
                            color: Theme.textPrimary
                            verticalAlignment: Text.AlignVCenter
                            leftPadding: Theme.spacingSmall
                        }
                        
                        delegate: ItemDelegate {
                            width: startMinuteCombo.width
                            height: 40
                            
                            background: Rectangle {
                                color: highlighted ? Theme.darkAccent : Theme.darkSurface
                            }
                            
                            contentItem: Text {
                                text: (modelData < 10 ? "0" : "") + modelData + tr.translate("timeUnit.minute")
                                font.pixelSize: Theme.fontSizeSmall
                                font.family: Theme.fontFamilyDefault
                                color: Theme.textPrimary
                                verticalAlignment: Text.AlignVCenter
                                leftPadding: Theme.spacingSmall
                            }
                        }
                        
                        popup: Popup {
                            y: startMinuteCombo.height
                            width: startMinuteCombo.width
                            height: Math.min(contentItem.implicitHeight, 240)
                            
                            background: Rectangle {
                                color: Theme.darkSurface
                                border.color: Theme.darkBorder
                                border.width: Theme.borderNormal
                                radius: Theme.radiusMedium
                            }
                            
                            contentItem: ListView {
                                clip: true
                                implicitHeight: contentHeight
                                model: startMinuteCombo.delegateModel
                                currentIndex: startMinuteCombo.highlightedIndex
                                
                                ScrollBar.vertical: ScrollBar {
                                    active: true
                                }
                            }
                        }
                    }
                }
            }
            
            // 分隔线
            Rectangle {
                width: parent.width
                height: 1
                color: Theme.darkBorder
            }
            
            // 结束时间
            Column {
                width: parent.width
                spacing: Theme.spacingSmall

                Text {
                    text: tr.translate("dialog.endTime")
                    font.pixelSize: Theme.fontSizeMedium
                    font.family: Theme.fontFamilyDefault
                    color: Theme.textSecondary
                }
                
                Row {
                    width: parent.width
                    spacing: Theme.spacingMedium
                    
                    // 年
                    ComboBox {
                        id: endYearCombo
                        width: 110
                        height: 48
                        model: {
                            var years = []
                            var currentYear = new Date().getFullYear()
                            for (var i = currentYear; i >= currentYear - 5; i--) {
                                years.push(i)
                            }
                            return years
                        }
                        currentIndex: 0
                        
                        background: Rectangle {
                            color: Theme.darkBackground
                            border.color: endYearCombo.activeFocus ? Theme.darkAccent : Theme.darkBorder
                            border.width: Theme.borderNormal
                            radius: Theme.radiusMedium
                        }
                        
                        contentItem: Text {
                            text: endYearCombo.displayText + tr.translate("timeUnit.year")
                            font.pixelSize: Theme.fontSizeSmall
                            font.family: Theme.fontFamilyDefault
                            color: Theme.textPrimary
                            verticalAlignment: Text.AlignVCenter
                            leftPadding: Theme.spacingSmall
                        }
                        
                        delegate: ItemDelegate {
                            width: endYearCombo.width
                            height: 40
                            
                            background: Rectangle {
                                color: highlighted ? Theme.darkAccent : Theme.darkSurface
                            }
                            
                            contentItem: Text {
                                text: modelData + tr.translate("timeUnit.year")
                                font.pixelSize: Theme.fontSizeSmall
                                font.family: Theme.fontFamilyDefault
                                color: Theme.textPrimary
                                verticalAlignment: Text.AlignVCenter
                                leftPadding: Theme.spacingSmall
                            }
                        }
                        
                        popup: Popup {
                            y: endYearCombo.height
                            width: endYearCombo.width
                            height: Math.min(contentItem.implicitHeight, 240)
                            
                            background: Rectangle {
                                color: Theme.darkSurface
                                border.color: Theme.darkBorder
                                border.width: Theme.borderNormal
                                radius: Theme.radiusMedium
                            }
                            
                            contentItem: ListView {
                                clip: true
                                implicitHeight: contentHeight
                                model: endYearCombo.delegateModel
                                currentIndex: endYearCombo.highlightedIndex
                                
                                ScrollBar.vertical: ScrollBar {
                                    active: true
                                }
                            }
                        }
                    }
                    
                    // 月
                    ComboBox {
                        id: endMonthCombo
                        width: 90
                        height: 48
                        model: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]
                        currentIndex: new Date().getMonth()
                        
                        background: Rectangle {
                            color: Theme.darkBackground
                            border.color: endMonthCombo.activeFocus ? Theme.darkAccent : Theme.darkBorder
                            border.width: Theme.borderNormal
                            radius: Theme.radiusMedium
                        }
                        
                        contentItem: Text {
                            text: endMonthCombo.displayText + tr.translate("timeUnit.month")
                            font.pixelSize: Theme.fontSizeSmall
                            font.family: Theme.fontFamilyDefault
                            color: Theme.textPrimary
                            verticalAlignment: Text.AlignVCenter
                            leftPadding: Theme.spacingSmall
                        }
                        
                        delegate: ItemDelegate {
                            width: endMonthCombo.width
                            height: 40
                            
                            background: Rectangle {
                                color: highlighted ? Theme.darkAccent : Theme.darkSurface
                            }
                            
                            contentItem: Text {
                                text: modelData + tr.translate("timeUnit.month")
                                font.pixelSize: Theme.fontSizeSmall
                                font.family: Theme.fontFamilyDefault
                                color: Theme.textPrimary
                                verticalAlignment: Text.AlignVCenter
                                leftPadding: Theme.spacingSmall
                            }
                        }
                        
                        popup: Popup {
                            y: endMonthCombo.height
                            width: endMonthCombo.width
                            height: Math.min(contentItem.implicitHeight, 240)
                            
                            background: Rectangle {
                                color: Theme.darkSurface
                                border.color: Theme.darkBorder
                                border.width: Theme.borderNormal
                                radius: Theme.radiusMedium
                            }
                            
                            contentItem: ListView {
                                clip: true
                                implicitHeight: contentHeight
                                model: endMonthCombo.delegateModel
                                currentIndex: endMonthCombo.highlightedIndex
                                
                                ScrollBar.vertical: ScrollBar {
                                    active: true
                                }
                            }
                        }
                    }
                    
                    // 日
                    ComboBox {
                        id: endDayCombo
                        width: 90
                        height: 48
                        model: {
                            var days = []
                            for (var i = 1; i <= 31; i++) {
                                days.push(i)
                            }
                            return days
                        }
                        currentIndex: new Date().getDate() - 1
                        
                        background: Rectangle {
                            color: Theme.darkBackground
                            border.color: endDayCombo.activeFocus ? Theme.darkAccent : Theme.darkBorder
                            border.width: Theme.borderNormal
                            radius: Theme.radiusMedium
                        }
                        
                        contentItem: Text {
                            text: endDayCombo.displayText + tr.translate("timeUnit.day")
                            font.pixelSize: Theme.fontSizeSmall
                            font.family: Theme.fontFamilyDefault
                            color: Theme.textPrimary
                            verticalAlignment: Text.AlignVCenter
                            leftPadding: Theme.spacingSmall
                        }
                        
                        delegate: ItemDelegate {
                            width: endDayCombo.width
                            height: 40
                            
                            background: Rectangle {
                                color: highlighted ? Theme.darkAccent : Theme.darkSurface
                            }
                            
                            contentItem: Text {
                                text: modelData + tr.translate("timeUnit.day")
                                font.pixelSize: Theme.fontSizeSmall
                                font.family: Theme.fontFamilyDefault
                                color: Theme.textPrimary
                                verticalAlignment: Text.AlignVCenter
                                leftPadding: Theme.spacingSmall
                            }
                        }
                        
                        popup: Popup {
                            y: endDayCombo.height
                            width: endDayCombo.width
                            height: Math.min(contentItem.implicitHeight, 240)
                            
                            background: Rectangle {
                                color: Theme.darkSurface
                                border.color: Theme.darkBorder
                                border.width: Theme.borderNormal
                                radius: Theme.radiusMedium
                            }
                            
                            contentItem: ListView {
                                clip: true
                                implicitHeight: contentHeight
                                model: endDayCombo.delegateModel
                                currentIndex: endDayCombo.highlightedIndex
                                
                                ScrollBar.vertical: ScrollBar {
                                    active: true
                                }
                            }
                        }
                    }
                    
                    // 小时
                    ComboBox {
                        id: endHourCombo
                        width: 90
                        height: 48
                        model: {
                            var hours = []
                            for (var i = 0; i < 24; i++) {
                                hours.push(i)
                            }
                            return hours
                        }
                        currentIndex: new Date().getHours()
                        
                        background: Rectangle {
                            color: Theme.darkBackground
                            border.color: endHourCombo.activeFocus ? Theme.darkAccent : Theme.darkBorder
                            border.width: Theme.borderNormal
                            radius: Theme.radiusMedium
                        }
                        
                        contentItem: Text {
                            text: (endHourCombo.displayText < 10 ? "0" : "") + endHourCombo.displayText + tr.translate("timeUnit.hour")
                            font.pixelSize: Theme.fontSizeSmall
                            font.family: Theme.fontFamilyDefault
                            color: Theme.textPrimary
                            verticalAlignment: Text.AlignVCenter
                            leftPadding: Theme.spacingSmall
                        }
                        
                        delegate: ItemDelegate {
                            width: endHourCombo.width
                            height: 40
                            
                            background: Rectangle {
                                color: highlighted ? Theme.darkAccent : Theme.darkSurface
                            }
                            
                            contentItem: Text {
                                text: (modelData < 10 ? "0" : "") + modelData + tr.translate("timeUnit.hour")
                                font.pixelSize: Theme.fontSizeSmall
                                font.family: Theme.fontFamilyDefault
                                color: Theme.textPrimary
                                verticalAlignment: Text.AlignVCenter
                                leftPadding: Theme.spacingSmall
                            }
                        }
                        
                        popup: Popup {
                            y: endHourCombo.height
                            width: endHourCombo.width
                            height: Math.min(contentItem.implicitHeight, 240)
                            
                            background: Rectangle {
                                color: Theme.darkSurface
                                border.color: Theme.darkBorder
                                border.width: Theme.borderNormal
                                radius: Theme.radiusMedium
                            }
                            
                            contentItem: ListView {
                                clip: true
                                implicitHeight: contentHeight
                                model: endHourCombo.delegateModel
                                currentIndex: endHourCombo.highlightedIndex
                                
                                ScrollBar.vertical: ScrollBar {
                                    active: true
                                }
                            }
                        }
                    }
                    
                    // 分钟
                    ComboBox {
                        id: endMinuteCombo
                        width: 90
                        height: 48
                        model: {
                            var minutes = []
                            for (var i = 0; i < 60; i++) {
                                minutes.push(i)
                            }
                            return minutes
                        }
                        currentIndex: new Date().getMinutes()
                        
                        background: Rectangle {
                            color: Theme.darkBackground
                            border.color: endMinuteCombo.activeFocus ? Theme.darkAccent : Theme.darkBorder
                            border.width: Theme.borderNormal
                            radius: Theme.radiusMedium
                        }
                        
                        contentItem: Text {
                            text: (endMinuteCombo.displayText < 10 ? "0" : "") + endMinuteCombo.displayText + tr.translate("timeUnit.minute")
                            font.pixelSize: Theme.fontSizeSmall
                            font.family: Theme.fontFamilyDefault
                            color: Theme.textPrimary
                            verticalAlignment: Text.AlignVCenter
                            leftPadding: Theme.spacingSmall
                        }
                        
                        delegate: ItemDelegate {
                            width: endMinuteCombo.width
                            height: 40
                            
                            background: Rectangle {
                                color: highlighted ? Theme.darkAccent : Theme.darkSurface
                            }
                            
                            contentItem: Text {
                                text: (modelData < 10 ? "0" : "") + modelData + tr.translate("timeUnit.minute")
                                font.pixelSize: Theme.fontSizeSmall
                                font.family: Theme.fontFamilyDefault
                                color: Theme.textPrimary
                                verticalAlignment: Text.AlignVCenter
                                leftPadding: Theme.spacingSmall
                            }
                        }
                        
                        popup: Popup {
                            y: endMinuteCombo.height
                            width: endMinuteCombo.width
                            height: Math.min(contentItem.implicitHeight, 240)
                            
                            background: Rectangle {
                                color: Theme.darkSurface
                                border.color: Theme.darkBorder
                                border.width: Theme.borderNormal
                                radius: Theme.radiusMedium
                            }
                            
                            contentItem: ListView {
                                clip: true
                                implicitHeight: contentHeight
                                model: endMinuteCombo.delegateModel
                                currentIndex: endMinuteCombo.highlightedIndex
                                
                                ScrollBar.vertical: ScrollBar {
                                    active: true
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // 底部按钮区域
    footer: Item {
        width: parent.width
        height: 56
        
        Row {
            anchors.centerIn: parent
            spacing: Theme.spacingMedium
            
            // 重置筛选按钮
            Button {
                id: resetButton
                text: tr.translate("dialog.resetFilter")
                implicitWidth: 120
                implicitHeight: 44
                
                background: Rectangle {
                    color: Theme.darkBorder
                    radius: Theme.radiusMedium
                    
                    // 悬停效果
                    opacity: resetButton.hovered ? 0.8 : 1.0
                    Behavior on opacity {
                        NumberAnimation { duration: Theme.animationDuration }
                    }
                }
                
                contentItem: Text {
                    text: parent.text
                    font.pixelSize: Theme.fontSizeMedium
                    font.family: Theme.fontFamilyDefault
                    color: Theme.textSecondary
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }
                
                onClicked: {
                    // 重置所有下拉框到当前时间
                    var now = new Date()
                    startYearCombo.currentIndex = 0
                    startMonthCombo.currentIndex = now.getMonth()
                    startDayCombo.currentIndex = now.getDate() - 1
                    startHourCombo.currentIndex = now.getHours()
                    startMinuteCombo.currentIndex = now.getMinutes()
                    
                    endYearCombo.currentIndex = 0
                    endMonthCombo.currentIndex = now.getMonth()
                    endDayCombo.currentIndex = now.getDate() - 1
                    endHourCombo.currentIndex = now.getHours()
                    endMinuteCombo.currentIndex = now.getMinutes()
                    
                    root.timeRangeReset()
                    root.close()
                }
            }
            
            // 应用筛选按钮
            Button {
                id: applyButton
                text: tr.translate("dialog.applyFilter")
                implicitWidth: 120
                implicitHeight: 44
                
                background: Rectangle {
                    color: Theme.darkAccent
                    radius: Theme.radiusMedium
                    
                    // 悬停效果
                    opacity: applyButton.hovered ? 0.9 : 1.0
                    Behavior on opacity {
                        NumberAnimation { duration: Theme.animationDuration }
                    }
                }
                
                contentItem: Text {
                    text: parent.text
                    font.pixelSize: Theme.fontSizeMedium
                    font.family: Theme.fontFamilyDefault
                    color: Theme.textPrimary
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }
                
                onClicked: {
                    // 格式化开始时间
                    var startYear = startYearCombo.currentText
                    var startMonth = startMonthCombo.currentText
                    var startDay = startDayCombo.currentText
                    var startHour = startHourCombo.currentText
                    var startMinute = startMinuteCombo.currentText
                    
                    // 补零
                    if (startMonth < 10) startMonth = "0" + startMonth
                    if (startDay < 10) startDay = "0" + startDay
                    if (startHour < 10) startHour = "0" + startHour
                    if (startMinute < 10) startMinute = "0" + startMinute
                    
                    var startTimeStr = startYear + "/" + startMonth + "/" + startDay + " " + startHour + ":" + startMinute
                    
                    // 格式化结束时间
                    var endYear = endYearCombo.currentText
                    var endMonth = endMonthCombo.currentText
                    var endDay = endDayCombo.currentText
                    var endHour = endHourCombo.currentText
                    var endMinute = endMinuteCombo.currentText
                    
                    // 补零
                    if (endMonth < 10) endMonth = "0" + endMonth
                    if (endDay < 10) endDay = "0" + endDay
                    if (endHour < 10) endHour = "0" + endHour
                    if (endMinute < 10) endMinute = "0" + endMinute
                    
                    var endTimeStr = endYear + "/" + endMonth + "/" + endDay + " " + endHour + ":" + endMinute
                    
                    root.timeRangeApplied(startTimeStr, endTimeStr)
                    root.close()
                }
            }
        }
    }
}