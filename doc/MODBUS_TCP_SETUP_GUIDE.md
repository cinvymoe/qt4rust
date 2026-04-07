# Modbus TCP 配置指南

本文档说明如何从模拟传感器切换到 Modbus TCP 真实传感器。

## 目录

1. [配置概览](#配置概览)
2. [Modbus Slave 配置](#modbus-slave-配置)
3. [Modbus Master 配置](#modbus-master-配置)
4. [启用 Modbus TCP](#启用-modbus-tcp)
5. [测试验证](#测试验证)
6. [故障排查](#故障排查)

---

## 配置概览

系统支持两种传感器模式：

- **模拟模式** (`use_simulator: true`): 使用软件生成的模拟数据
- **Modbus TCP 模式** (`use_simulator: false`): 从真实 Modbus 设备读取数据

切换流程：
```
1. 配置 Modbus Slave 设备（硬件端）
2. 配置 Modbus Master 参数（软件端）
3. 修改 pipeline_config.toml 启用真实传感器
4. 重启应用程序
```

---

## Modbus Slave 配置

### 1. Modbus Slave 设备要求

你的 Modbus Slave 设备（传感器）需要配置以下参数：

| 参数 | 说明 | 推荐值 |
|------|------|--------|
| IP 地址 | 设备网络地址 | `192.168.1.100` |
| 端口 | Modbus TCP 端口 | `502`（标准端口）|
| Slave ID | 从站地址 | `1`（1-247）|
| 寄存器地址 | 数据存储位置 | 见下表 |
| 数据类型 | 寄存器数据格式 | `Float32`（推荐）|
| 字节序 | 浮点数字节顺序 | `BigEndian`（推荐）|

### 2. 寄存器地址分配

系统需要读取 3 个传感器的数据，推荐的寄存器地址分配：

| 传感器 | 寄存器地址 | 寄存器数量 | 数据类型 | 说明 |
|--------|-----------|-----------|---------|------|
| AD1 (载荷) | `0` | `2` | Float32 | 载荷传感器（吨）|
| AD2 (半径) | `2` | `2` | Float32 | 半径传感器（米）|
| AD3 (角度) | `4` | `2` | Float32 | 角度传感器（度）|

**注意**:
- Float32 类型占用 2 个寄存器（32位 = 2 × 16位）
- 寄存器地址连续分配，避免冲突
- 如果使用 UInt16/Int16，每个传感器只需 1 个寄存器

### 3. Modbus Slave 设备配置示例

以 **Modbus Slave 模拟器**为例（用于测试）：

```
设备 IP: 192.168.1.100
端口: 502
Slave ID: 1

寄存器配置:
- 地址 0-1: Float32 = 15.5 (载荷，吨)
- 地址 2-3: Float32 = 8.0  (半径，米)
- 地址 4-5: Float32 = 60.0 (角度，度)
```

**推荐工具**:
- [Modbus Slave](https://www.modbustools.com/modbus_slave.html) - Windows 模拟器
- [pyModSlave](https://github.com/sourceperl/pyModSlave) - Python 模拟器
- [modbus-simulator](https://github.com/ClassicDIY/ModbusTool) - 跨平台工具

---

## Modbus Master 配置

### 1. 编辑 Modbus 配置文件

打开 `config/modbus_sensors.toml`，配置 Modbus Master 参数：

```toml
# ModbusTcp 传感器配置
# 用于配置通过 ModbusTcp 协议读取真实传感器数据

[global]
mode = "modbus_tcp"

[server]
host = "192.168.1.100"    # Modbus Slave 设备 IP 地址
port = 502                # Modbus TCP 端口（标准端口）
timeout_ms = 1000         # 读取超时时间（毫秒）

# ========== 传感器配置 ==========

[ad1_load]
name = "Load Cell"        # 传感器名称
slave_id = 1              # Slave ID（从站地址）
register_address = 0      # 起始寄存器地址
register_count = 2        # 寄存器数量（Float32 = 2）
data_type = "Float32"     # 数据类型: UInt16, Int16, Float32
byte_order = "BigEndian"  # 字节序: BigEndian, LittleEndian

[ad2_radius]
name = "Radius Sensor"
slave_id = 1
register_address = 2      # 地址 2-3
register_count = 2
data_type = "Float32"
byte_order = "BigEndian"

[ad3_angle]
name = "Angle Sensor"
slave_id = 1
register_address = 4      # 地址 4-5
register_count = 2
data_type = "Float32"
byte_order = "BigEndian"
```

### 2. 配置参数说明

#### [server] 部分

| 参数 | 说明 | 示例 |
|------|------|------|
| `host` | Modbus Slave 设备 IP 地址 | `"192.168.1.100"` |
| `port` | Modbus TCP 端口 | `502` |
| `timeout_ms` | 读取超时时间（毫秒） | `1000` |

#### [adX_xxx] 传感器部分

| 参数 | 说明 | 可选值 |
|------|------|--------|
| `name` | 传感器名称（用于日志） | 任意字符串 |
| `slave_id` | Slave ID（从站地址） | `1-247` |
| `register_address` | 起始寄存器地址 | `0-65535` |
| `register_count` | 寄存器数量 | `1`（UInt16/Int16）或 `2`（Float32）|
| `data_type` | 数据类型 | `"UInt16"`, `"Int16"`, `"Float32"` |
| `byte_order` | 字节序 | `"BigEndian"`, `"LittleEndian"` |

### 3. 数据类型选择

| 数据类型 | 寄存器数量 | 数值范围 | 适用场景 |
|---------|-----------|---------|---------|
| `UInt16` | 1 | 0 ~ 65535 | 无符号整数（如 AD 原始值）|
| `Int16` | 1 | -32768 ~ 32767 | 有符号整数 |
| `Float32` | 2 | ±3.4E38 | 浮点数（推荐，精度高）|

**推荐**: 使用 `Float32` 类型，可以直接存储物理量（如 15.5 吨、8.3 米）。

### 4. 字节序说明

Modbus 浮点数有两种字节序：

- **BigEndian** (ABCD): 高字节在前（推荐，Modbus 标准）
- **LittleEndian** (DCBA): 低字节在前

**如何确定字节序**:
1. 查看 Modbus Slave 设备文档
2. 如果不确定，先尝试 `BigEndian`
3. 如果读取的数值异常（如 NaN、极大/极小值），尝试切换到 `LittleEndian`

---

## 启用 Modbus TCP

### 1. 修改管道配置

编辑 `config/pipeline_config.toml`，将 `use_simulator` 改为 `false`：

```toml
[collection]
interval_ms = 10          # 采集间隔（毫秒）
buffer_size = 1000        # 缓冲区大小
use_simulator = false     # ✅ 改为 false，启用真实传感器
```

### 2. 重启应用程序

```bash
# 停止当前运行的应用
pkill qt-rust-demo

# 重新启动应用
./qt-rust-demo
```

### 3. 检查日志

应用启动后，查看日志确认 Modbus 连接状态：

```
[INFO] Modbus TCP enabled: connecting to 192.168.1.100:502
[INFO] AD1 Modbus connected: Load Cell
[INFO] AD2 Modbus connected: Radius Sensor
[INFO] AD3 Modbus connected: Angle Sensor
[INFO] Data collector started (10ms interval)
```

如果连接失败，会显示错误信息：

```
[ERROR] AD1 Modbus 连接失败: Connection refused
[ERROR] Failed to collect sensor data: Modbus read error
```

---

## 测试验证

### 1. 网络连通性测试

在运行应用的设备上，测试是否能连接到 Modbus Slave：

```bash
# 测试网络连通性
ping 192.168.1.100

# 测试 Modbus TCP 端口
nc -zv 192.168.1.100 502
# 或
telnet 192.168.1.100 502
```

**预期结果**:
```
Connection to 192.168.1.100 502 port [tcp/*] succeeded!
```

### 2. 使用 Modbus 测试工具

推荐使用 **Modbus Poll** 或 **mbpoll** 测试读取：

```bash
# 使用 mbpoll 读取寄存器 0-5（3 个 Float32）
mbpoll -a 1 -r 0 -c 6 -t 4 192.168.1.100

# 参数说明:
# -a 1: Slave ID = 1
# -r 0: 起始地址 = 0
# -c 6: 读取 6 个寄存器（3 × Float32）
# -t 4: 类型 = Holding Register
```

**预期输出**:
```
[0]: 16457  (Float32 高位)
[1]: 0      (Float32 低位)
[2]: 16384  (Float32 高位)
[3]: 0      (Float32 低位)
[4]: 16968  (Float32 高位)
[5]: 0      (Float32 低位)
```

### 3. 应用内验证

启动应用后，在监控界面查看：

- ✅ 传感器连接状态显示为"已连接"
- ✅ 载荷、半径、角度数值正常更新
- ✅ 数值范围合理（如载荷 0-50 吨，半径 0-20 米）

---

## 故障排查

### 问题 1: 连接失败 "Connection refused"

**原因**:
- Modbus Slave 设备未启动
- IP 地址或端口配置错误
- 防火墙阻止连接

**解决方案**:
1. 检查 Modbus Slave 设备是否运行
2. 确认 IP 地址和端口正确
3. 检查防火墙规则：
   ```bash
   # 允许 Modbus TCP 端口
   sudo ufw allow 502/tcp
   ```

### 问题 2: 读取超时 "Modbus read error"

**原因**:
- Slave ID 不匹配
- 寄存器地址不存在
- 网络延迟过高

**解决方案**:
1. 确认 Slave ID 配置正确（1-247）
2. 确认寄存器地址在设备支持范围内
3. 增加超时时间：
   ```toml
   [server]
   timeout_ms = 3000  # 增加到 3 秒
   ```

### 问题 3: 读取的数值异常（NaN、极大值）

**原因**:
- 字节序配置错误
- 数据类型不匹配

**解决方案**:
1. 尝试切换字节序：
   ```toml
   byte_order = "LittleEndian"  # 改为 LittleEndian
   ```
2. 确认数据类型与设备一致

### 问题 4: 数值不更新

**原因**:
- Modbus Slave 设备数据未变化
- 采集间隔过长

**解决方案**:
1. 确认 Modbus Slave 设备数据在变化
2. 减小采集间隔：
   ```toml
   [collection]
   interval_ms = 100  # 改为 100ms
   ```

### 问题 5: 应用崩溃或卡死

**原因**:
- Modbus 连接阻塞主线程
- 内存泄漏

**解决方案**:
1. 检查日志中的错误信息
2. 确保 Modbus 读取在后台线程执行
3. 重启应用并监控资源使用

---

## 高级配置

### 1. 多 Slave 设备

如果传感器分布在不同的 Slave 设备上：

```toml
[ad1_load]
slave_id = 1  # 设备 1
register_address = 0

[ad2_radius]
slave_id = 2  # 设备 2
register_address = 0

[ad3_angle]
slave_id = 3  # 设备 3
register_address = 0
```

### 2. 不同数据类型混合

```toml
[ad1_load]
data_type = "Float32"
register_count = 2

[ad2_radius]
data_type = "UInt16"  # 使用整数类型
register_count = 1

[ad3_angle]
data_type = "Int16"   # 有符号整数
register_count = 1
```

### 3. 性能优化

对于高频采集（如 10ms），建议：

```toml
[server]
timeout_ms = 500  # 减小超时时间

[collection]
interval_ms = 10  # 高频采集
buffer_size = 2000  # 增大缓冲区
```

---

## 配置文件位置

| 文件 | 路径 | 说明 |
|------|------|------|
| Modbus 配置 | `config/modbus_sensors.toml` | Modbus Master 参数 |
| 管道配置 | `config/pipeline_config.toml` | 启用/禁用模拟器 |
| 日志配置 | `config/logging.toml` | 日志级别和输出 |

---

## 总结

切换到 Modbus TCP 的完整步骤：

1. ✅ 配置 Modbus Slave 设备（IP、端口、寄存器）
2. ✅ 编辑 `config/modbus_sensors.toml`（Master 参数）
3. ✅ 修改 `config/pipeline_config.toml`（`use_simulator = false`）
4. ✅ 重启应用程序
5. ✅ 检查日志和监控界面

**注意事项**:
- 确保网络连通性
- 确认 Slave ID 和寄存器地址正确
- 根据设备文档选择正确的数据类型和字节序
- 测试时先使用 Modbus 模拟器验证配置

**下一步**:
- 参考 `doc/PIPELINE_CONFIG_GUIDE.md` 了解管道配置
- 参考 `doc/LOGGING_GUIDE.md` 配置日志输出
- 参考 `crates/sensor-simulator/README.md` 了解传感器接口

---

**文档版本**: 1.0  
**更新日期**: 2026-04-07  
**适用版本**: qt-rust-demo v0.1.0
