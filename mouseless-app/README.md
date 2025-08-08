# Mouseless App - React + Tauri + TypeScript

这是 Mouseless 应用的前端部分，使用 React + TypeScript + Tauri 构建的现代桌面应用。

## 技术栈

- **React 18** - 现代 React 框架
- **TypeScript** - 类型安全的 JavaScript
- **Tauri 2.0** - 轻量级桌面应用框架
- **Vite** - 快速的构建工具
- **Tailwind CSS** - 实用优先的 CSS 框架
- **Framer Motion** - 流畅的动画库
- **Lucide React** - 美观的图标库

## 功能特性

### UI 覆盖系统
- **网格覆盖** - 可配置的网格系统，支持标签显示
- **区域覆盖** - 9宫格区域选择
- **预测目标覆盖** - 基于置信度的智能目标预测

### 动画效果
- 流畅的进入/退出动画
- 悬停交互效果
- 基于置信度的动态大小调整

### 权限管理
- macOS 辅助功能权限检查
- 用户友好的权限请求界面

## 开发设置

### 前置要求
- Node.js 18+
- Rust 1.70+
- Tauri CLI

### 安装依赖
```bash
npm install
```

### 开发模式
```bash
npm run dev
```

### 构建应用
```bash
npm run build
```

### 运行 Tauri 开发模式
```bash
npm run tauri dev
```

### 构建 Tauri 应用
```bash
npm run tauri build
```

## 项目结构

```
src/
├── components/          # React 组件
│   ├── GridOverlay.tsx     # 网格覆盖组件
│   ├── AreaOverlay.tsx     # 区域覆盖组件
│   ├── PredictionOverlay.tsx # 预测覆盖组件
│   └── PermissionCheck.tsx   # 权限检查组件
├── types.ts            # TypeScript 类型定义
├── App.tsx             # 主应用组件
├── main.tsx            # React 入口点
└── index.css           # 全局样式

dist/                   # 构建输出
overlay.html           # 覆盖窗口 HTML
index.html             # 主窗口 HTML
```

## IPC 通信

应用通过 Tauri 的 IPC 系统与 Rust 后端通信：

### 事件监听
- `configure-grid` - 配置网格覆盖
- `configure-area` - 配置区域覆盖  
- `configure-prediction` - 配置预测目标
- `hide-overlays` - 隐藏所有覆盖

### 命令调用
- `check_accessibility_permissions` - 检查权限
- `request_accessibility_permissions` - 请求权限
- `show_grid_overlay` - 显示网格覆盖
- `show_area_overlay` - 显示区域覆盖
- `show_prediction_targets` - 显示预测目标
- `hide_all_overlays` - 隐藏所有覆盖

## 样式系统

使用 Tailwind CSS 进行样式管理，包含：

- 响应式设计
- 暗色主题支持
- 自定义动画
- 覆盖窗口专用样式

## 开发注意事项

1. **透明窗口** - 覆盖窗口使用透明背景
2. **权限处理** - macOS 需要辅助功能权限
3. **多窗口管理** - 支持多个覆盖窗口同时存在
4. **性能优化** - 使用 React.memo 和 useMemo 优化渲染

## 调试

- 使用浏览器开发者工具调试前端
- 使用 `console.log` 进行日志输出
- Tauri 控制台会显示 Rust 后端日志