#!/bin/bash
# 收集 ARM32 设备所需字体
# 策略：轻量拉丁字体 + 中文回退字体，总计约 6MB

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

FONTS_DIR="fonts"
mkdir -p "$FONTS_DIR"

echo -e "${YELLOW}收集字体文件...${NC}"

# 需要收集的字体列表：路径|目标文件名
FONTS=(
    # DejaVu Sans - 拉丁/数字/符号，嵌入式设备首选
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf|DejaVuSans.ttf"
    "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf|DejaVuSans-Bold.ttf"
    "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf|DejaVuSansMono.ttf"
    # Noto Sans - 备用拉丁字体
    "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf|NotoSans-Regular.ttf"
    "/usr/share/fonts/truetype/noto/NotoSans-Bold.ttf|NotoSans-Bold.ttf"
    # DroidSansFallbackFull - 中文/日文/韩文回退（3.9MB，覆盖完整 CJK）
    "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf|DroidSansFallbackFull.ttf"
)

COPIED=0
MISSING=0

for entry in "${FONTS[@]}"; do
    src="${entry%%|*}"
    dst="${entry##*|}"
    if [ -f "$src" ]; then
        cp "$src" "$FONTS_DIR/$dst"
        size=$(du -sh "$FONTS_DIR/$dst" | cut -f1)
        echo -e "  ${GREEN}✓${NC} $dst ($size)"
        COPIED=$((COPIED + 1))
    else
        echo -e "  ${YELLOW}⚠${NC} 未找到: $src"
        MISSING=$((MISSING + 1))
    fi
done

# 生成 fonts.conf
cat > "$FONTS_DIR/fonts.conf" << 'CONF'
<?xml version="1.0"?>
<!DOCTYPE fontconfig SYSTEM "fonts.dtd">
<fontconfig>
    <!-- 应用自带字体目录（优先） -->
    <dir>/data/local/tmp/qt-rust-demo/fonts</dir>
    <!-- 系统字体目录（回退） -->
    <dir>/system/fonts</dir>
    <dir>/usr/share/fonts</dir>

    <!-- fontconfig 缓存目录 -->
    <cachedir>/tmp/fontconfig-cache</cachedir>

    <!-- sans-serif 字体映射 -->
    <match target="pattern">
        <test qual="any" name="family">
            <string>sans-serif</string>
        </test>
        <edit name="family" mode="prepend" binding="strong">
            <string>DejaVu Sans</string>
            <string>Noto Sans</string>
            <string>Droid Sans Fallback</string>
        </edit>
    </match>

    <!-- serif 字体映射 -->
    <match target="pattern">
        <test qual="any" name="family">
            <string>serif</string>
        </test>
        <edit name="family" mode="prepend" binding="strong">
            <string>DejaVu Serif</string>
            <string>Droid Sans Fallback</string>
        </edit>
    </match>

    <!-- monospace 字体映射 -->
    <match target="pattern">
        <test qual="any" name="family">
            <string>monospace</string>
        </test>
        <edit name="family" mode="prepend" binding="strong">
            <string>DejaVu Sans Mono</string>
        </edit>
    </match>

    <!-- 中文字体映射：优先使用 DroidSansFallback -->
    <match target="pattern">
        <test name="lang" compare="contains">
            <string>zh</string>
        </test>
        <edit name="family" mode="prepend" binding="strong">
            <string>Droid Sans Fallback</string>
        </edit>
    </match>

    <!-- 全局回退：任何未匹配字符使用 DroidSansFallback -->
    <match target="pattern">
        <edit name="family" mode="append" binding="weak">
            <string>Droid Sans Fallback</string>
        </edit>
    </match>
</fontconfig>
CONF

echo -e "${GREEN}✓ fonts.conf 已生成${NC}"

TOTAL_SIZE=$(du -sh "$FONTS_DIR" | cut -f1)
echo ""
echo -e "${GREEN}字体收集完成: $COPIED 个文件，$MISSING 个缺失，总计 $TOTAL_SIZE${NC}"
