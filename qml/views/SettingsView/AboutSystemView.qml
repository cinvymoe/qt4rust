// AboutSystemView.qml - 关于系统子页面
import QtQuick
import QtQuick.Controls
import "../../styles"

Flickable {
    id: aboutSystemView
    width: parent.width
    height: parent.height
    contentHeight: contentColumn.height + Theme.spacingMedium * 2
    clip: true
    
    Column {
        id: contentColumn
        width: parent.width - 200  // 左右各留 219px 边距 (从 Figma 提取)
        anchors.horizontalCenter: parent.horizontalCenter
        spacing: Theme.spacingLarge
        topPadding: Theme.spacingMedium
        
        // 系统信息卡片
        Rectangle {
            width: parent.width
            height: 321
            color: Theme.darkSurface
            border.color: Theme.darkBorder
            border.width: Theme.borderThin
            radius: Theme.radiusMedium
            
            Column {
                anchors.fill: parent
                anchors.margins: Theme.spacingLarge
                spacing: Theme.spacingLarge
                
                // Logo 和标题
                Row {
                    width: parent.width
                    spacing: Theme.spacingMedium
                    
                    // Logo 图标
                    Rectangle {
                        width: 64
                        height: 64
                        color: "#155dfc"
                        radius: Theme.radiusMedium
                        
                        Image {
                            anchors.centerIn: parent
                            width: 40
                            height: 40
                            source: "../../assets/images/icon-logo.png"
                            fillMode: Image.PreserveAspectFit
                        }
                    }
                    
                    // 标题文本
                    Column {
                        anchors.verticalCenter: parent.verticalCenter
                        spacing: 0
                        
                        Text {
                            text: "汽车吊力矩监测系统"
                            font.pixelSize: Theme.fontSizeXLarge
                            font.family: Theme.fontFamilyDefault
                            font.weight: Font.Medium
                            color: Theme.textPrimary
                        }
                        
                        Text {
                            text: "Crane Moment Monitoring System"
                            font.pixelSize: Theme.fontSizeMedium
                            font.family: Theme.fontFamilyDefault
                            color: Theme.textTertiary
                        }
                    }
                }
                
                // 版本信息网格
                Grid {
                    width: parent.width
                    columns: 2
                    rowSpacing: Theme.spacingMedium
                    columnSpacing: Theme.spacingMedium
                    
                    // 系统版本
                    Rectangle {
                        width: (parent.width - Theme.spacingMedium) / 2
                        height: 84
                        color: Qt.rgba(49/255, 65/255, 88/255, 0.5)
                        radius: Theme.radiusMedium
                        
                        Column {
                            anchors.left: parent.left
                            anchors.top: parent.top
                            anchors.margins: Theme.spacingMedium
                            spacing: Theme.spacingTiny
                            
                            Text {
                                text: "系统版本"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textTertiary
                            }
                            
                            Text {
                                text: "v2.5.3"
                                font.pixelSize: Theme.fontSizeLarge
                                font.family: Theme.fontFamilyMono
                                color: Theme.textPrimary
                            }
                        }
                    }
                    
                    // 发布日期
                    Rectangle {
                        width: (parent.width - Theme.spacingMedium) / 2
                        height: 84
                        color: Qt.rgba(49/255, 65/255, 88/255, 0.5)
                        radius: Theme.radiusMedium
                        
                        Column {
                            anchors.left: parent.left
                            anchors.top: parent.top
                            anchors.margins: Theme.spacingMedium
                            spacing: Theme.spacingTiny
                            
                            Text {
                                text: "发布日期"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textTertiary
                            }
                            
                            Text {
                                text: "2025-11-13"
                                font.pixelSize: Theme.fontSizeLarge
                                color: Theme.textPrimary
                            }
                        }
                    }
                    
                    // 固件版本
                    Rectangle {
                        width: (parent.width - Theme.spacingMedium) / 2
                        height: 84
                        color: Qt.rgba(49/255, 65/255, 88/255, 0.5)
                        radius: Theme.radiusMedium
                        
                        Column {
                            anchors.left: parent.left
                            anchors.top: parent.top
                            anchors.margins: Theme.spacingMedium
                            spacing: Theme.spacingTiny
                            
                            Text {
                                text: "固件版本"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textTertiary
                            }
                            
                            Text {
                                text: "v1.8.2"
                                font.pixelSize: Theme.fontSizeLarge
                                font.family: Theme.fontFamilyMono
                                color: Theme.textPrimary
                            }
                        }
                    }
                    
                    // 硬件版本
                    Rectangle {
                        width: (parent.width - Theme.spacingMedium) / 2
                        height: 84
                        color: Qt.rgba(49/255, 65/255, 88/255, 0.5)
                        radius: Theme.radiusMedium
                        
                        Column {
                            anchors.left: parent.left
                            anchors.top: parent.top
                            anchors.margins: Theme.spacingMedium
                            spacing: Theme.spacingTiny
                            
                            Text {
                                text: "硬件版本"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textTertiary
                            }
                            
                            Text {
                                text: "Rev.C"
                                font.pixelSize: Theme.fontSizeLarge
                                font.family: Theme.fontFamilyMono
                                color: Theme.textPrimary
                            }
                        }
                    }
                }
            }
        }
        
        // 系统特性卡片
        Rectangle {
            width: parent.width
            height: 261
            color: Theme.darkSurface
            border.color: Theme.darkBorder
            border.width: Theme.borderThin
            radius: Theme.radiusMedium
            
            Column {
                anchors.fill: parent
                anchors.margins: Theme.spacingLarge
                spacing: Theme.spacingMedium
                
                // 标题
                Row {
                    width: parent.width
                    spacing: Theme.spacingSmall
                    
                    Rectangle {
                        width: 4
                        height: 24
                        color: Theme.darkAccent
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    
                    Text {
                        text: "系统特性"
                        font.pixelSize: Theme.fontSizeLarge
                        color: Theme.textPrimary
                        anchors.verticalCenter: parent.verticalCenter
                    }
                }
                
                // 特性列表
                Column {
                    width: parent.width
                    spacing: Theme.spacingSmall
                    
                    // 实时安全监控
                    Row {
                        width: parent.width
                        spacing: Theme.spacingSmall
                        
                        Rectangle {
                            width: 32
                            height: 32
                            color: Qt.rgba(21/255, 93/255, 252/255, 0.2)
                            radius: Theme.radiusSmall
                            anchors.top: parent.top
                            anchors.topMargin: 2
                            
                            Image {
                                anchors.centerIn: parent
                                width: Theme.iconSizeSmall
                                height: Theme.iconSizeSmall
                                source: "../../assets/images/icon-sensor.svg"
                                fillMode: Image.PreserveAspectFit
                            }
                        }
                        
                        Column {
                            width: parent.width - 44
                            spacing: Theme.spacingTiny
                            
                            Text {
                                text: "实时安全监控"
                                font.pixelSize: Theme.fontSizeMedium
                                color: "#e2e8f0"
                            }
                            
                            Text {
                                text: "24/7不间断监测起重机力矩状态，确保作业安全"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textTertiary
                                wrapMode: Text.WordWrap
                            }
                        }
                    }
                    
                    // 三级预警系统
                    Row {
                        width: parent.width
                        spacing: Theme.spacingSmall
                        
                        Rectangle {
                            width: 32
                            height: 32
                            color: Qt.rgba(208/255, 135/255, 0/255, 0.2)
                            radius: Theme.radiusSmall
                            anchors.top: parent.top
                            anchors.topMargin: 2
                            
                            Image {
                                anchors.centerIn: parent
                                width: Theme.iconSizeSmall
                                height: Theme.iconSizeSmall
                                source: "../../assets/images/icon-alert.png"
                                fillMode: Image.PreserveAspectFit
                            }
                        }
                        
                        Column {
                            width: parent.width - 44
                            spacing: Theme.spacingTiny
                            
                            Text {
                                text: "三级预警系统"
                                font.pixelSize: Theme.fontSizeMedium
                                color: "#e2e8f0"
                            }
                            
                            Text {
                                text: "正常、预警、危险三级报警，提前预防安全事故"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textTertiary
                                wrapMode: Text.WordWrap
                            }
                        }
                    }
                    
                    // 高精度传感器
                    Row {
                        width: parent.width
                        spacing: Theme.spacingSmall
                        
                        Rectangle {
                            width: 32
                            height: 32
                            color: Qt.rgba(0/255, 166/255, 62/255, 0.2)
                            radius: Theme.radiusSmall
                            anchors.top: parent.top
                            anchors.topMargin: 2
                            
                            Image {
                                anchors.centerIn: parent
                                width: Theme.iconSizeSmall
                                height: Theme.iconSizeSmall
                                source: "../../assets/images/icon-gauge.png"
                                fillMode: Image.PreserveAspectFit
                            }
                        }
                        
                        Column {
                            width: parent.width - 44
                            spacing: Theme.spacingTiny
                            
                            Text {
                                text: "高精度传感器"
                                font.pixelSize: Theme.fontSizeMedium
                                color: "#e2e8f0"
                            }
                            
                            Text {
                                text: "采用工业级传感器，精度±0.5%，响应时间<50ms"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textTertiary
                                wrapMode: Text.WordWrap
                            }
                        }
                    }
                }
            }
        }
        
        // 技术规格卡片
        Rectangle {
            width: parent.width
            height: 333
            color: Theme.darkSurface
            border.color: Theme.darkBorder
            border.width: Theme.borderThin
            radius: Theme.radiusMedium
            
            Column {
                anchors.fill: parent
                anchors.margins: Theme.spacingLarge
                spacing: Theme.spacingMedium
                
                // 标题
                Row {
                    width: parent.width
                    spacing: Theme.spacingSmall
                    
                    Rectangle {
                        width: 4
                        height: 24
                        color: Theme.successColor
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    
                    Text {
                        text: "技术规格"
                        font.pixelSize: Theme.fontSizeLarge
                        color: Theme.textPrimary
                        anchors.verticalCenter: parent.verticalCenter
                    }
                }
                
                // 规格网格
                Grid {
                    width: parent.width
                    columns: 2
                    rowSpacing: Theme.spacingMedium
                    columnSpacing: Theme.spacingMedium
                    
                    Repeater {
                        model: [
                            {label: "工作温度", value: "-20°C ~ +70°C"},
                            {label: "存储温度", value: "-40°C ~ +85°C"},
                            {label: "防护等级", value: "IP65"},
                            {label: "供电电压", value: "DC 12-36V"},
                            {label: "功耗", value: "<15W"},
                            {label: "显示屏", value: "10.1\" 1920×1200"},
                            {label: "处理器", value: "ARM Cortex-A53 Quad"},
                            {label: "内存", value: "4GB RAM / 32GB ROM"}
                        ]
                        
                        Column {
                            width: (parent.width - Theme.spacingMedium) / 2
                            spacing: Theme.spacingTiny
                            
                            Text {
                                text: modelData.label
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textTertiary
                            }
                            
                            Text {
                                text: modelData.value
                                font.pixelSize: Theme.fontSizeMedium
                                color: "#e2e8f0"
                            }
                        }
                    }
                }
            }
        }
        
        // 认证与标准卡片
        Rectangle {
            width: parent.width
            height: 241
            color: Theme.darkSurface
            border.color: Theme.darkBorder
            border.width: Theme.borderThin
            radius: Theme.radiusMedium
            
            Column {
                anchors.fill: parent
                anchors.margins: Theme.spacingLarge
                spacing: Theme.spacingMedium
                
                // 标题
                Row {
                    width: parent.width
                    spacing: Theme.spacingSmall
                    
                    Rectangle {
                        width: 4
                        height: 24
                        color: "#ad46ff"
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    
                    Text {
                        text: "认证与标准"
                        font.pixelSize: Theme.fontSizeLarge
                        color: Theme.textPrimary
                        anchors.verticalCenter: parent.verticalCenter
                    }
                }
                
                // 认证网格
                Grid {
                    width: parent.width
                    columns: 2
                    rowSpacing: Theme.spacingSmall
                    columnSpacing: Theme.spacingSmall
                    
                    Repeater {
                        model: [
                            {title: "GB/T 6067.1-2010", desc: "起重机械安全规程"},
                            {title: "GB/T 12602-2009", desc: "起重机力矩限制器"},
                            {title: "CE认证", desc: "欧盟安全认证"},
                            {title: "ISO 9001", desc: "质量管理体系"}
                        ]
                        
                        Rectangle {
                            width: (parent.width - Theme.spacingSmall) / 2
                            height: 68
                            color: Qt.rgba(49/255, 65/255, 88/255, 0.5)
                            radius: Theme.radiusSmall
                            
                            Column {
                                anchors.centerIn: parent
                                width: parent.width - Theme.spacingLarge
                                spacing: Theme.spacingTiny
                                
                                Text {
                                    text: modelData.title
                                    font.pixelSize: Theme.fontSizeMedium
                                    color: "#e2e8f0"
                                    horizontalAlignment: Text.AlignHCenter
                                    width: parent.width
                                }
                                
                                Text {
                                    text: modelData.desc
                                    font.pixelSize: Theme.fontSizeTiny
                                    color: Theme.textTertiary
                                    horizontalAlignment: Text.AlignHCenter
                                    width: parent.width
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // 技术支持卡片
        Rectangle {
            width: parent.width
            height: 249
            color: Theme.darkSurface
            border.color: Theme.darkBorder
            border.width: Theme.borderThin
            radius: Theme.radiusMedium
            
            Column {
                anchors.fill: parent
                anchors.margins: Theme.spacingLarge
                spacing: Theme.spacingMedium
                
                // 标题
                Row {
                    width: parent.width
                    spacing: Theme.spacingSmall
                    
                    Rectangle {
                        width: 4
                        height: 24
                        color: "#00b8db"
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    
                    Text {
                        text: "技术支持"
                        font.pixelSize: Theme.fontSizeLarge
                        color: Theme.textPrimary
                        anchors.verticalCenter: parent.verticalCenter
                    }
                }
                
                // 联系信息列表
                Column {
                    width: parent.width
                    spacing: Theme.spacingSmall
                    
                    // 服务热线
                    Row {
                        width: parent.width
                        spacing: Theme.spacingSmall
                        
                        Image {
                            width: Theme.iconSizeSmall
                            height: Theme.iconSizeSmall
                            source: "../../assets/images/icon-phone.svg"
                            fillMode: Image.PreserveAspectFit
                            anchors.verticalCenter: parent.verticalCenter
                        }
                        
                        Column {
                            width: parent.width - Theme.iconSizeSmall - Theme.spacingSmall
                            spacing: 0
                            
                            Text {
                                text: "服务热线"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textTertiary
                            }
                            
                            Text {
                                text: "400-888-6688"
                                font.pixelSize: Theme.fontSizeMedium
                                color: "#e2e8f0"
                            }
                        }
                    }
                    
                    // 技术邮箱
                    Row {
                        width: parent.width
                        spacing: Theme.spacingSmall
                        
                        Image {
                            width: Theme.iconSizeSmall
                            height: Theme.iconSizeSmall
                            source: "../../assets/images/icon-email.svg"
                            fillMode: Image.PreserveAspectFit
                            anchors.verticalCenter: parent.verticalCenter
                        }
                        
                        Column {
                            width: parent.width - Theme.iconSizeSmall - Theme.spacingSmall
                            spacing: 0
                            
                            Text {
                                text: "技术邮箱"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textTertiary
                            }
                            
                            Text {
                                text: "support@crane-monitor.com"
                                font.pixelSize: Theme.fontSizeMedium
                                color: "#e2e8f0"
                            }
                        }
                    }
                    
                    // 公司地址
                    Row {
                        width: parent.width
                        spacing: Theme.spacingSmall
                        
                        Image {
                            width: Theme.iconSizeSmall
                            height: Theme.iconSizeSmall
                            source: "../../assets/images/icon-location.svg"
                            fillMode: Image.PreserveAspectFit
                            anchors.verticalCenter: parent.verticalCenter
                        }
                        
                        Column {
                            width: parent.width - Theme.iconSizeSmall - Theme.spacingSmall
                            spacing: 0
                            
                            Text {
                                text: "公司地址"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textTertiary
                            }
                            
                            Text {
                                text: "北京市海淀区中关村科技园区"
                                font.pixelSize: Theme.fontSizeMedium
                                color: "#e2e8f0"
                            }
                        }
                    }
                }
            }
        }
        
        // 版权信息卡片
        Rectangle {
            width: parent.width
            height: 93
            color: Theme.darkSurface
            border.color: Theme.darkBorder
            border.width: Theme.borderThin
            radius: Theme.radiusMedium
            
            Column {
                anchors.centerIn: parent
                width: parent.width - Theme.spacingLarge * 2
                spacing: Theme.spacingSmall
                
                Text {
                    text: "© 2025 汽车吊力矩监测系统 版权所有"
                    font.pixelSize: Theme.fontSizeSmall
                    color: Theme.textTertiary
                    horizontalAlignment: Text.AlignHCenter
                    width: parent.width
                }
                
                Text {
                    text: "未经授权禁止复制、传播或使用本系统的任何内容"
                    font.pixelSize: Theme.fontSizeTiny
                    color: "#62748e"
                    horizontalAlignment: Text.AlignHCenter
                    width: parent.width
                }
            }
        }
    }
}
