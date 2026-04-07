// MomentCurveView.qml - 力矩曲线子页面
// 从 rated_load_table.csv 加载数据并显示
import QtQuick
import QtQuick.Controls
import "../../styles"
import "../../components/controls"
import qt.rust.demo 1.0

Flickable {
    id: momentCurveView
    width: parent.width
    height: parent.height
    contentHeight: contentColumn.height
    clip: true
    
    // ViewModel 实例
    MomentCurveViewModel {
        id: viewModel
        
        Component.onCompleted: {
            loadData()
        }
    }
    
    // 臂长颜色配置
    property var boomColors: ["#22c55e", "#3b82f6", "#f59e0b", "#ec4899", "#8b5cf6", "#14b8a6"]
    
    // 获取臂长的颜色
    function getBoomColor(index) {
        return boomColors[index % boomColors.length]
    }
    
    // 获取当前选中的臂长颜色
    function getCurrentBoomColor() {
        return getBoomColor(viewModel.selected_boom_index)
    }
    
    // 解析 JSON 数据为 QML 数组
    function parseCurveData(jsonString) {
        try {
            var data = JSON.parse(jsonString)
            return data
        } catch (e) {
            console.error("Failed to parse curve data:", e)
            return []
        }
    }
    
    // 获取当前臂长的曲线数据
    function getCurrentCurveData() {
        var json = viewModel.getCurveDataJson(viewModel.current_boom_length)
        return parseCurveData(json)
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
                                text: "力矩曲线图说明："
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
                                    "• 点击下方按钮切换不同臂长的性能曲线"
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
                    height: childrenRect.height + Theme.borderThin
                    color: Theme.darkSurface
                    border.color: Theme.darkBorder
                    border.width: Theme.borderThin
                    radius: Theme.radiusMedium
                    visible: viewModel.data_loaded
                    
                    Column {
                        width: parent.width
                        spacing: Theme.spacingMedium
                        topPadding: 16.667
                        bottomPadding: Theme.borderThin
                        leftPadding: 16.667
                        rightPadding: 16.667
                        
                        // 标题
                        Item {
                            width: parent.width - 2 * 16.667
                            height: 40
                            
                            Row {
                                spacing: 12
                                anchors.left: parent.left
                                anchors.verticalCenter: parent.verticalCenter
                                
                                Rectangle {
                                    width: 4
                                    height: 24
                                    color: Theme.successColor
                                    anchors.verticalCenter: parent.verticalCenter
                                }
                                
                                Text {
                                    text: "额定载荷曲线"
                                    font.pixelSize: Theme.fontSizeLarge
                                    font.family: Theme.fontFamilyDefault
                                    color: Theme.textPrimary
                                    anchors.verticalCenter: parent.verticalCenter
                                }
                            }
                        }
                        
                        // 臂长选择按钮组
                        Row {
                            width: parent.width - 2 * 16.667
                            height: 48
                            spacing: Theme.spacingSmall
                            visible: viewModel.boom_length_list.length > 0
                            
                            Repeater {
                                model: viewModel.boom_length_list
                                
                                Button {
                                    width: (parent.width - (viewModel.boom_length_list.length - 1) * Theme.spacingSmall) / viewModel.boom_length_list.length
                                    height: parent.height
                                    
                                    property bool isSelected: index === viewModel.selected_boom_index
                                    property string boomColor: getBoomColor(index)
                                    
                                    background: Rectangle {
                                        color: isSelected ? boomColor : Theme.darkBackground
                                        border.color: boomColor
                                        border.width: isSelected ? 0 : 2
                                        radius: Theme.radiusMedium
                                        
                                        Behavior on color {
                                            ColorAnimation { duration: 150 }
                                        }
                                    }
                                    
                                    contentItem: Text {
                                        text: modelData + "m"
                                        font.pixelSize: Theme.fontSizeMedium
                                        font.weight: Font.Medium
                                        font.family: Theme.fontFamilyDefault
                                        color: isSelected ? "#ffffff" : boomColor
                                        horizontalAlignment: Text.AlignHCenter
                                        verticalAlignment: Text.AlignVCenter
                                    }
                                    
                                    onClicked: {
                                        viewModel.selectBoomByIndex(index)
                                        loadCurveChart.requestRepaint()
                                    }
                                }
                            }
                        }
                        
                        // 图表
                        LoadCurveChart {
                            id: loadCurveChart
                            width: parent.width - 2 * 16.667
                            height: 384
                            curveData: getCurrentCurveData()
                            curveColor: getCurrentBoomColor()
                            boomLength: viewModel.current_boom_length
                            maxX: viewModel.getGlobalMaxRadius()
                            maxY: viewModel.getGlobalMaxLoad()
                        }
                        
                        // 图例 - 显示当前选中臂长的详细信息
                        Rectangle {
                            width: parent.width - 2 * 16.667
                            height: 80
                            color: Theme.darkBackground
                            radius: Theme.radiusSmall
                            
                            Row {
                                anchors.fill: parent
                                anchors.margins: 16
                                spacing: Theme.spacingLarge
                                
                                // 当前曲线指示
                                Row {
                                    width: parent.width * 0.4
                                    height: parent.height
                                    spacing: Theme.spacingSmall
                                    
                                    Rectangle {
                                        width: 20
                                        height: 4
                                        color: getCurrentBoomColor()
                                        radius: Theme.radiusSmall
                                        anchors.verticalCenter: parent.verticalCenter
                                    }
                                    
                                    Column {
                                        anchors.verticalCenter: parent.verticalCenter
                                        spacing: 4
                                        
                                        Text {
                                            text: viewModel.current_boom_length.toFixed(1) + "米臂长"
                                            font.pixelSize: Theme.fontSizeMedium
                                            font.weight: Font.Medium
                                            font.family: Theme.fontFamilyDefault
                                            color: Theme.textPrimary
                                        }
                                        
                                        Text {
                                            text: "最大载荷: " + viewModel.getMaxLoadForBoom(viewModel.current_boom_length).toFixed(1) + 
                                                  "吨 / 最大幅度: " + viewModel.getMaxRadiusForBoom(viewModel.current_boom_length).toFixed(1) + "米"
                                            font.pixelSize: Theme.fontSizeSmall
                                            font.family: Theme.fontFamilyDefault
                                            color: Theme.textSecondary
                                        }
                                    }
                                }
                                
                                // 统计信息
                                Row {
                                    width: parent.width * 0.6
                                    height: parent.height
                                    spacing: Theme.spacingMedium
                                    
                                    // 数据点数量
                                    Rectangle {
                                        width: (parent.width - 2 * Theme.spacingMedium) / 3
                                        height: parent.height
                                        color: "transparent"
                                        
                                        Column {
                                            anchors.centerIn: parent
                                            spacing: 2
                                            
                                            Text {
                                                text: viewModel.getDataPointCount(viewModel.current_boom_length)
                                                font.pixelSize: Theme.fontSizeXLarge
                                                font.weight: Font.Bold
                                                font.family: Theme.fontFamilyDefault
                                                color: getCurrentBoomColor()
                                                horizontalAlignment: Text.AlignHCenter
                                                anchors.horizontalCenter: parent.horizontalCenter
                                            }
                                            
                                            Text {
                                                text: "数据点"
                                                font.pixelSize: Theme.fontSizeTiny
                                                font.family: Theme.fontFamilyDefault
                                                color: Theme.textTertiary
                                                horizontalAlignment: Text.AlignHCenter
                                                anchors.horizontalCenter: parent.horizontalCenter
                                            }
                                        }
                                    }
                                    
                                    // 平均载荷
                                    Rectangle {
                                        width: (parent.width - 2 * Theme.spacingMedium) / 3
                                        height: parent.height
                                        color: "transparent"
                                        
                                        Column {
                                            anchors.centerIn: parent
                                            spacing: 2
                                            
                                            Text {
                                                text: viewModel.getAverageLoad(viewModel.current_boom_length).toFixed(1)
                                                font.pixelSize: Theme.fontSizeXLarge
                                                font.weight: Font.Bold
                                                font.family: Theme.fontFamilyDefault
                                                color: Theme.textPrimary
                                                horizontalAlignment: Text.AlignHCenter
                                                anchors.horizontalCenter: parent.horizontalCenter
                                            }
                                            
                                            Text {
                                                text: "平均载荷(吨)"
                                                font.pixelSize: Theme.fontSizeTiny
                                                font.family: Theme.fontFamilyDefault
                                                color: Theme.textTertiary
                                                horizontalAlignment: Text.AlignHCenter
                                                anchors.horizontalCenter: parent.horizontalCenter
                                            }
                                        }
                                    }
                                    
                                    // 载荷范围
                                    Rectangle {
                                        width: (parent.width - 2 * Theme.spacingMedium) / 3
                                        height: parent.height
                                        color: "transparent"
                                        
                                        Column {
                                            anchors.centerIn: parent
                                            spacing: 2
                                            
                                            Text {
                                                text: viewModel.getLoadRange(viewModel.current_boom_length)
                                                font.pixelSize: Theme.fontSizeLarge
                                                font.weight: Font.Bold
                                                font.family: Theme.fontFamilyDefault
                                                color: Theme.textPrimary
                                                horizontalAlignment: Text.AlignHCenter
                                                anchors.horizontalCenter: parent.horizontalCenter
                                            }
                                            
                                            Text {
                                                text: "载荷范围(吨)"
                                                font.pixelSize: Theme.fontSizeTiny
                                                font.family: Theme.fontFamilyDefault
                                                color: Theme.textTertiary
                                                horizontalAlignment: Text.AlignHCenter
                                                anchors.horizontalCenter: parent.horizontalCenter
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                // 加载中提示
                Rectangle {
                    width: parent.width
                    height: 500
                    color: Theme.darkSurface
                    border.color: Theme.darkBorder
                    border.width: Theme.borderThin
                    radius: Theme.radiusMedium
                    visible: !viewModel.data_loaded && viewModel.error_message === ""
                    
                    Column {
                        anchors.centerIn: parent
                        spacing: Theme.spacingMedium
                        
                        Text {
                            text: "⏳ 正在加载额定载荷数据..."
                            font.pixelSize: Theme.fontSizeLarge
                            font.family: Theme.fontFamilyDefault
                            color: Theme.textSecondary
                            anchors.horizontalCenter: parent.horizontalCenter
                        }
                        
                        Text {
                            text: "配置文件: config/rated_load_table.csv"
                            font.pixelSize: Theme.fontSizeSmall
                            font.family: Theme.fontFamilyDefault
                            color: Theme.textTertiary
                            anchors.horizontalCenter: parent.horizontalCenter
                        }
                    }
                }
                
                // 错误提示
                Rectangle {
                    width: parent.width
                    height: 200
                    color: Theme.dangerBackground
                    border.color: Theme.dangerColor
                    border.width: Theme.borderThick
                    radius: Theme.radiusMedium
                    visible: viewModel.error_message !== ""
                    
                    Column {
                        anchors.centerIn: parent
                        spacing: Theme.spacingSmall
                        
                        Text {
                            text: "❌ 数据加载失败"
                            font.pixelSize: Theme.fontSizeLarge
                            font.weight: Font.Bold
                            font.family: Theme.fontFamilyDefault
                            color: Theme.dangerColor
                            anchors.horizontalCenter: parent.horizontalCenter
                        }
                        
                        Text {
                            text: viewModel.error_message
                            font.pixelSize: Theme.fontSizeMedium
                            font.family: Theme.fontFamilyDefault
                            color: Theme.textSecondary
                            anchors.horizontalCenter: parent.horizontalCenter
                            wrapMode: Text.WordWrap
                            width: parent.width - 40
                            horizontalAlignment: Text.AlignHCenter
                        }
                    }
                }
                
                // 3. 使用示例
                Rectangle {
                    width: parent.width
                    height: childrenRect.height + Theme.borderThin
                    color: Theme.darkSurface
                    border.color: Theme.darkBorder
                    border.width: Theme.borderThin
                    radius: Theme.radiusMedium
                    visible: viewModel.data_loaded
                    
                    Column {
                        width: parent.width
                        spacing: Theme.spacingMedium
                        topPadding: 16.667
                        bottomPadding: Theme.borderThin
                        leftPadding: 16.667
                        rightPadding: 16.667
                        
                        // 标题
                        Item {
                            width: parent.width - 2 * 16.667
                            height: 28
                            
                            Rectangle {
                                width: 4
                                height: 24
                                color: "#ff6900"
                                anchors.left: parent.left
                                anchors.verticalCenter: parent.verticalCenter
                            }
                            
                            Text {
                                text: "使用示例"
                                font.pixelSize: Theme.fontSizeLarge
                                font.family: Theme.fontFamilyDefault
                                color: Theme.textPrimary
                                anchors.left: parent.left
                                anchors.leftMargin: 12
                                anchors.verticalCenter: parent.verticalCenter
                            }
                        }
                        
                        // 示例卡片容器
                        Column {
                            width: parent.width - 2 * 16.667
                            spacing: 12
                            
                            // 安全作业示例
                            Rectangle {
                                width: parent.width
                                height: 148
                                color: Theme.darkBackground
                                border.color: Theme.successColor
                                border.width: 4
                                radius: Theme.radiusMedium
                                
                                Column {
                                    anchors.fill: parent
                                    anchors.topMargin: Theme.spacingMedium
                                    anchors.leftMargin: 20
                                    anchors.rightMargin: Theme.spacingMedium
                                    spacing: Theme.spacingSmall
                                    
                                    Text {
                                        text: "✓ 安全作业示例"
                                        font.pixelSize: Theme.fontSizeMedium
                                        font.family: Theme.fontFamilyDefault
                                        color: "#05df72"
                                        width: parent.width
                                    }
                                    
                                    Text {
                                        text: viewModel.current_boom_length.toFixed(1) + "米臂长，工作半径10m，吊运载荷12吨"
                                        font.pixelSize: Theme.fontSizeSmall
                                        font.family: Theme.fontFamilyDefault
                                        color: Theme.textSecondary
                                        width: parent.width
                                    }
                                    
                                    Column {
                                        width: parent.width
                                        spacing: 4
                                        
                                        Repeater {
                                            model: {
                                                var radiusLoad = 25
                                                var rate = ((12 / radiusLoad) * 100).toFixed(1)
                                                return [
                                                    "• 根据曲线，10m半径时额定载荷约" + radiusLoad + "吨",
                                                    "• 实际载荷12吨，低于额定载荷" + radiusLoad + "吨",
                                                    "• 载荷率为" + rate + "%，处于安全范围"
                                                ]
                                            }
                                            
                                            Text {
                                                text: modelData
                                                font.pixelSize: Theme.fontSizeTiny
                                                font.family: Theme.fontFamilyDefault
                                                color: Theme.textTertiary
                                                width: parent.width
                                            }
                                        }
                                    }
                                }
                            }
                            
                            // 预警作业示例
                            Rectangle {
                                width: parent.width
                                height: 148
                                color: Theme.darkBackground
                                border.color: Theme.warningColor
                                border.width: 4
                                radius: Theme.radiusMedium
                                
                                Column {
                                    anchors.fill: parent
                                    anchors.topMargin: Theme.spacingMedium
                                    anchors.leftMargin: 20
                                    anchors.rightMargin: Theme.spacingMedium
                                    spacing: Theme.spacingSmall
                                    
                                    Text {
                                        text: "⚠ 预警作业示例"
                                        font.pixelSize: Theme.fontSizeMedium
                                        font.family: Theme.fontFamilyDefault
                                        color: "#fdc700"
                                        width: parent.width
                                    }
                                    
                                    Text {
                                        text: viewModel.current_boom_length.toFixed(1) + "米臂长，工作半径15m，吊运载荷10吨"
                                        font.pixelSize: Theme.fontSizeSmall
                                        font.family: Theme.fontFamilyDefault
                                        color: Theme.textSecondary
                                        width: parent.width
                                    }
                                    
                                    Column {
                                        width: parent.width
                                        spacing: 4
                                        
                                        Repeater {
                                            model: {
                                                var radiusLoad = 15
                                                var rate = ((10 / radiusLoad) * 100).toFixed(1)
                                                return [
                                                    "• 根据曲线，15m半径时额定载荷约" + radiusLoad + "吨",
                                                    "• 实际载荷10吨，载荷率为" + rate + "%",
                                                    "• 超过75%预警线，建议减载或减小半径"
                                                ]
                                            }
                                            
                                            Text {
                                                text: modelData
                                                font.pixelSize: Theme.fontSizeTiny
                                                font.family: Theme.fontFamilyDefault
                                                color: Theme.textTertiary
                                                width: parent.width
                                            }
                                        }
                                    }
                                }
                            }
                            
                            // 危险作业示例
                            Rectangle {
                                width: parent.width
                                height: 148
                                color: Theme.darkBackground
                                border.color: Theme.dangerLight
                                border.width: 4
                                radius: Theme.radiusMedium
                                
                                Column {
                                    anchors.fill: parent
                                    anchors.topMargin: Theme.spacingMedium
                                    anchors.leftMargin: 20
                                    anchors.rightMargin: Theme.spacingMedium
                                    spacing: Theme.spacingSmall
                                    
                                    Text {
                                        text: "✗ 危险作业示例"
                                        font.pixelSize: Theme.fontSizeMedium
                                        font.family: Theme.fontFamilyDefault
                                        color: "#ff6467"
                                        width: parent.width
                                    }
                                    
                                    Text {
                                        text: viewModel.current_boom_length.toFixed(1) + "米臂长，工作半径20m，吊运载荷3吨"
                                        font.pixelSize: Theme.fontSizeSmall
                                        font.family: Theme.fontFamilyDefault
                                        color: Theme.textSecondary
                                        width: parent.width
                                    }
                                    
                                    Column {
                                        width: parent.width
                                        spacing: 4
                                        
                                        Repeater {
                                            model: {
                                                var radiusLoad = 3.2
                                                var rate = ((3 / radiusLoad) * 100).toFixed(1)
                                                return [
                                                    "• 根据曲线，20m半径时额定载荷约" + radiusLoad + "吨",
                                                    "• 实际载荷3吨，载荷率为" + rate + "%",
                                                    "• 超过90%危险线！必须立即减载"
                                                ]
                                            }
                                            
                                            Text {
                                                text: modelData
                                                font.pixelSize: Theme.fontSizeTiny
                                                font.family: Theme.fontFamilyDefault
                                                color: Theme.textTertiary
                                                width: parent.width
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
}
