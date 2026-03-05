#!/bin/sh
cd /data/local/tmp/qt-rust-demo
export LD_LIBRARY_PATH=/data/local/tmp/qt-rust-demo/lib:$LD_LIBRARY_PATH
export QT_PLUGIN_PATH=/data/local/tmp/qt-rust-demo/plugins
export QT_QPA_PLATFORM_PLUGIN_PATH=/data/local/tmp/qt-rust-demo/plugins
export QT_QPA_PLATFORM=linuxfb:fb=/dev/fb0
export QML2_IMPORT_PATH=/data/local/tmp/qt-rust-demo/qml_modules
export QT_QPA_FB_DISABLE_INPUT=0
export QT_IM_MODULE=qtvirtualkeyboard
export QT_VIRTUALKEYBOARD_DESKTOP_DISABLE=0
export QT_DEBUG_PLUGINS=1
export QT_LOGGING_RULES="qt.virtualkeyboard=true"
./qt-rust-demo
