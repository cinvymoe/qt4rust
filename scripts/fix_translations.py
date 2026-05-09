#!/usr/bin/env python3
"""
批量修复 QML 文件中的翻译调用，使其支持响应式更新
"""
import re
import sys
from pathlib import Path

def fix_simple_translation(content):
    """
    修复简单的翻译调用: text: TranslationBridge.translate("key")
    添加 _localeVersion 依赖
    """
    # 匹配 Text { ... text: TranslationBridge.translate("key") ... }
    # 需要处理多行情况
    
    lines = content.split('\n')
    result_lines = []
    i = 0
    
    while i < len(lines):
        line = lines[i]
        
        # 检查是否是 Text { 行
        if re.match(r'^\s*Text\s*\{', line):
            # 找到对应的 Text 组件
            text_start = i
            brace_count = 0
            text_block = []
            
            # 收集整个 Text 组件
            j = i
            while j < len(lines):
                text_block.append(lines[j])
                brace_count += lines[j].count('{') - lines[j].count('}')
                if brace_count == 0 and j > i:
                    break
                j += 1
            
            text_component = '\n'.join(text_block)
            
            # 检查是否包含简单的 TranslationBridge.translate
            # 简单模式: text: TranslationBridge.translate("key")
            # 不包含字符串拼接、条件表达式等
            
            simple_pattern = r'text:\s*TranslationBridge\.translate\("([^"]+)"\)\s*$'
            
            if re.search(simple_pattern, text_component, re.MULTILINE):
                # 添加 _localeVersion 属性
                modified = add_locale_version_to_text(text_component)
                result_lines.extend(modified.split('\n'))
                i = j + 1
                continue
        
        result_lines.append(line)
        i += 1
    
    return '\n'.join(result_lines)

def add_locale_version_to_text(text_component):
    """
    在 Text 组件中添加 _localeVersion 属性
    """
    lines = text_component.split('\n')
    result = []
    added_property = False
    
    for i, line in enumerate(lines):
        result.append(line)
        
        # 在第一个属性之前添加 _localeVersion
        if not added_property and i == 0:
            # 在 Text { 之后添加属性
            result.append('            property int _localeVersion: TranslationBridge.locale_version')
            added_property = True
        
        # 修改 text 行
        match = re.match(r'^(\s*)text:\s*TranslationBridge\.translate\("([^"]+)"\)\s*$', line)
        if match:
            indent = match.group(1)
            key = match.group(2)
            result[-1] = f'{indent}text: {{'
            result.append(f'{indent}    const _ = _localeVersion')
            result.append(f'{indent}    return TranslationBridge.translate("{key}")')
            result.append(f'{indent}}}')
    
    return '\n'.join(result)

def process_file(file_path):
    """处理单个文件"""
    print(f"Processing: {file_path}")
    
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # 跳过 TrText.qml 本身
    if 'TrText.qml' in str(file_path):
        print(f"  Skipping (TrText.qml)")
        return
    
    # 跳过 TranslationManager.qml
    if 'TranslationManager.qml' in str(file_path):
        print(f"  Skipping (TranslationManager.qml)")
        return
    
    original = content
    modified = fix_simple_translation(content)
    
    if modified != original:
        # 备份原文件
        backup_path = file_path.with_suffix('.qml.bak')
        with open(backup_path, 'w', encoding='utf-8') as f:
            f.write(original)
        
        # 写入修改后的内容
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(modified)
        
        print(f"  ✓ Modified (backup: {backup_path.name})")
    else:
        print(f"  - No changes needed")

def main():
    qml_dir = Path('/mnt/sdb1/qt4rust/.worktrees/i18n/qml')
    
    # 找到所有包含 TranslationBridge.translate 的 QML 文件
    qml_files = []
    for qml_file in qml_dir.rglob('*.qml'):
        content = qml_file.read_text(encoding='utf-8')
        if 'TranslationBridge.translate' in content:
            qml_files.append(qml_file)
    
    print(f"Found {len(qml_files)} files with translations\n")
    
    for qml_file in sorted(qml_files):
        process_file(qml_file)
    
    print("\nDone!")

if __name__ == '__main__':
    main()
