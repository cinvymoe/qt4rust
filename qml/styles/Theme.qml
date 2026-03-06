// Theme.qml - 主题配置
pragma Singleton
import QtQuick

QtObject {
    // 颜色配置 - 深色主题
    readonly property color darkBackground: "#0f172b"        // 主背景色
    readonly property color darkSurface: "#1d293d"           // 卡片/表面色
    readonly property color darkBorder: "#314158"            // 边框色
    readonly property color darkAccent: "#2b7fff"            // 强调色（蓝色）
    
    // 颜色配置 - 状态色
    readonly property color dangerColor: "#e7000b"           // 危险色（红色）
    readonly property color dangerBackground: "#460809"      // 危险背景色
    readonly property color dangerLight: "#fb2c36"           // 危险色（浅）
    readonly property color warningColor: "#f0b100"          // 警告色（黄色）
    readonly property color successColor: "#00c950"          // 成功色（绿色）
    
    // 颜色配置 - 文本色
    readonly property color textPrimary: "#ffffff"           // 主文本色（白色）
    readonly property color textSecondary: "#cad5e2"         // 次要文本色
    readonly property color textTertiary: "#90a1b9"          // 三级文本色
    readonly property color textAccent: "#51a2ff"            // 强调文本色
    
    // 颜色配置 - 浅色主题（保留原有）
    readonly property color backgroundColor: "#f0f0f0"
    readonly property color primaryColor: "#2196F3"
    readonly property color textColor: "#333333"
    readonly property color secondaryTextColor: "#666666"
    
    // 字体配置
    readonly property int fontSizeTiny: 12                   // 极小字体
    readonly property int fontSizeSmall: 14                  // 小字体
    readonly property int fontSizeMedium: 16                 // 中等字体
    readonly property int fontSizeNormal: 18                 // 常规字体
    readonly property int fontSizeLarge: 20                  // 大字体
    readonly property int fontSizeXLarge: 24                 // 超大字体
    readonly property int fontSizeXXLarge: 30                // 特大字体
    readonly property int fontSizeHuge: 36                   // 巨大字体
    readonly property int fontSizeDisplay: 60                // 显示字体
    
    // 字体家族
    readonly property string fontFamilyDefault: "Inter"
    readonly property string fontFamilyMono: "Consolas"
    
    // 间距配置
    readonly property int spacingTiny: 4                     // 极小间距
    readonly property int spacingSmall: 8                    // 小间距
    readonly property int spacingMedium: 16                  // 中等间距
    readonly property int spacingLarge: 24                   // 大间距
    readonly property int spacingXLarge: 32                  // 超大间距
    
    // 圆角配置
    readonly property int radiusSmall: 4
    readonly property int radiusMedium: 10
    readonly property int radiusLarge: 22369600              // 完全圆角
    
    // 边框配置
    readonly property real borderThin: 0.667
    readonly property int borderNormal: 1
    readonly property int borderThick: 2
    
    // 尺寸配置
    readonly property int iconSizeSmall: 20
    readonly property int iconSizeMedium: 24
    readonly property int iconSizeLarge: 48
    
    readonly property int buttonHeightSmall: 32
    readonly property int buttonHeightMedium: 40
    
    readonly property int headerHeight: 68
    readonly property int navigationHeight: 68
    
    // 动画配置
    readonly property int animationDuration: 250
    
    // 不透明度配置
    readonly property real opacityDisabled: 0.3
    readonly property real opacityMedium: 0.62
}
