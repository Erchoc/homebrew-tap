# homebrew-tap

个人 Homebrew Tap — Rust CLI 工具集。

```bash
brew install erchoc/tap/<工具名>
```

## 工具列表

| 工具 | 安装命令 | 说明 |
|------|---------|------|
| [vvpn](packages/vvpn/) | `brew install erchoc/tap/vvpn` | Stash VPN 环境诊断工具 |

## 手动安装

所有工具均提供 macOS 通用二进制（x86_64 + aarch64），从 [Releases](https://github.com/Erchoc/homebrew-tap/releases) 下载后放到 `$PATH` 即可。

## 开发

每个工具独立维护在 `packages/<name>/` 下：

```bash
cd packages/vvpn
cargo run -- scan          # debug 模式运行
cargo test                 # 运行测试
```

### 发版

```bash
# 修改 packages/vvpn/Cargo.toml 中的 version
git tag v0.1.0             # vvpn 用 v* 格式
git push origin v0.1.0     # 触发 CI 构建 + 发布 + 自动更新 Formula
```

## License

MIT
