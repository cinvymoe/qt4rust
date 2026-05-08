import QtQuick
import qt.rust.demo

Item {
    id: root

    property TranslationBridge _bridge: TranslationBridge {}

    function t(key) {
        return root._bridge.translate(key)
    }

    function tWithArgs(key, args) {
        return root._bridge.translateWithArgs(key, JSON.stringify(args))
    }

    function availableLocales() {
        return root._bridge.availableLocales()
    }

    function setLocale(locale) {
        return root._bridge.setLocale(locale)
    }

    function getLocale() {
        return root._bridge.currentLocale
    }
}
