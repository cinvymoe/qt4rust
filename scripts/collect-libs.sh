#!/bin/bash
# 自动收集应用所需的所有共享库

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# 优先使用 release 版本，如果不存在则使用 debug 版本
BINARY_RELEASE="target/armv7-unknown-linux-gnueabihf/release/qt-rust-demo"
BINARY_DEBUG="target/armv7-unknown-linux-gnueabihf/debug/qt-rust-demo"

if [ -f "$BINARY_RELEASE" ]; then
    BINARY="$BINARY_RELEASE"
    echo -e "${GREEN}使用 release 版本${NC}"
elif [ -f "$BINARY_DEBUG" ]; then
    BINARY="$BINARY_DEBUG"
    echo -e "${YELLOW}使用 debug 版本${NC}"
else
    echo -e "${RED}错误: 未找到二进制文件${NC}"
    echo "请先运行: cargo build --release --target armv7-unknown-linux-gnueabihf"
    exit 1
fi

OUTPUT_DIR="libs-to-deploy"

echo -e "${GREEN}=== 收集共享库依赖 ===${NC}"

# 创建输出目录
mkdir -p "$OUTPUT_DIR"

echo -e "${YELLOW}分析依赖...${NC}"
LIBS=$(arm-linux-gnueabihf-readelf -d "$BINARY" | grep "NEEDED" | sed 's/.*\[\(.*\)\]/\1/')

echo "找到以下依赖库："
echo "$LIBS"
echo ""

echo -e "${YELLOW}复制库文件...${NC}"
LIB_BASE="/usr/lib/arm-linux-gnueabihf"

for lib in $LIBS; do
    # 跳过系统基础库（通常设备已有）
    if [[ "$lib" == "libc.so.6" ]] || \
       [[ "$lib" == "libm.so.6" ]] || \
       [[ "$lib" == "ld-linux-armhf.so.3" ]]; then
        echo -e "  ${YELLOW}跳过系统库: $lib${NC}"
        continue
    fi
    
    LIB_PATH="$LIB_BASE/$lib"
    if [ -f "$LIB_PATH" ]; then
        echo -e "  ${GREEN}✓${NC} 复制 $lib"
        cp -L "$LIB_PATH" "$OUTPUT_DIR/"
        
        # 同时复制实际的 .so 文件（如果是符号链接）
        if [ -L "$LIB_PATH" ]; then
            REAL_LIB=$(readlink -f "$LIB_PATH")
            if [ -f "$REAL_LIB" ]; then
                cp "$REAL_LIB" "$OUTPUT_DIR/"
            fi
        fi
    else
        echo -e "  ${RED}✗${NC} 未找到 $lib"
    fi
done

# 添加 Qt 插件依赖
echo ""
echo -e "${YELLOW}添加 Qt 平台插件依赖...${NC}"
QT_PLUGIN_DIR="/usr/lib/arm-linux-gnueabihf/qt6/plugins"
if [ -f "$QT_PLUGIN_DIR/platforms/libqlinuxfb.so" ]; then
    PLUGIN_DEPS=$(arm-linux-gnueabihf-readelf -d "$QT_PLUGIN_DIR/platforms/libqlinuxfb.so" 2>/dev/null | grep "NEEDED" | sed 's/.*\[\(.*\)\]/\1/' | grep -v "^lib[cm].so" | grep -v "ld-linux" || true)
    
    for dep in $PLUGIN_DEPS; do
        if [ -z "$dep" ]; then
            continue
        fi
        
        DEP_PATH="$LIB_BASE/$dep"
        if [ -f "$DEP_PATH" ] && [ ! -f "$OUTPUT_DIR/$dep" ]; then
            echo -e "  ${GREEN}+${NC} 添加插件依赖 $dep"
            cp -L "$DEP_PATH" "$OUTPUT_DIR/" 2>/dev/null || true
            
            if [ -L "$DEP_PATH" ]; then
                REAL_DEP=$(readlink -f "$DEP_PATH")
                if [ -f "$REAL_DEP" ] && [ ! -f "$OUTPUT_DIR/$(basename $REAL_DEP)" ]; then
                    cp "$REAL_DEP" "$OUTPUT_DIR/" 2>/dev/null || true
                fi
            fi
        fi
    done
