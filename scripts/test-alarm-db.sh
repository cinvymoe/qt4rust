#!/bin/bash
# 测试 ADB 数据库中的报警信息内容
# 用法: ./test-alarm-db.sh [选项]

set -e

# 颜色定义
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# 默认参数
SHOW_ALL=false
SHOW_ACKNOWLEDGED=false
SHOW_UNACKNOWLEDGED=true
LIMIT=20
DB_PATH="db/crane_data.db"
ALARM_TYPE=""
VERBOSE=false
JSON_OUTPUT=false

# 帮助信息
show_help() {
	echo "用法: $0 [选项]"
	echo ""
	echo "测试 ADB 数据库中的报警信息内容"
	echo ""
	echo "选项:"
	echo "  --all              显示所有报警（包括已确认和未确认）"
	echo "  --acknowledged     仅显示已确认的报警"
	echo "  --unacknowledged   仅显示未确认的报警（默认）"
	echo "  --type TYPE        按报警类型过滤 (warning/danger)"
	echo "  --limit N          限制显示最近 N 条记录（默认: 20）"
	echo "  --verbose          显示详细信息"
	echo "  --json             以 JSON 格式输出"
	echo "  --help, -h         显示帮助信息"
	echo ""
	echo "示例:"
	echo "  $0                              # 测试最近 20 条未确认报警"
	echo "  $0 --all --limit 50             # 测试所有报警，限制 50 条"
	echo "  $0 --type danger                # 仅测试危险报警"
	echo "  $0 --acknowledged               # 测试已确认的报警"
	echo "  $0 --verbose                    # 详细模式"
	echo "  $0 --json                       # JSON 格式输出"
	exit 0
}

# 解析参数
while [[ $# -gt 0 ]]; do
	case $1 in
	--all)
		SHOW_ALL=true
		SHOW_ACKNOWLEDGED=true
		SHOW_UNACKNOWLEDGED=true
		shift
		;;
	--acknowledged)
		SHOW_ACKNOWLEDGED=true
		SHOW_UNACKNOWLEDGED=false
		shift
		;;
	--unacknowledged)
		SHOW_UNACKNOWLEDGED=true
		SHOW_ACKNOWLEDGED=false
		shift
		;;
	--type)
		ALARM_TYPE="$2"
		shift 2
		;;
	--limit)
		LIMIT="$2"
		shift 2
		;;
	--verbose)
		VERBOSE=true
		shift
		;;
	--json)
		JSON_OUTPUT=true
		shift
		;;
	--help | -h)
		show_help
		;;
	*)
		echo "未知选项: $1"
		echo "使用 --help 查看帮助"
		exit 1
		;;
	esac
done

echo -e "${CYAN}=========================================="
echo "   ADB 数据库报警信息测试工具"
echo -e "==========================================${NC}"
echo ""

# 检查 sqlite3
if ! command -v sqlite3 &>/dev/null; then
	echo -e "${RED}错误: 未安装 sqlite3 命令行工具${NC}"
	echo "请安装 sqlite3: sudo apt-get install sqlite3"
	exit 1
fi

# 检查 ADB 设备
echo -e "${BLUE}[1/4] 检查 ADB 设备连接...${NC}"
if ! adb devices | grep -q "device$"; then
	echo -e "${RED}✗ 未检测到 ADB 设备${NC}"
	echo "请检查设备连接: adb devices"
	exit 1
fi
DEVICE_INFO=$(adb devices | grep "device$" | head -1)
echo -e "${GREEN}✓ 设备已连接: $DEVICE_INFO${NC}"
echo ""

# 拉取数据库
echo -e "${BLUE}[2/4] 从设备拉取数据库...${NC}"
mkdir -p db
if ! adb pull /data/local/tmp/qt-rust-demo/crane_data.db ./db/crane_data.db 2>/dev/null; then
	echo -e "${YELLOW}⚠ 无法从设备拉取数据库，尝试使用本地数据库${NC}"
	if [ ! -f "$DB_PATH" ]; then
		echo -e "${RED}✗ 本地数据库文件不存在: $DB_PATH${NC}"
		exit 1
	fi
fi
echo -e "${GREEN}✓ 数据库文件: $DB_PATH${NC}"

DB_SIZE=$(ls -lh "$DB_PATH" | awk '{print $5}')
echo "  文件大小: $DB_SIZE"
echo ""

# 检查数据库结构
echo -e "${BLUE}[3/4] 检查数据库结构...${NC}"

# 检查 alarm_records 表是否存在
TABLE_EXISTS=$(sqlite3 "$DB_PATH" "SELECT name FROM sqlite_master WHERE type='table' AND name='alarm_records';" 2>/dev/null || true)
if [ -z "$TABLE_EXISTS" ]; then
	echo -e "${RED}✗ 数据库中不存在 alarm_records 表${NC}"
	echo ""
	echo "现有表:"
	sqlite3 "$DB_PATH" ".tables"
	exit 1
