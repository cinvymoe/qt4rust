#!/usr/bin/env python3
"""
将 SVG 图标转换为 PNG 格式
处理 Figma 导出的带有 CSS 变量的 SVG 文件
"""

import os
import re
import subprocess
from pathlib import Path

# 图标目录
ICONS_DIR = Path("qml/assets/images")

# CSS 变量到实际颜色的映射（从设计系统提取）
COLOR_MAP = {
    "var(--stroke-0, #90A1B9)": "#90A1B9",  # textTertiary
    "var(--stroke-0, white)": "#ffffff",     # textPrimary
    "var(--stroke-0, #00D3F2)": "#51a2ff",   # textAccent (调整为设计系统颜色)
    "var(--stroke-0, #FB2C36)": "#fb2c36",   # dangerLight
    "var(--stroke-0, #FF8904)": "#f0b100",   # warningColor (调整)
    "var(--stroke-0, #C27AFF)": "#c27aff",   # 紫色（保持）
    "var(--stroke-0, #51A2FF)": "#51a2ff",   # textAccent
}

def replace_css_vars(svg_content):
    """替换 SVG 中的 CSS 变量为实际颜色值"""
    for css_var, color in COLOR_MAP.items():
        # 替换 stroke 属性中的变量
        svg_content = svg_content.replace(f'stroke="{css_var}"', f'stroke="{color}"')
        # 替换 fill 属性中的变量
        svg_content = svg_content.replace(f'fill="{css_var}"', f'fill="{color}"')
    return svg_content

def convert_svg_to_png(svg_path, size=48):
    """将 SVG 文件转换为 PNG"""
    print(f"处理: {svg_path.name}")
    
    # 先用 file 命令检查文件类型
    result = subprocess.run(['file', str(svg_path)], capture_output=True, text=True)
    if 'SVG' not in result.stdout:
        print(f"  跳过: 已经是 PNG 或其他格式")
        return False
    
    # 读取 SVG 内容
    try:
        with open(svg_path, 'r', encoding='utf-8') as f:
            svg_content = f.read()
    except UnicodeDecodeError:
        print(f"  跳过: 无法读取为文本文件")
        return False
    
    # 检查是否是 SVG 文件
    if not svg_content.strip().startswith('<svg') and not svg_content.strip().startswith('<?xml'):
        print(f"  跳过: 不是 SVG 文件")
        return False
    
    # 替换 CSS 变量
    svg_content = replace_css_vars(svg_content)
    
    # 创建临时 SVG 文件
    temp_svg = svg_path.with_suffix('.tmp.svg')
    with open(temp_svg, 'w', encoding='utf-8') as f:
        f.write(svg_content)
    
    # 使用 ImageMagick 转换
    temp_png = svg_path.with_suffix('.tmp.png')
    try:
        result = subprocess.run(
            ['convert', '-background', 'none', str(temp_svg), 
             '-resize', f'{size}x{size}', str(temp_png)],
            capture_output=True,
            text=True
        )
        
        if result.returncode == 0 and temp_png.exists():
            # 替换原文件
            temp_png.replace(svg_path)
            print(f"  ✓ 转换成功: {svg_path.name}")
            success = True
        else:
            print(f"  ✗ 转换失败: {result.stderr}")
            success = False
    except Exception as e:
        print(f"  ✗ 错误: {e}")
        success = False
    finally:
        # 清理临时文件
        if temp_svg.exists():
            temp_svg.unlink()
        if temp_png.exists() and not success:
            temp_png.unlink()
    
    return success

def main():
    """主函数"""
    print("=== SVG 转 PNG 转换工具 ===\n")
    
    # 查找所有需要转换的图标文件
    icon_files = []
    for pattern in ['icon-*.png', 'icon-logo.png']:
        icon_files.extend(ICONS_DIR.glob(pattern))
    
    if not icon_files:
        print("未找到需要转换的图标文件")
        return
    
    print(f"找到 {len(icon_files)} 个图标文件\n")
    
    success_count = 0
    for icon_file in sorted(icon_files):
        if convert_svg_to_png(icon_file):
            success_count += 1
    
    print(f"\n=== 转换完成 ===")
    print(f"成功: {success_count}/{len(icon_files)}")

if __name__ == "__main__":
    main()
