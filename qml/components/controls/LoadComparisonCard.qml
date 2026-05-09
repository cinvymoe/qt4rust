// LoadComparisonCard.qml - 载荷对比卡片组件
import qt.rust.demo
import QtQuick
import "../../styles"

Rectangle {
    id: root
    
    
    // 公开属性
    property real currentLoad: 0.0    // 当前载荷（吨）
    property real ratedLoad: 0.0      // 额定载荷（吨）
    
    // 计算属性
    readonly property real loadRatio: ratedLoad > 0 ? (currentLoad / ratedLoad) : 0
    
    // 样式
    color: Theme.darkSurface
    border.color: Theme.darkBorder
    border.width: Theme.borderThin
    radius: Theme.radiusMedium
    
    Column {
        anchors.fill: parent
        anchors.margins: Theme.spacingLarge
        spacing: Theme.spacingMedium
        
        // 标题
        Text {
            text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("loadComparison.title") }
            font.pixelSize: Theme.fontSizeSmall
            font.family: Theme.fontFamilyDefault
            color: Theme.textSecondary
        }
        
        // 进度条容器
        Rectangle {
            width: parent.width
            height: 56
            radius: Theme.radiusMedium
            color: Theme.darkBorder
            
            // 进度条填充
            Rectangle {
                width: parent.width * root.loadRatio
                height: parent.height
                radius: Theme.radiusMedium
                color: Theme.successColor
                
                Behavior on width {
                    NumberAnimation {
                        duration: Theme.animationDuration
                    }
                }
            }
            
            // 文本标签
            Column {
                anchors.centerIn: parent
                spacing: 0
                
                Text {
                    text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("loadComparison.actual") + " " + root.currentLoad.toFixed(1) + "t" }
                    font.pixelSize: Theme.fontSizeTiny
                    font.family: Theme.fontFamilyMono
                    color: Theme.textPrimary
                    anchors.horizontalCenter: parent.horizontalCenter
                }
                
                Text {
                    text: { const _ = TranslationBridge.locale_version; return TranslationBridge.translate("loadComparison.rated") + " " + root.ratedLoad.toFixed(1) + "t" }
                    font.pixelSize: Theme.fontSizeTiny
                    font.family: Theme.fontFamilyMono
                    color: Theme.textSecondary
                    anchors.horizontalCenter: parent.horizontalCenter
                }
            }
        }
    }
}