fi
echo -e "${GREEN}✓ alarm_records 表存在${NC}"

if [ "$VERBOSE" = true ]; then
	echo ""
	echo "表结构:"
	sqlite3 "$DB_PATH" "PRAGMA table_info(alarm_records);" | while read line; do
		echo "  $line"
	done
fi

# 获取列信息
COLUMN_COUNT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM pragma_table_info('alarm_records');" 2>/dev/null || echo "0")
echo "  列数: $COLUMN_COUNT"
echo ""

# 查询报警数据
echo -e "${BLUE}[4/4] 查询报警信息...${NC}"
echo ""

# 构建查询条件
WHERE_CLAUSES=()
if [ "$SHOW_ALL" = false ]; then
	if [ "$SHOW_ACKNOWLEDGED" = true ] && [ "$SHOW_UNACKNOWLEDGED" = false ]; then
		WHERE_CLAUSES+=("acknowledged = 1")
		echo -e "${YELLOW}查询条件: 已确认报警${NC}"
	elif [ "$SHOW_UNACKNOWLEDGED" = true ] && [ "$SHOW_ACKNOWLEDGED" = false ]; then
		WHERE_CLAUSES+=("acknowledged = 0")
		echo -e "${YELLOW}查询条件: 未确认报警${NC}"
	fi
else
	echo -e "${YELLOW}查询条件: 所有报警${NC}"
fi

if [ -n "$ALARM_TYPE" ]; then
	WHERE_CLAUSES+=("alarm_type = '$ALARM_TYPE'")
	echo -e "${YELLOW}报警类型: $ALARM_TYPE${NC}"
fi

