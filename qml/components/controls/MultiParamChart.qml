// MultiParamChart.qml - 多参数对比分析图组件
import qt.rust.demo
import QtQuick
import "../../styles"

Rectangle {
    id: root
    
    TranslationBridge { id: tr }
    
    // 公开属性
    property var timeLabels: []
    property var radiusData: []
    property var angleData: []
    property real maxValue: 80
    
    color: Theme.darkSurface
    border.color: Theme.darkBorder
    border.width: Theme.borderThin
    radius: Theme.radiusMedium
    
    Column {
        anchors.fill: parent
        anchors.margins: Theme.spacingMedium
        spacing: Theme.spacingMedium
        
        // 标题
        Row {
            spacing: Theme.spacingMedium
            
            Rectangle {
                width: 4
                height: Theme.fontSizeLarge
                color: "#ad46ff"
                anchors.verticalCenter: parent.verticalCenter
            }
            
            Text {
                text: tr.translate("chart.multiParam")
                font.pixelSize: Theme.fontSizeLarge
                font.family: Theme.fontFamilyDefault
                font.weight: Font.Medium
                color: Theme.textPrimary
                anchors.verticalCenter: parent.verticalCenter
            }
        }
        
        // 图表容器
        Rectangle {
            width: parent.width
            height: 320
            color: Theme.darkBackground
            radius: Theme.radiusMedium
            
            Canvas {
                id: multiCanvas
                anchors.fill: parent
                anchors.margins: Theme.spacingLarge
                
                onPaint: {
                    if (radiusData.length === 0 || angleData.length === 0) return
                    
                    var ctx = getContext("2d")
                    ctx.clearRect(0, 0, width, height)
                    
                    var padding = 40
                    var chartWidth = width - padding * 2
                    var chartHeight = height - padding * 2
                    var dataCount = radiusData.length
                    
                    // 绘制网格线
                    ctx.strokeStyle = Theme.darkBorder
                    ctx.lineWidth = 1
                    for (var i = 0; i <= 4; i++) {
                        var y = padding + (chartHeight / 4) * i
                        ctx.beginPath()
                        ctx.moveTo(padding, y)
                        ctx.lineTo(padding + chartWidth, y)
                        ctx.stroke()
                    }
                    
                    // 绘制工作半径曲线
                    ctx.strokeStyle = "#00b8db"
                    ctx.lineWidth = 2
                    ctx.beginPath()
                    
                    for (var j = 0; j < dataCount; j++) {
                        var x = padding + (chartWidth / (dataCount - 1)) * j
                        var value = radiusData[j] / maxValue
                        y = padding + chartHeight * (1 - value)
                        if (j === 0) {
                            ctx.moveTo(x, y)
                        } else {
                            ctx.lineTo(x, y)
                        }
                    }
                    ctx.stroke()
                    
                    // 绘制吊臂角度曲线
                    ctx.strokeStyle = "#ff6900"
                    ctx.beginPath()
                    
                    for (j = 0; j < dataCount; j++) {
                        x = padding + (chartWidth / (dataCount - 1)) * j
                        value = angleData[j] / maxValue
                        y = padding + chartHeight * (1 - value)
                        if (j === 0) {
                            ctx.moveTo(x, y)
                        } else {
                            ctx.lineTo(x, y)
                        }
                    }
                    ctx.stroke()
                    
                    // Y轴标签
                    ctx.fillStyle = Theme.textTertiary
                    ctx.font = "12px " + Theme.fontFamilyDefault
                    ctx.textAlign = "right"
                    for (i = 0; i <= 4; i++) {
                        y = padding + (chartHeight / 4) * i
                        var label = (maxValue - i * 20).toString()
                        ctx.fillText(label, padding - 10, y + 4)
                    }
                    
                    // X轴时间标签
                    ctx.textAlign = "center"
                    ctx.fillStyle = Theme.textTertiary
                    var labelStep = Math.floor(dataCount / 5)
                    for (i = 0; i < dataCount; i += labelStep) {
                        x = padding + (chartWidth / (dataCount - 1)) * i
                        ctx.fillText(timeLabels[i], x, padding + chartHeight + 20)
                    }
                }
                
                Component.onCompleted: requestPaint()
                
                Connections {
                    target: root
                    function onRadiusDataChanged() { multiCanvas.requestPaint() }
                    function onAngleDataChanged() { multiCanvas.requestPaint() }
                    function onTimeLabelsChanged() { multiCanvas.requestPaint() }
                }
            }
        }
        
        // 图例
        Row {
            anchors.horizontalCenter: parent.horizontalCenter
            spacing: Theme.spacingLarge
            
            // 工作半径图例
            Row {
                spacing: Theme.spacingSmall
                
                Rectangle {
                    width: 24
                    height: 2
                    color: "#00b8db"
                    anchors.verticalCenter: parent.verticalCenter
                }
                
                Text {
                    text: tr.translate("chart.workingRadius")
                    font.pixelSize: Theme.fontSizeSmall
                    font.family: Theme.fontFamilyDefault
                    color: Theme.textSecondary
                    anchors.verticalCenter: parent.verticalCenter
                }
            }
            
            // 吊臂角度图例
            Row {
                spacing: Theme.spacingSmall
                
                Rectangle {
                    width: 24
                    height: 2
                    color: "#ff6900"
                    anchors.verticalCenter: parent.verticalCenter
                }
                
                Text {
                    text: tr.translate("chart.boomAngle")
                    font.pixelSize: Theme.fontSizeSmall
                    font.family: Theme.fontFamilyDefault
                    color: Theme.textSecondary
                    anchors.verticalCenter: parent.verticalCenter
                }
            }
        }
    }
}