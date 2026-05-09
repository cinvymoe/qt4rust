#!/usr/bin/env python3
"""
处理剩余的翻译调用
"""
import re
from pathlib import Path

def fix_header(file_path: Path) -> bool:
    """修复 Header.qml"""
    print(f"Processing: {file_path}")
    content = file_path.read_text(encoding='utf-8')
    original = content
    
    # 在 getAlertText 函数开始处添加 locale_version
    content = re.sub(
        r'(function getAlertText\(\) \{\s*)(if \(isDanger)',
        r'\1const _ = TranslationBridge.locale_version\n        \2',
        content
    )
    
    if content != original:
        file_path.write_text(content, encoding='utf-8')
        print(f"  ✓ Modified")
        return True
    else:
        print(f"  - No changes")
        return False

def fix_moment_card(file_path: Path) -> bool:
    """修复 MomentCard.qml"""
    print(f"Processing: {file_path}")
    content = file_path.read_text(encoding='utf-8')
    original = content
    
    # 在三元表达式前添加 locale_version
    lines = content.split('\n')
    result = []
    
    for i, line in enumerate(lines):
        if 'text: momentCard.percentage >= momentCard.dangerThreshold ? TranslationBridge.translate' in line:
            # 在这一行前面插入 locale_version
            indent = len(line) - len(line.lstrip())
            result.append(' ' * indent + 'const _ = TranslationBridge.locale_version')
            result.append(line)
            print(f"  Fixed line {i+1}: ternary expression")
        else:
            result.append(line)
    
    new_content = '\n'.join(result)
    
    if new_content != original:
        file_path.write_text(new_content, encoding='utf-8')
        print(f"  ✓ Modified")
        return True
    else:
        print(f"  - No changes")
        return False

def fix_calibration_view(file_path: Path) -> bool:
    """修复 CalibrationView.qml"""
    print(f"Processing: {file_path}")
    content = file_path.read_text(encoding='utf-8')
    original = content
    
    # 替换 model 数组为函数调用
    old_model = '''                            model: [
                                {text: TranslationBridge.translate("calibration.loadSensor")},
                                {text: TranslationBridge.translate("calibration.angleSensor")},
                                {text: TranslationBridge.translate("calibration.radiusSensor")},
                                {text: TranslationBridge.translate("calibration.alarmThreshold")}
                            ]'''
    
    new_model = '''                            model: getCalibrationTabModel()'''
    
    content = content.replace(old_model, new_model)
    
    # 添加函数
    if 'function getCalibrationTabModel()' not in content:
        # 在文件末尾添加函数
        content = content.rstrip()
        content += '''

    // 获取标定 Tab 模型
    function getCalibrationTabModel() {
        const _ = TranslationBridge.locale_version
        return [
            {text: TranslationBridge.translate("calibration.loadSensor")},
            {text: TranslationBridge.translate("calibration.angleSensor")},
            {text: TranslationBridge.translate("calibration.radiusSensor")},
            {text: TranslationBridge.translate("calibration.alarmThreshold")}
        ]
    }
'''
    
    if content != original:
        file_path.write_text(content, encoding='utf-8')
        print(f"  ✓ Modified")
        return True
    else:
        print(f"  - No changes")
        return False

def fix_simple_text(file_path: Path) -> bool:
    """修复简单的 text 属性"""
    print(f"Processing: {file_path}")
    content = file_path.read_text(encoding='utf-8')
    original = content
    
    lines = content.split('\n')
    result = []
    
    for i, line in enumerate(lines):
        # 检查是否是简单的 text: TranslationBridge.translate(...) 模式
        if re.match(r'^\s*text:\s*TranslationBridge\.translate\("([^"]+)"\)(\s*\|\|\s*"[^"]*")?\s*$', line):
            match = re.match(r'^(\s*)text:\s*TranslationBridge\.translate\("([^"]+)"\)(\s*\|\|\s*"[^"]*")?\s*$', line)
            if match and '{ const _' not in line:
                indent = match.group(1)
                key = match.group(2)
                default_value = match.group(3) or ''
                new_line = f'{indent}text: {{ const _ = TranslationBridge.locale_version; return TranslationBridge.translate("{key}"){default_value} }}'
                result.append(new_line)
                print(f"  Fixed line {i+1}: simple text")
            else:
                result.append(line)
        else:
            result.append(line)
    
    new_content = '\n'.join(result)
    
    if new_content != original:
        file_path.write_text(new_content, encoding='utf-8')
        print(f"  ✓ Modified")
        return True
    else:
        print(f"  - No changes")
        return False

def main():
    qml_dir = Path('/mnt/sdb1/qt4rust/.worktrees/i18n/qml')
    
    # 特殊处理
    fix_header(qml_dir / 'components/layouts/Header.qml')
    fix_moment_card(qml_dir / 'components/controls/MomentCard.qml')
    fix_calibration_view(qml_dir / 'views/SettingsView/CalibrationView.qml')
    
    # 处理其他简单文件
    simple_files = [
        'views/SettingsView/CalibrationContents/AngleCalibrationContent.qml',
        'views/SettingsView/CalibrationContents/LoadCalibrationContent.qml',
        'views/SettingsView/CalibrationContents/RadiusCalibrationContent.qml',
        'views/SettingsView/MomentCurveView.qml',
    ]
    
    for rel_path in simple_files:
        fix_simple_text(qml_dir / rel_path)
    
    print("\nDone!")

if __name__ == '__main__':
    main()