WHERE_SQL=""
if [ ${#WHERE_CLAUSES[@]} -gt 0 ]; then
	WHERE_SQL="WHERE "
	for i in "${!WHERE_CLAUSES[@]}"; do
		if [ $i -gt 0 ]; then
			WHERE_SQL+=" AND "
		fi
		WHERE_SQL+="${WHERE_CLAUSES[$i]}"
	done
fi

echo -e "${YELLOW}显示数量: 最近 $LIMIT 条${NC}"
echo ""

# 统计信息
TOTAL_ALARMS=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM alarm_records;" 2>/dev/null || echo "0")
UNACK_ALARMS=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM alarm_records WHERE acknowledged = 0;" 2>/dev/null || echo "0")
ACK_ALARMS=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM alarm_records WHERE acknowledged = 1;" 2>/dev/null || echo "0")
WARNING_ALARMS=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM alarm_records WHERE alarm_type = 'warning';" 2>/dev/null || echo "0")
DANGER_ALARMS=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM alarm_records WHERE alarm_type = 'danger';" 2>/dev/null || echo "0")

# JSON 输出
if [ "$JSON_OUTPUT" = true ]; then
	echo "{"
	echo "  \"statistics\": {"
	echo "    \"total\": $TOTAL_ALARMS,"
	echo "    \"unacknowledged\": $UNACK_ALARMS,"
	echo "    \"acknowledged\": $ACK_ALARMS,"
	echo "    \"warning\": $WARNING_ALARMS,"
	echo "    \"danger\": $DANGER_ALARMS"
	echo "  },"
	echo "  \"records\": ["

	QUERY="SELECT json_object(
        'id', id,
        'sequence_number', sequence_number,
        'timestamp', timestamp,
        'alarm_type', alarm_type,
        'current_load', current_load,
        'rated_load', rated_load,
        'working_radius', working_radius,
        'boom_angle', boom_angle,
        'boom_length', boom_length,
        'moment_percentage', moment_percentage,
        'description', description,
        'acknowledged', acknowledged,
        'acknowledged_at', acknowledged_at
    ) FROM alarm_records $WHERE_SQL ORDER BY timestamp DESC LIMIT $LIMIT;"

	sqlite3 "$DB_PATH" "$QUERY" | head -n -1 | sed 's/$/,/'
	sqlite3 "$DB_PATH" "$QUERY" | tail -1

	echo "  ]"
	echo "}"
	exit 0
fi

# 常规输出
echo -e "${CYAN}=========================================="
echo "   报警统计"
echo -e "==========================================${NC}"
printf "总报警数:       %b%s%b\n" "$GREEN" "$TOTAL_ALARMS" "$NC"
printf "未确认报警:     %b%s%b\n" "$RED" "$UNACK_ALARMS" "$NC"
printf "已确认报警:     %b%s%b\n" "$GREEN" "$ACK_ALARMS" "$NC"
printf "预警(warning):  %b%s%b\n" "$YELLOW" "$WARNING_ALARMS" "$NC"
printf "危险(danger):   %b%s%b\n" "$RED" "$DANGER_ALARMS" "$NC"
echo ""

# 详细记录
QUERY="SELECT 
    id,
    datetime(timestamp, 'unixepoch', 'localtime') as alarm_time,
    alarm_type,
    printf('%.1f', current_load) as load_ton,
    printf('%.1f', rated_load) as rated_ton,
    printf('%.1f', working_radius) as radius_m,
    printf('%.1f', boom_angle) as angle_deg,
    printf('%.1f', moment_percentage) as moment_pct,
    CASE WHEN acknowledged = 1 THEN '已确认' ELSE '未确认' END as status,
    description
FROM alarm_records 
$WHERE_SQL
ORDER BY timestamp DESC 
LIMIT $LIMIT;"

RESULT=$(sqlite3 -header -column "$DB_PATH" "$QUERY" 2>/dev/null || true)

if [ -z "$RESULT" ]; then
	if [ "$SHOW_UNACKNOWLEDGED" = true ] && [ "$SHOW_ACKNOWLEDGED" = false ]; then
		echo -e "${GREEN}✓ 没有未确认的报警记录${NC}"
		echo ""
		echo "提示: 使用 --all 参数查看所有报警记录"
	else
		echo -e "${YELLOW}数据库中没有符合条件的报警记录${NC}"
	fi
else
	echo -e "${CYAN}=========================================="
	echo "   详细报警记录"
	echo -e "==========================================${NC}"
	echo ""
	echo "$RESULT"
	echo ""
fi

# 详细信息模式
if [ "$VERBOSE" = true ] && [ "$TOTAL_ALARMS" -gt 0 ]; then
	echo -e "${CYAN}=========================================="
	echo "   报警内容详情"
	echo -e "==========================================${NC}"
	echo ""

	DETAIL_QUERY="SELECT 
        'ID: ' || id,
        '时间: ' || datetime(timestamp, 'unixepoch', 'localtime'),
        '类型: ' || alarm_type,
        '载荷: ' || printf('%.1f', current_load) || 't / ' || printf('%.1f', rated_load) || 't',
        '半径: ' || printf('%.1f', working_radius) || 'm',
        '角度: ' || printf('%.1f', boom_angle) || '°',
        '力矩: ' || printf('%.1f', moment_percentage) || '%',
        '状态: ' || CASE WHEN acknowledged = 1 THEN '已确认' ELSE '未确认' END,
        '描述: ' || COALESCE(description, '无')
    FROM alarm_records 
    $WHERE_SQL
    ORDER BY timestamp DESC 
    LIMIT $LIMIT;"

	sqlite3 "$DB_PATH" "$DETAIL_QUERY" | while IFS='|' read -r id time type load radius angle moment status desc; do
		echo -e "${BLUE}────────────────────────────────────────${NC}"
		echo -e "  $id"
		echo -e "  $time"
		echo -e "  $type"
		echo -e "  $load"
		echo -e "  $radius"
		echo -e "  $angle"
		echo -e "  $moment"
		echo -e "  $status"
		echo -e "  $desc"
	done
	echo -e "${BLUE}────────────────────────────────────────${NC}"
fi

# 警告提示
if [ "$SHOW_UNACKNOWLEDGED" = true ] && [ "$DANGER_ALARMS" -gt 0 ]; then
	UNACK_DANGER=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM alarm_records WHERE alarm_type = 'danger' AND acknowledged = 0;" 2>/dev/null || echo "0")
	if [ "$UNACK_DANGER" -gt 0 ]; then
		echo ""
		echo -e "${RED}⚠ 警告: 存在 $UNACK_DANGER 条未确认的危险报警！${NC}"
	fi
fi

# 测试结果
echo ""
echo -e "${CYAN}=========================================="
echo "   测试完成"
echo -e "==========================================${NC}"
echo "数据库位置: $DB_PATH"
echo "当前时间: $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

# 测试总结
if [ "$TOTAL_ALARMS" -eq 0 ]; then
	echo -e "${GREEN}✓ 测试通过: 数据库中没有报警记录${NC}"
elif [ "$UNACK_ALARMS" -eq 0 ]; then
	echo -e "${GREEN}✓ 测试通过: 所有报警已确认${NC}"
elif [ "$UNACK_DANGER" -gt 0 ] 2>/dev/null; then
	echo -e "${RED}✗ 测试警告: 存在未确认的危险报警${NC}"
else
	echo -e "${YELLOW}⚠ 测试提示: 存在 $UNACK_ALARMS 条未确认报警${NC}"
fi
