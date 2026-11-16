# 发布流程

推送 `v*` tag 触发 GitHub Actions，出包、签名、生成 `latest.json` 并发布 Release。发布配置见 [release.yml](../.github/workflows/release.yml)，changelog 规则见 [cliff.toml](../cliff.toml)。

## 前提

- 三处版本号一致：根 `package.json`、`frontend/package.json`、`src-tauri/Cargo.toml`。
- CI 的签名私钥存于 GitHub Secret `TAURI_SIGNING_PRIVATE_KEY`。
- 本地出包前设置私钥：
  ```powershell
  $env:TAURI_SIGNING_PRIVATE_KEY = Get-Content -Raw "$env:USERPROFILE\.tauri\fishvn.key"
  ```

## 发布顺序

版本号与日期由 tag 提供。

| 步骤 | 操作 |
|---|---|
| 1 | 改三处版本号，提交 |
| 2 | `git tag vX.Y.Z` |
| 3 | `npm run changelog`，生成 `[vX.Y.Z] - 日期` 段 |
| 4 | 润色 `CHANGELOG.md`，提交 |
| 5 | `git tag -f vX.Y.Z`，使 tag 指向含 changelog 的提交 |
| 6 | `git push origin main` 与 `git push origin vX.Y.Z` |

第 2 步在生成 changelog 之前。`git cliff` 依据 tag 切分提交，tag 存在时生成正式版本段。

Release 显示名由 workflow 的 `releaseName: FishVN v__VERSION__` 生成。移动已推送的 tag 用 `git push -f origin vX.Y.Z`。

## CHANGELOG 润色

CHANGELOG 由 Conventional Commits 自动生成。发布时轻润色，不改写成宣传文案。

- 清除发布流程自身的提交：发布、升级版本号、生成变更日志、CI 与构建修复。
- 只陈述功能现状，不写技术栈变更与版本对比。
- 保留提交的 hash 引用，便于追溯。

面向用户的 Release 正文另写。

## Release 正文

CI 写入固定的 SmartScreen 警告。功能列表在 CI 完成后手动补入：

```
FishVN vX.Y.Z 更新说明

✨ 新增

- ...

🔧 优化

- ...

---

⚠️ Windows 安装包未做代码签名。SmartScreen 可能提示「Windows 已保护你的电脑」，点「更多信息 → 仍要运行」即可继续；UAC 显示「未知发布者」属正常情况。请优先校验 SHA-256（见下方），哈希一致后再运行。此警告是预期行为，程序本身不进行任何可能损害电脑的操作。

SHA-256（FishVN_X.Y.Z_x64-setup.exe）
粘贴哈希值
```

- 新增与优化按能力划分。此前没有的能力算新增，已有功能的改进算优化。
- 描述功能是什么、能做什么，术语与 `frontend/src/lib/i18n-zh.js` 一致。
- 哈希取自本次发布的安装包：
  ```powershell
  Get-FileHash -Algorithm SHA256 .\FishVN_X.Y.Z_x64-setup.exe
  ```
- 正文写入临时文件，`gh release edit vX.Y.Z --notes-file <文件>`，写完删除。

## 收尾核对

CI 约 8 分钟完成。核对三项：

```bash
gh run list --limit 1
gh release view vX.Y.Z --json tagName,name,assets
curl -fsSL https://github.com/ScarletFish/FishVN/releases/latest/download/latest.json
```

- Release 非草稿、非预发布，资产含安装包、`.sig` 和 `latest.json`。
- `latest.json` 的 `version` 与安装包文件名对应本次版本号，`windows-x86_64` 和 `windows-x86_64-nsis` 指向 NSIS 安装包并带签名。
