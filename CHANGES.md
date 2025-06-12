# 更新日志

## 2.3.0 (2025-03-20)

注意：从本次发布开始，该模板的版本号将采用如下格式：
[Tauri.major].[Tauri.minor].[Template.version]

- 新增：添加 `tauri-plugin-prevent-default` 插件以阻止浏览器默认快捷键。
- 改进：默认开启浏览器开发者工具。
- 改进：使用更轻量的图标库以加快开发速度。
- 改进：更新了 Vite 构建配置。
- 重构：升级 Tailwind 到 v4。
- 修复：允许 pnpm 的 postinstall 脚本（修复 #87，感谢 @onurusluca 提出问题）
- 优化：升级所有依赖项。

## 2.0.2 (2024-11-16)

- 新增：添加 `@egoist/tailwindcss-icons` 插件。
- 改进：在 Tauri 配置中添加优化设置。
- 优化：清理从 v1 到 v2 的残留配置。
- 修复：修复 Vite、Eslint 配置。

## 2.0.0 (2024-11-07)

- 升级：将模板升级至 Tauri v2。
- 新增：添加 Pinia。
- 新增：添加 CSP（内容安全策略）以增强安全性。
- 新增：新增 `bump` 命令以程序化方式升级版本号。
- 修复：修复 Vue 开发者工具。
- 优化：迁移到新的 Renovate 配置。
- 优化：升级所有前端依赖。

## 0.4.0 (2023-12-09)

- 新增：启用 Renovate Bot 的自动合并功能。
- 修复：使用 `cross-env` 设置环境变量以兼容 Windows。
- 优化：调整项目设置以更好地匹配官方 Tauri 模板。
- 优化：升级所有前端依赖。
- 优化：升级所有 GitHub 工作流中的 action 版本。

## 0.3.0 (2022-09-29)

- 新增：为 VSCode 添加调试配置。感谢 @gabriel-andreescu 的贡献！
- 新增：添加 vue-devtools 集成。
- 新增：启动时自动打开浏览器开发者工具。
- 新增：更改应用图标为 Vue 图标。
- 新增：在默认 dev 命令中添加 `RUST_BACKTRACE=1`。
- 修复：创建空的 dist 文件夹以满足 Rust 插件需求。
- 修复：修复 Vite 构建目标以匹配 tauri create app 配置。
- UI：更智能的样式和 HTML 结构。
- 文档：说明如何实现类似 Electron 的 `titleBarStyle: 'hidden'` 窗口效果，并指出同步后端命令会阻塞 UI。
- 优化：升级所有依赖项。

## 0.2.0 (2022-07-10)

- 修复：更改应用名称（`src-tauri/tauri.conf.json` 中的 `packageName`）。
- 修复：使 Vite 配置更加稳健。
- 工具：添加 `.vscode` 文件夹并推荐安装 Volar 扩展。
- 重构：从前端依赖中移除了 `vite-plugin-tauri`。
- 优化：改进 Vite 配置以提升开发体验。
- 优化：升级所有依赖项。
- 文档：优化 README 内容。

## 0.1.0 (2022-06-16)

欢迎 Tauri 1.0！

- 新增：添加独立的发布流程。
- 重构：将菜单改为使用 `Menu::os_default`。感谢 @JonasKruckenberg 的建议！
- 文档：增加更多使用文档，修复前提条件链接。
- 优化：升级所有 Vite + Tauri 依赖。

## 0.0.1 (2022-04-27)

- 初始版本