fi

# 添加 QML 插件依赖
echo ""
echo -e "${YELLOW}添加 QML 插件依赖...${NC}"
QT_QML_DIR="/usr/lib/arm-linux-gnueabihf/qt6/qml"
if [ -d "$QT_QML_DIR" ]; then
    # 查找所有 QML 插件 .so 文件
    find "$QT_QML_DIR" -name "*.so" -type f 2>/dev/null | while read qml_plugin; do
        QML_DEPS=$(arm-linux-gnueabihf-readelf -d "$qml_plugin" 2>/dev/null | grep "NEEDED" | sed 's/.*\[\(.*\)\]/\1/' | grep -v "^lib[cm].so" | grep -v "ld-linux" || true)
        
        for dep in $QML_DEPS; do
            if [ -z "$dep" ]; then
                continue
            fi
            
            DEP_PATH="$LIB_BASE/$dep"
            if [ -f "$DEP_PATH" ] && [ ! -f "$OUTPUT_DIR/$dep" ]; then
                echo -e "  ${GREEN}+${NC} 添加 QML 插件依赖 $dep (来自 $(basename $qml_plugin))"
                cp -L "$DEP_PATH" "$OUTPUT_DIR/" 2>/dev/null || true
                
                if [ -L "$DEP_PATH" ]; then
                    REAL_DEP=$(readlink -f "$DEP_PATH")
                    if [ -f "$REAL_DEP" ] && [ ! -f "$OUTPUT_DIR/$(basename $REAL_DEP)" ]; then
                        cp "$REAL_DEP" "$OUTPUT_DIR/" 2>/dev/null || true
                    fi
                fi
            fi
        done
    done
fi

# 递归检查 Qt 库的依赖
echo ""
echo -e "${YELLOW}检查库的递归依赖...${NC}"

# 重复检查几次以确保获取所有递归依赖
for iteration in {1..5}; do
    FOUND_NEW=0
    for libfile in "$OUTPUT_DIR"/*.so* "$OUTPUT_DIR"/*.so; do
        if [ -f "$libfile" ]; then
            QT_DEPS=$(arm-linux-gnueabihf-readelf -d "$libfile" 2>/dev/null | grep "NEEDED" | sed 's/.*\[\(.*\)\]/\1/' | grep -v "^lib[cm].so" | grep -v "ld-linux" || true)
            
            for dep in $QT_DEPS; do
                if [ -z "$dep" ]; then
                    continue
                fi
                
                DEP_PATH="$LIB_BASE/$dep"
                if [ -f "$DEP_PATH" ] && [ ! -f "$OUTPUT_DIR/$dep" ]; then
                    echo -e "  ${GREEN}+${NC} 添加依赖 $dep (来自 $(basename $libfile))"
                    cp -L "$DEP_PATH" "$OUTPUT_DIR/" 2>/dev/null || true
                    FOUND_NEW=1
                    
                    # 复制实际文件
                    if [ -L "$DEP_PATH" ]; then
                        REAL_DEP=$(readlink -f "$DEP_PATH")
                        if [ -f "$REAL_DEP" ] && [ ! -f "$OUTPUT_DIR/$(basename $REAL_DEP)" ]; then
                            cp "$REAL_DEP" "$OUTPUT_DIR/" 2>/dev/null || true
                        fi
                    fi
                fi
            done
        fi
    done
    
    if [ $FOUND_NEW -eq 0 ]; then
        break
    fi
done

echo ""
echo -e "${GREEN}完成！${NC}"
echo "库文件已复制到: $OUTPUT_DIR/"
echo ""
echo "文件列表:"
ls -lh "$OUTPUT_DIR/"
echo ""
echo "总大小:"
du -sh "$OUTPUT_DIR/"
