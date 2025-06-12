# ImmediateEarth

这是一款动态壁纸应用，可以实时抓取向日葵8号卫星拍摄到的地球图像，并进行加工、渲染。然后布置到桌面上。


ImmediateEarth是一个自制的非盈利性软件，但是提供了很全面的功能，包括对壁纸的渲染、界面UI的设计、网络和系统资源的使用都进行过全面的考虑。无需任何更多操作，软件可自动为您规划好一切。

* 每一刻的实时4K地球高清图像。

* 能展现出压迫感和魅力的角度。

* 所有的下载过程都是在后台涓流下载，不会占用过高带宽。

* 当处于离线状态时，会自动使用昨日缓存中最接近时间点的照片。如果没有缓存，则使用自带的预设时间点壁纸。

* 自动根据您的计算机当前的Cpu、内存使用情况判定是否进行壁纸的渲染，避免在您处于高强度工作时占用额外资源。

* 无Dock图标、无状态栏的完全隐藏式设计。

ImmediateEarth是一款免费的动态壁纸软件
每隔十分钟左右，会抓取一张向日葵8号卫星所拍摄到的地球照片。
然后进行美化处理，渲染出一张适合用作壁纸的图片。
根据时间和太阳照射角度的不同，ImmediateEarth会对原始照片进行艺术化处理，使其更适合用作桌面。
太平洋上方的实时云层，直接展现！
ImmediateEarth通过涓流下载的方式获取资源占用极少的的网速。
当设备在执行其它复杂操作时，ImmediateEarth会自动避开图像渲染等略耗资源的操作，使用缓存的图像代替更新。
当无互联网连接时，ImmediateEarth也能通过加载本地缓存来展现不同时刻的景色


30: ImmediateEarth的运作过程
第一步、获取到地球实时高清图像：


原始图像
第二步、渲染出适合用作壁纸的图像：


加工渲染
39: 根据太阳照射角度的不同，ImmediateEarth会自动选择不同的画面布局，为了提升壁纸的表现力，所有的角度都是由人工精挑细选的。



# Tauri + Vue + Vite 模板

![Screenshot](./public/v2_screenshot.webp)

完全配置好的 Tauri 和 Vue 3（带 TypeScript）项目模板，并集成 CI 工具。

## 功能特性

- **Vue 3 (TypeScript)** 前端（支持开发工具）
- **Vite** 配置了 [AutoImport 插件](https://github.com/antfu/unplugin-auto-import)
- **Vitest** 用于单元测试
- **Github Actions** 实现自动化测试和 CI 流水线
- **VS Code** 推荐插件和调试配置

## 环境搭建

1. 安装 [Tauri 开发环境依赖](https://tauri.app/start/prerequisites/)
2. 克隆项目并安装依赖（本模板默认使用 `pnpm`）：

```sh
pnpm i
```

## 使用说明

一个 Tauri 应用至少包含两个进程：

- 核心进程 (`backend`，也叫 _main_ 进程，Electron 的术语)
- WebView 进程 (`frontend`，也叫 _renderer_ 进程)

### 🦢 前端 (TS, PnPM)

#### 启动开发服务器

前后端可以通过一条命令同时启动：

```sh
pnpm tauri dev
```

#### 执行测试

```sh
pnpm test
```

### 🦀 后端 (Rust, Cargo)

后端代码位于 `src-tauri/` 目录下（以下命令应在此目录中执行）

#### 查看过期的 Rust 依赖

如果你安装了 [cargo-outdated](https://github.com/kbknapp/cargo-outdated)：

```sh
cargo outdated
```

#### 升级 Rust 依赖

如果你安装了 [cargo-edit](https://github.com/killercup/cargo-edit)：

```sh
cargo upgrade
```

### 调试

- 默认 `dev` 命令设置了 `RUST_BACKTRACE=1`，这会让 Rust 输出完整的错误堆栈到控制台。（如果你不需要它，可以从 [package.json](file://c:\Users\YiranzaiHWin\Documents\GitHub\ImmediateEarth\package.json) 的命令中移除它）
- 如果你使用 VS Code，可以使用内置的 `Debug Tauri` 配置来调试 Rust 代码。

### 构建与发布

#### 构建

项目已集成了 GitHub Actions，每次提交或 PR 都会自动测试和构建你的应用。也可以手动构建：

```sh
pnpm tauri build
```

#### 发布新版本

1. 通过运行 `pnpm bump [x.y.z]` 来升级版本号
2. 运行 `pnpm check` 更新 [Cargo.lock](file://c:\Users\YiranzaiHWin\Documents\GitHub\ImmediateEarth\src-tauri\Cargo.lock)
3. 为你要发布的 commit 添加标签 `vX.Y.Z`
4. 编辑发布说明并推送到远程（包括标签！）
5. GitHub 工作流将自动生成一个新的草稿版本。准备就绪后发布🎉

## 其他信息

- 关注 [unessa.net on Bluesky](https://bsky.app/profile/uninen.net) 或 [@uninen on Twitter](https://twitter.com/uninen)
- 阅读我在 Tauri / Vue / TypeScript 和其他 Web 开发主题上的学习记录，请访问我的 [Today I Learned 网站](https://til.unessa.net/)

## 贡献指南

欢迎贡献！请在与其他开发者互动时遵循 [行为准则](./CODE_OF_CONDUCT.md)。