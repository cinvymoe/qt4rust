import qt.rust.demo
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../../styles"

Item {
    id: languageView
    
    TranslationBridge { id: tr }
    
    Column {
        anchors.fill: parent
        anchors.margins: Theme.spacingLarge
        spacing: Theme.spacingLarge
        
        // Title
        Row {
            spacing: Theme.spacingMedium
            
            Rectangle {
                width: 4
                height: Theme.fontSizeLarge
                color: Theme.darkAccent
                anchors.verticalCenter: parent.verticalCenter
            }
            
            Text {
                text: tr.translate("settings.language")
                font.pixelSize: Theme.fontSizeLarge
                font.family: Theme.fontFamilyDefault
                font.weight: Font.Medium
                color: Theme.textPrimary
            }
        }
        
        // Language options
        Rectangle {
            width: parent.width
            height: 200
            color: Theme.darkSurface
            border.color: Theme.darkBorder
            border.width: Theme.borderThin
            radius: Theme.radiusMedium
            
            Column {
                anchors.fill: parent
                anchors.margins: Theme.spacingMedium
                spacing: Theme.spacingMedium
                
                // Chinese option
                Rectangle {
                    width: parent.width
                    height: 60
                    color: tr.currentLocale === "zh-CN" ? Theme.darkAccent : Theme.darkBackground
                    radius: Theme.radiusSmall
                    
                    Text {
                        text: tr.translate("settings.language.zhCN")
                        font.pixelSize: Theme.fontSizeMedium
                        color: Theme.textPrimary
                        font.family: Theme.fontFamilyDefault
                        anchors.centerIn: parent
                    }
                    
                    MouseArea {
                        anchors.fill: parent
                        onClicked: {
                            tr.setLocale("zh-CN")
                        }
                    }
                }
                
                // English option
                Rectangle {
                    width: parent.width
                    height: 60
                    color: tr.currentLocale === "en-US" ? Theme.darkAccent : Theme.darkBackground
                    radius: Theme.radiusSmall
                    
                    Text {
                        text: tr.translate("settings.language.enUS")
                        font.pixelSize: Theme.fontSizeMedium
                        color: Theme.textPrimary
                        font.family: Theme.fontFamilyDefault
                        anchors.centerIn: parent
                    }
                    
                    MouseArea {
                        anchors.fill: parent
                        onClicked: {
                            tr.setLocale("en-US")
                        }
                    }
                }
            }
        }
    }
}
