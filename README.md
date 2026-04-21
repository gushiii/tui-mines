# 🚀 Rust TUI Minesweeper (MVC Edition)
[![Rust](https://img.shields.io/badge/rust-1.56%2B-orange)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

一款基于 Rust 开发的终端扫雷游戏，采用经典的 MVC (Model-View-Controller) 架构实现。它不仅拥有高性能的渲染，还集成了 Nerd Fonts 图标、专业级操作逻辑以及跨平台原生音效反馈。

## ✨ 项目亮点

* MVC 架构：逻辑、视图、控制完全解耦，代码结构清晰，易于扩展。
* 支持 Nerd Fonts 图标。
   * 自适应动态居中渲染。
   * 胜利/失败弹窗。
   * 辅助功能：光标周围 3x3 范围高亮，辅助判定雷区。
* 开局保护：第一下点击必不踩雷（且自动炸开 3x3 安全区）。
   * 快速翻开 (Chording)：当数字周围旗帜数满足时，按空格可一键清空周边。
   * 难度切换：支持 1 (初级)、2 (中级)、3 (高级) 实时切换。
* 跨平台支持（`macOS` / `Linux` / `Windows`）。
   * 标记、取消标记、点击、爆炸及胜利均有独立音效。
* 持久化排行：自动将最佳成绩保存至本地 scores.json。

## 🛠️ 安装与运行
### 1. 前置要求

* 安装 [Nerd Fonts](https://www.nerdfonts.com/) 并将其设为终端字体（否则图标会显示为乱码）。
* 安装 Rust 工具链 (cargo, rustc)。

### 2. 快速启动

#### 克隆仓库
```shell
git clone https://github.com/gushiii/tui-mines.git
cd tui-mines
```
#### 运行游戏
```shell
cargo run
```

## 🎮 操作说明

| 按键 | 动作 |
|---|---|
| 方向键 | 移动光标 (带有 3x3 辅助高亮) |
| 空格 (Space) | 翻开格子 / 已翻开格触发 Chording (自动清空) |
| F | 标记 (🚩) / 取消标记 (仅剩雷数 > 0 时可标记) |
| 1 / 2 / 3 | 切换难度 (初级 / 中级 / 高级) |
| Q | 退出游戏 |
| 任意键 | 游戏结束后重新开始 |

## 📁 项目结构
```tree
tui-mines/
├── Cargo.toml
└── src/
    ├── main.rs          # 程序入口 (Controller 逻辑)
    ├── model.rs         # 游戏逻辑与数据结构 (Model)
    └── view.rs          # 界面渲染逻辑 (View)

```

* Model: 处理棋盘生成、洗牌埋雷算法、递归翻开逻辑及排行榜 I/O。
* View: 基于 ratatui 的 Table 和 Paragraph 实现的彩色渲染层。
* Controller: 负责 crossterm 事件监听及非阻塞式的计时器更新循环。

## ⚙️ 配置文件
游戏会自动在当前目录下生成 scores.json。

{
  "scores": [
    { "date": "2026-04-10 14:30", "seconds": 45, "difficulty": "10x10" }
  ]
}

## 📜 开源协议
本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。
