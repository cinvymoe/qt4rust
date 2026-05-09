// MomentCurveView.qml - 力矩曲线子页面
import QtQuick
import QtQuick.Controls
import "../../styles"
import "../../components/controls"
import "../../components/dialogs"
import qt.rust.demo

Flickable {
    id: momentCurveView
    
    property int _localeVersion: TranslationBridge.locale_version
    width: parent.width
    height: parent.height
    contentHeight: contentColumn.height
    clip: true


    // ViewModel 实例
    MomentCurveViewModel {
        id: viewModel
        Component.onCompleted: {
            console.log("MomentCurveViewModel initialized")
            console.log("Available properties:")
            for (var prop in viewModel) {
                console.log("  -", prop, ":", typeof viewModel[prop])
            }
            loadData()
            console.log("After loadData()")
            console.log("boom_length_list:", boom_length_list)
            console.log("data_loaded:", data_loaded)
        }
    }
    
    // 文件选择对话框
    FileSelectDialog {
        id: fileDialog
        
        onFileSelected: function(filePath) {
            console.log("Selected file:", filePath)
            
            var success = viewModel.importCurveFromFile(filePath)
            var message = viewModel.getImportStatusMessage()
            
            if (success) {
                importSuccessDialog.message = message
                importSuccessDialog.open()
                loadCurveChart.updateChart()
            } else {
                importErrorDialog.message = message
                importErrorDialog.open()
            }
        }
    }
    
    // 导入成功对话框
    InfoDialog {
        id: importSuccessDialog
        title: { const _ = momentCurveView._localeVersion; return TranslationBridge.translate("dialog.importSuccess") || "导入成功" }
        message: ""

        onAccepted: {
            close()
        }
    }

    // 导入失败对话框
    InfoDialog {
        id: importErrorDialog
        title: { const _ = momentCurveView._localeVersion; return TranslationBridge.translate("dialog.importError") || "导入失败" }
        message: ""

        onAccepted: {
            close()
        }
    }
    
    Column {
        id: contentColumn
        width: parent.width
        
        Item {
            width: parent.width
            height: childrenRect.height
            
            Column {
                anchors.horizontalCenter: parent.horizontalCenter
                width: parent.width - 200
                spacing: Theme.spacingMedium
                topPadding: Theme.spacingMedium
                bottomPadding: Theme.spacingMedium
                
                // 1. 说明卡片
                Rectangle {
                    width: parent.width
                    height: 153.333
                    color: "#162456"
                    border.color: "#1447e6"
                    border.width: Theme.borderThin
                    radius: Theme.radiusMedium
                    
                    Column {
                        anchors.fill: parent
                        anchors.topMargin: 16.667
                        anchors.leftMargin: 16.667
                        anchors.rightMargin: 16.667
                        spacing: 0
                        
                        Row {
                            width: parent.width
                            spacing: 12
                            
                            Text {
                                text: "ℹ"
                                font.pixelSize: Theme.fontSizeLarge
                                color: "#dbeafe"
                                anchors.verticalCenter: parent.verticalCenter
                            }
                            
                            Text {
                                text: { const _ = momentCurveView._localeVersion; return TranslationBridge.translate("momentCurve.title") }
                                font.pixelSize: Theme.fontSizeSmall
                                font.family: Theme.fontFamilyDefault
                                color: "#dbeafe"
                                anchors.verticalCenter: parent.verticalCenter
                            }
                        }
                        
                        Column {
                            width: parent.width
                            spacing: 4
                            topPadding: Theme.spacingSmall
                            opacity: 0.8
                            
                            Repeater {
                                model: [
                                    "• 曲线显示不同工作半径下的额定载荷能力",
                                    "• 实际作业时，载荷必须低于对应半径的额定值",
                                    "• 工作半径越大，额定载荷越小",
                                    "• 不同臂长配置对应不同的性能曲线"
                                ]
                                
                                Text {
                                    text: modelData
                                    font.pixelSize: Theme.fontSizeSmall
                                    font.family: Theme.fontFamilyDefault
                                    color: "#bedbff"
                                    width: parent.width
                                }
                            }
                        }
                    }
                }
                
                // 2. 额定载荷曲线
                Rectangle {
                    width: parent.width
                    height: childrenRect.height + 20
                    color: Theme.darkSurface
                    border.color: Theme.darkBorder
                    border.width: Theme.borderThin
                    radius: 12
                    
                    Column {
                        width: parent.width
                        spacing: 20
                        topPadding: 20
                        bottomPadding: 20
                        leftPadding: 20
                        rightPadding: 20
                        
                        // 标题
                        Item {
                            width: parent.width - 40
                            height: 36
                            
                            Row {
                                anchors.left: parent.left
                                anchors.verticalCenter: parent.verticalCenter
                                spacing: 12
                                
                                Rectangle {
                                    width: 4
                                    height: 28
                                    color: Theme.successColor
                                    radius: 2
                                    anchors.verticalCenter: parent.verticalCenter
                                }
                                
                                Text {
                                    text: { const _ = momentCurveView._localeVersion; return TranslationBridge.translate("momentCurve.ratedCurve") }
                                    font.pixelSize: 20
                                    font.family: Theme.fontFamilyDefault
                                    font.weight: Font.Bold
                                    color: Theme.textPrimary
                                    anchors.verticalCenter: parent.verticalCenter
                                }
                            }
                            
                            // 导入按钮
                            Button {
                                anchors.right: parent.right
                                anchors.verticalCenter: parent.verticalCenter
                                width: 100
                                height: 36
                                
                                background: Rectangle {
                                    color: parent.hovered ? "#1e5dff" : "#155dfc"
                                    border.color: "#1447e6"
                                    border.width: 1
                                    radius: 8
                                    
                                    Behavior on color { ColorAnimation { duration: 150 } }
                                }
                                
                                contentItem: Text {
                                    text: { const _ = momentCurveView._localeVersion; return TranslationBridge.translate("momentCurve.importCurve") }
                                    font.pixelSize: 14
                                    font.family: Theme.fontFamilyDefault
                                    color: "#ffffff"
                                    horizontalAlignment: Text.AlignHCenter
                                    verticalAlignment: Text.AlignVCenter
                                }
                                
                                onClicked: {
                                    fileDialog.open()
                                }
                            }
                        }
                        
                        // 图表
                        LoadCurveChart {
                            id: loadCurveChart
                            width: parent.width - 40
                            height: 400
                            viewModel: viewModel
                        }
                        
                        // 臂长选择
                        Column {
                            width: parent.width - 40
                            spacing: 16
                            
                            Flow {
                                width: parent.width
                                spacing: 12
                                
                                Repeater {
                                    model: viewModel.boom_length_list
                                    
                                    Button {
                                        width: 110
                                        height: 56
                                        
                                        property bool isSelected: viewModel.selected_boom_index === index
                                        
                                        background: Rectangle {
                                            color: isSelected ? "#155dfc" : Theme.darkBackground
                                            border.color: isSelected ? "#1447e6" : "#2d3748"
                                            border.width: 2
                                            radius: 8
                                            
                                            Rectangle {
                                                anchors.fill: parent
                                                anchors.margins: -2
                                                color: "transparent"
                                                border.color: isSelected ? "#60a5fa" : "transparent"
                                                border.width: 2
                                                radius: 10
                                                opacity: 0.3
                                                visible: isSelected
                                            }
                                            
                                            Behavior on color { ColorAnimation { duration: 200 } }
                                            Behavior on border.color { ColorAnimation { duration: 200 } }
                                        }
                                        
                                        contentItem: Column {
                                            anchors.centerIn: parent
                                            spacing: 4
                                            
                                            Text {
                                                text: modelData + "m"
                                                font.pixelSize: 18
                                                font.family: Theme.fontFamilyDefault
                                                font.weight: isSelected ? Font.Bold : Font.Medium
                                                color: isSelected ? "#ffffff" : Theme.textSecondary
                                                horizontalAlignment: Text.AlignHCenter
                                                anchors.horizontalCenter: parent.horizontalCenter
                                                Behavior on color { ColorAnimation { duration: 200 } }
                                            }
                                            
                                            Text {
                                                text: { const _ = momentCurveView._localeVersion; return TranslationBridge.translate("momentCurve.boomLength") }
                                                font.pixelSize: 12
                                                font.family: Theme.fontFamilyDefault
                                                color: isSelected ? "#93c5fd" : Theme.textTertiary
                                                horizontalAlignment: Text.AlignHCenter
                                                anchors.horizontalCenter: parent.horizontalCenter
                                                Behavior on color { ColorAnimation { duration: 200 } }
                                            }
                                        }
                                        
                                        onClicked: {
                                            viewModel.selectBoomByIndex(index)
                                            loadCurveChart.updateChart()
                                        }
                                        
                                        scale: hovered ? 1.05 : 1.0
                                        Behavior on scale { NumberAnimation { duration: 150 } }
                                    }
                                }
                            }
                            
                            // 统计信息卡片
                            Rectangle {
                                width: parent.width
                                height: 100
                                color: "#1a202c"
                                border.color: "#2d3748"
                                border.width: 1
                                radius: 12
                                visible: viewModel.data_loaded
                                
                                Rectangle {
                                    anchors.fill: parent
                                    radius: parent.radius
                                    gradient: Gradient {
                                        GradientStop { position: 0.0; color: "#1e293b" }
                                        GradientStop { position: 1.0; color: "#0f172a" }
                                    }
                                    opacity: 0.5
                                }
                                
                                Row {
                                    anchors.fill: parent
                                    anchors.margins: 20
                                    spacing: 20
                                    
                                    Column {
                                        width: (parent.width - 3 * 20) / 4
                                        spacing: 6
                                        anchors.verticalCenter: parent.verticalCenter
                                        
                                        Row {
                                            spacing: 6
                                            Rectangle {
                                                width: 6
                                                height: 6
                                                radius: 3
                                                color: "#3b82f6"
                                                anchors.verticalCenter: parent.verticalCenter
                                            }
                                            Text {
                                                text: { const _ = momentCurveView._localeVersion; return TranslationBridge.translate("momentCurve.currentLength") }
                                                font.pixelSize: 12
                                                color: "#94a3b8"
                                            }
                                        }
                                        Text {
                                            text: (viewModel.current_boom_length || 0).toFixed(1) + " m"
                                            font.pixelSize: 22
                                            font.weight: Font.Bold
                                            color: "#3b82f6"
                                        }
                                    }
                                    
                                    Rectangle { width: 1; height: parent.height - 20; color: "#2d3748"; anchors.verticalCenter: parent.verticalCenter }
                                    
                                    Column {
                                        width: (parent.width - 3 * 20) / 4
                                        spacing: 6
                                        anchors.verticalCenter: parent.verticalCenter
                                        
                                        Row {
                                            spacing: 6
                                            Rectangle { width: 6; height: 6; radius: 3; color: "#22c55e"; anchors.verticalCenter: parent.verticalCenter }
                                            Text { 
                                                text: { const _ = momentCurveView._localeVersion; return TranslationBridge.translate("momentCurve.maxLoad") }
                                                font.pixelSize: 12
                                                color: "#94a3b8"
                                            }
                                        }
                                        Text {
                                            text: viewModel.getMaxLoadForBoom(viewModel.current_boom_length || 0).toFixed(1) + " t"
                                            font.pixelSize: 22
                                            font.weight: Font.Bold
                                            color: "#22c55e"
                                        }
                                    }
                                    
                                    Rectangle { width: 1; height: parent.height - 20; color: "#2d3748"; anchors.verticalCenter: parent.verticalCenter }
                                    
                                    Column {
                                        width: (parent.width - 3 * 20) / 4
                                        spacing: 6
                                        anchors.verticalCenter: parent.verticalCenter
                                        
                                        Row {
                                            spacing: 6
                                            Rectangle { width: 6; height: 6; radius: 3; color: "#f59e0b"; anchors.verticalCenter: parent.verticalCenter }
                                            Text { 
                                                text: { const _ = momentCurveView._localeVersion; return TranslationBridge.translate("momentCurve.maxRadius") }
                                                font.pixelSize: 12
                                                color: "#94a3b8"
                                            }
                                        }
                                        Text {
                                            text: viewModel.getMaxRadiusForBoom(viewModel.current_boom_length || 0).toFixed(1) + " m"
                                            font.pixelSize: 22
                                            font.weight: Font.Bold
                                            color: "#f59e0b"
                                        }
                                    }
                                    
                                    Rectangle { width: 1; height: parent.height - 20; color: "#2d3748"; anchors.verticalCenter: parent.verticalCenter }
                                    
                                    Column {
                                        width: (parent.width - 3 * 20) / 4
                                        spacing: 6
                                        anchors.verticalCenter: parent.verticalCenter
                                        
                                        Row {
                                            spacing: 6
                                            Rectangle { width: 6; height: 6; radius: 3; color: "#8b5cf6"; anchors.verticalCenter: parent.verticalCenter }
                                            Text { 
                                                text: { const _ = momentCurveView._localeVersion; return TranslationBridge.translate("momentCurve.dataPoints") }
                                                font.pixelSize: 12
                                                color: "#94a3b8"
                                            }
                                        }
                                        Text {
                                            text: viewModel.getDataPointCount(viewModel.current_boom_length || 0).toString()
                                            font.pixelSize: 22
                                            font.weight: Font.Bold
                                            color: "#8b5cf6"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
