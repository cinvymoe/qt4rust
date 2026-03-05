#!/usr/bin/env python3
"""QML 预览工具 - 使用 PySide6"""

import sys
from PySide6.QtGui import QGuiApplication
from PySide6.QtQml import QQmlApplicationEngine

def main():
    if len(sys.argv) < 2:
        print("用法: python preview_qml.py <qml文件路径>")
        sys.exit(1)
    
    app = QGuiApplication(sys.argv)
    engine = QQmlApplicationEngine()
    
    qml_file = sys.argv[1]
    engine.load(qml_file)
    
    if not engine.rootObjects():
        print(f"错误: 无法加载 QML 文件: {qml_file}")
        sys.exit(-1)
    
    sys.exit(app.exec())

if __name__ == "__main__":
    main()
