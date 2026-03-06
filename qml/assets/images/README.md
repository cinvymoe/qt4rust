# 图标资源说明

本目录包含从 Figma 设计中提取的图标资源。

## 图标列表

### 应用图标
- `icon-logo.png` - 应用 Logo (24x24)
- `icon-alert.png` - 报警图标 (24x24)
- `icon-danger.png` - 危险状态图标 (48x48)

### 数据指标图标
- `icon-weight.png` - 载荷图标 (20x20)
- `icon-radius.png` - 工作半径图标 (20x20)
- `icon-angle.png` - 吊臂角度图标 (20x20)
- `icon-moment.png` - 力矩百分比图标 (24x24)

### 导航图标
- `icon-home.png` - 主界面图标 (24x24)
- `icon-chart.png` - 数据曲线图标 (24x24)
- `icon-alarm-record.png` - 报警记录图标 (24x24)
- `icon-settings.png` - 设置图标 (24x24)

### 图形资源
- `canvas-crane.png` - 起重机臂架示意图 (462x462)

## 使用方式

在 QML 中引用图标：

```qml
Image {
    source: "assets/images/icon-logo.png"
    width: Theme.iconSizeMedium
    height: Theme.iconSizeMedium
}
```

## 注意事项

1. 所有图标均为 SVG 格式（虽然扩展名为 .png）
2. 图标颜色可以通过 QML 的 ColorOverlay 进行调整
3. 建议使用 Theme 中定义的图标尺寸常量
4. 图标资源来自 Figma，有效期 7 天，需要定期更新

## 更新日期

最后更新: 2026-03-05
