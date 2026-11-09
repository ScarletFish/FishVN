<p align="center">
<img src="src-tauri/icons/128x128.png" alt="FishVN" width="80" />
</p>

<h1 align="center">FishVN</h1>

<div align="center">
<img alt="Version" src="https://img.shields.io/badge/version-v0.3.0-blue">
<img alt="Windows 10+" src="https://img.shields.io/badge/Windows-10%2B-blue?logo=windows">
<img alt="Tauri v2" src="https://img.shields.io/badge/Tauri-v2-orange?logo=tauri">
<img alt="Svelte 5" src="https://img.shields.io/badge/Svelte-5-ff3e00?logo=svelte">
<img alt="Rust" src="https://img.shields.io/badge/Rust-2021-dea584?logo=rust">
<img alt="VNDB" src="https://img.shields.io/badge/VNDB-metadata-yellow">
</div>

<p align="center">
  <img src="docs/images/README/hero.png" alt="同一详情页的浅色与深色主题" />
</p>
<p align="center"><sub>浅色 / 深色主题</sub></p>

## 简要介绍

FishVN 是一个 **Windows 桌面应用**，用来管理本地安装（甚至未安装）的 Galgame：**导入、记录、启动、计时、回顾**。

> [!IMPORTANT]
> 仅支持 Windows 10 / 11。转区与缩放依赖 Locale Emulator 与 Magpie，需自行安装并在设置页填写路径。

## 功能

- **添加**：手动添加、从启动文件添加、扫描目录批量识别。
- **VNDB 同步支持**：支持导入 VNDB 的愿望单、游玩中等状态游戏，双向可选同步。
- **资料补全**：批量拉取缺失的元数据、封面与角色，逐条确认后写入；未匹配的条目可改词重搜或重试。
- **数据自定义**：VNDB 源仅为建议值，本地全元数据可新建、可修改、可查询。
- **启动项**：支持多游戏安装实例添加（原版 / 汉化版）；可单独设置转区、Magpie 超分缩放（全屏 / 窗口）、启动附加参数。
- **计时**：前景 / 存活双口径，可以精准记录窗口在前台的时间或者进程存活时间，设置可选统计口径。
- **标签**：标签与特征提供中文译名；成人向与剧透标签可隐藏，支持个人偏好设置。
- **角色**：颗粒度到角色特征的角色系统。详情页首页展示角色卡片。
- **查询**：按标签、特征、年代、评分、时长等组合条件筛选作品与角色，结果可导出。
- **统计**：基于结构化的标签，本地数据统计的游玩活跃度、时长排行、发行年代分布、题材词云等图表。
- **外观**：六色强调色 + 深浅主题支持。横幅可选柔和 / 清晰两档 + 自定义图片。支持图片展示焦点选取。

<p align="center">
  <img src="docs/images/README/dashboard.png" alt="首页：统计概览、继续游玩与我的游戏封面墙" />
</p>
<p align="center"><sub>首页：统计概览与游戏库</sub></p>

<p align="center">
  <img src="docs/images/README/query.png" alt="查询：条件组合筛选后的作品结果" />
</p>
<p align="center"><sub>查询页</sub></p>

## 快速开始

[下载最新安装包](https://github.com/ScarletFish/FishVN/releases/latest)

从源码构建，需要 Node.js 22+ 与 Rust stable（MSVC 工具链）。

```bash
npm install
npm run dev      # 起开发窗口
npm run build    # 前端产物到 frontend/dist
```

安装包构建走 `start.bat` 菜单 3。

## 目标

该项目基于个人需求，所以可能不会满足所有人的需要。如果它有帮助到你，那就再好不过了。同类中有不少优秀的项目，我也参考借鉴了他们的实现，特此列出：

- [Playnite](https://github.com/JosefNemec/Playnite)
- [PotatoVN](https://github.com/GoldenPotato137/PotatoVN)
- [ReinaManager](https://github.com/huoshen80/ReinaManager)

### 计划外的功能

- 多源刮削（Bangumi、YmGal 等）
- 非 Windows 平台（macOS / Linux）
- 游戏存档管理（备份、云同步）

## 技术栈

- 前端：Svelte 5、Vite、纯 CSS
- 后端与桌面：Tauri v2、Rust、rusqlite
- 数据源：VNDB API

## 许可

本项目采用 MIT 许可证，详情请见 [LICENSE](./LICENSE)。第三方声明见 [ATTRIBUTION.md](./src-tauri/resources/vndb/ATTRIBUTION.md)。

## 致谢

[Tauri](https://tauri.app)、[VNDB](https://vndb.org)、[PotatoVN](https://github.com/GoldenPotato137/PotatoVN)、[Playnite](https://github.com/JosefNemec/Playnite)、[VNDBTranslatorLib](https://greasyfork.org/scripts/445990)、[Locale Emulator](https://github.com/xupefei/Locale-Emulator)、[Magpie](https://github.com/Blinue/Magpie)、[Noto Sans SC](https://fonts.google.com/noto)。

<p align="center">
  <sub>ScarletFish</sub>
</p>
