<div align="center">

# <img src="assets/icon.svg" width="72" valign="middle" alt="idalib-cli"/>️ idalib-cli

**面向 IDA Pro IDALib 的 Agent 原生 CLI** — 每条命令一个 `-d/--db`，每个回答都是 JSON。

**[English](README.md)** · **[中文](README.zh-CN.md)**

![IDA Pro 9.1](https://img.shields.io/badge/IDA_Pro-9.1-blue) ![idalib-rs 0.6.1](https://img.shields.io/badge/idalib--rs-0.6.1-orange) ![v0.9.1](https://img.shields.io/badge/version-0.9.1-green) ![Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-lightgrey)

</div>

---

## ✨ 特性

- 🗄️ **无状态** — 每条命令带 `-d/--db <路径>`（二进制或 `.i64`）；无守护进程、无会话簿记。
- 💾 **IDB 状态持久** — IDB 文件本身就是状态：注释、书签、命名、分析结果跨进程保留。
- 🧩 **全量 IDALib 覆盖** — 段、函数、CFG、反汇编、Hex-Rays 反编译、字符串、命名、交叉引用、入口点、元数据、注释、书签、FLIRT 签名。
- ⚡ **batch & parallel** — 多条命令只开一次 IDB；一条命令扇出到多个数据库并发执行（支持 glob）。
- 🤖 **Agent 优先** — stdout 输出一个 JSON，错误走 stderr；内置 Codex / Claude Code / OpenCode 可用的 skills。

## 🚀 安装

> **前置条件**：IDA Pro 9.1（已授权、启动过一次）· 已解压 IDA 9.1 SDK（仅编译期需要）· Rust + LLVM/Clang（[bindgen 要求](https://rust-lang.github.io/rust-bindgen/requirements.html)）

```bash
export IDADIR="/Applications/IDA Professional 9.1.app/Contents/MacOS"  # IDA 安装目录
export IDASDKDIR=$HOME/idasdk91                                        # 解压后的 SDK

git clone <this-repo> && cd idalib-cli
cargo install --path .

idalib-cli info    # ✅ 验证工具版本、IDA 版本、许可证
```

> SDK 仅**编译期**需要；编译产物运行时链接你本机的 IDA 库。无 SDK 的开发检查：`cargo test --no-default-features --features stub-idalib`。

## 🎮 命令与用法

每条命令都需要 `-d/--db <路径>` — 一个 IDB 文件（`.i64`）或一个二进制（首次
使用时会在其旁自动生成 IDB）。地址接受 `0x401000` 或 `401000`。输出始终是
一个 JSON 文档；错误走 stderr，退出码非零。

### 数据库

| 命令 | 说明 |
|---|---|
| `idalib-cli -d <bin> db info` | 路径、IDB 状态、大小 |
| `idalib-cli -d <bin> db open [--save] [--auto-analyse]` | 显式打开/创建并分析 |
| `idalib-cli -d <bin> db close` | 刷盘 |
| `idalib-cli db remove -d <bin-or-i64>` | 删除 IDB（绝不删二进制） |

### 查询

| 命令 | 说明 |
|---|---|
| `idalib-cli -d <db> meta` | 文件类型、编译器、位数 |
| `idalib-cli -d <db> processor` | 处理器信息 |
| `idalib-cli -d <db> segments` | 所有段 |
| `idalib-cli -d <db> segments-by-range -a <ea>` | 地址所在段 |
| `idalib-cli -d <db> functions [-u]` | 函数列表（`-u` = 跳过库/桩函数） |
| `idalib-cli -d <db> function -a <ea>` | 单个函数：CFG、基本块、xrefs |
| `idalib-cli -d <db> disasm -a <ea> [-n N]` | 反汇编 N 条指令（默认 8） |
| `idalib-cli -d <db> decompile -a <ea> [--all-blocks]` | Hex-Rays 伪代码 |
| `idalib-cli -d <db> insn -a <ea>` | 单条指令 |
| `idalib-cli -d <db> strings` | 字符串列表 |
| `idalib-cli -d <db> names` | 命名位置 |
| `idalib-cli -d <db> xrefs [-a <ea>] [--all]` | 到某地址的交叉引用，或全部 |
| `idalib-cli -d <db> entries` | 入口点 |

### 编辑（持久化进 IDB）

| 命令 | 说明 |
|---|---|
| `idalib-cli -d <db> comments get\|set\|append\|remove -a <ea> [-c "文本"]` | 注释 |
| `idalib-cli -d <db> bookmarks list\|add\|get\|remove -a <ea> [-d "描述"]` | 书签 |
| `idalib-cli -d <db> signatures --make [--only-pat]` | 生成 FLIRT 签名 |

### 组合

| 命令 | 说明 |
|---|---|
| `idalib-cli -d <db> batch -- <op> [<op>...]` | 顺序执行多条，IDB 只开一次 |
| `idalib-cli parallel -d <列表\|glob> [--jobs N] -- <op>` | 一条命令跑多个库，每库一个子进程 |
| `idalib-cli info [--version\|--ida\|--all]` | 工具 / IDA 版本、许可证 |

### 场景示例

**🔎 未知样本概览**

```bash
idalib-cli -d ./sample meta            # 是什么？（文件类型/编译器/位数）
idalib-cli -d ./sample segments        # 内存布局
idalib-cli -d ./sample strings         # 快速找线索
idalib-cli -d ./sample functions -u    # 仅用户代码
```

**🔍 深挖某个函数**

```bash
idalib-cli -d ./sample function -a 0x401000       # CFG + 基本块 + xrefs
idalib-cli -d ./sample decompile -a 0x401000      # 读伪代码
idalib-cli -d ./sample disasm -a 0x401000 -n 20   # 或原始指令
idalib-cli -d ./sample xrefs -a 0x401000 --all    # 谁调用了它
```

**📝 标注结论（跨进程/跨 Agent 可见）**

```bash
idalib-cli -d ./sample comments set -a 0x401000 -c "解析配置，参见 0x402100"
idalib-cli -d ./sample bookmarks add -a 0x401000 -d "入口点"
idalib-cli -d ./sample comments get -a 0x401000    # 验证
```

**⚡ 大量样本批量分析**

```bash
# 第一遍：为每个样本建 IDB + 概览
idalib-cli parallel -d "./samples/*.bin" -- "batch -- meta functions -u"

# 第二遍：在每个 IDB 里反编译同一个热点函数
idalib-cli parallel -d "./samples/*.i64" --jobs 8 -- "decompile -a 0x401000"
```

**🤖 Agent 式批量取证（一个 JSON 文档）**

```bash
idalib-cli -d ./sample batch -- "meta" "segments" "functions -u" "decompile -a 0x401000"
```

面向 Agent 的工作流指南在 [`skills/`](skills/)；可运行的端到端示例在
[`examples/workflow.sh`](examples/workflow.sh)。

<details>
<summary>输出示例（<code>decompile</code>）</summary>

```json
{
  "id": 7,
  "start": "0x401000",
  "end": "0x401080",
  "size": 128,
  "name": "main",
  "blocks": 3,
  "decompiled": true,
  "pseudocode": "int __cdecl main(...) { ... }"
}
```

</details>

## ⚙️ 配置

可选配置 `~/.idapro/idalib-cli/config.toml`（基目录：`$IDALIB_CLI_HOME`）：

| 字段 | 说明 |
|---|---|
| **idadir** | IDA 安装目录（默认自动探测） |
| **idb_dir** | 新 IDB 的生成位置（默认在二进制同目录） |
| **default_db** | 省略 `-d` 时使用 |
| **save** | 每条命令后保存 IDB（默认 `true`） |
| **auto_analyse** | 创建 IDB 时跑完整自动分析（默认 `true`） |

## ❓ 常见问题

<details>
<summary>传入二进制时，IDB 生成在哪？</summary>

在二进制同目录：`./target.bin` → `./target.bin.i64`。可在配置中设置 `idb_dir`
更改位置。

</details>

<details>
<summary>batch / parallel 到底做了什么？</summary>

`batch` 只打开 IDB 一次，所有 op 复用同一个句柄（最后统一保存）— 适合对
同一个库要多个事实的场景。`parallel` 为每个数据库派一个子进程（IDALib 非线程
安全，故用进程隔离），工作池由 `--jobs` 限流 — 适合大量样本。`-d` 接受单个
路径、逗号分隔列表或 glob（`*.i64`）。

</details>

<details>
<summary>两个进程能同时用同一个 IDB 吗？</summary>

不要。同一 IDB 同一时刻只允许一个进程。`parallel` 通过每库一个子进程保证
这一点；手动多 Agent 协作时，给每个 Agent 分配各自的 `-d` 目标。

</details>

<details>
<summary>为什么编译需要 IDA SDK？</summary>

`idalib-rs` 在编译期解析 SDK 头文件生成 FFI 绑定（bindgen）。SDK 只随你的
Hex-Rays 许可证提供，绝不会被再分发或内嵌 — 编译产物运行时链接的是你自己
的 IDA 安装。

</details>

## 📁 项目结构

```
idalib-cli/
├── src/
│   ├── cli.rs            # clap 定义；所有命令 + -d/--db
│   ├── ops/              # metadata、comments、bookmarks、db、batch、parallel、...
│   ├── session/          # config.toml 处理
│   └── helpers/          # JSON 输出视图
├── stubs/idalib/         # 仅开发用 API 桩（无 SDK 检查，绝不随产物发布）
├── tests/                # 集成测试
├── skills/               # Agent 工作流指南
└── examples/workflow.sh  # 可运行的端到端示例
```

## 🌿 版本与分支

工具版本 `x.y.z`；每个 minor 维护两条分支：

| Ref | 用途 | 示例 |
|---|---|---|
| `main` | 最新开发（`v*_dev` 的合并目标） | — |
| `v0.9_dev` | 工具 0.9.x 开发分支 | 日常开发 |
| `v0.9_release` | 工具 0.9.x 稳定分支（仅修复） | 回迁补丁 |
| `v0.9.1`（tag） | 发布点 | 当前发布 |

| 工具版本 | 兼容 IDA | idalib-rs |
|---|---|---|
| **0.9.x** | 9.1 | 0.6.1（锁定 `=0.6.1`） |
| 下一条线（`v0.10_*`） | 新 IDA 版本 | 升级后对应 |

适配新的 IDA 版本 = 升级 `idalib` 依赖 + 更新 `Cargo.toml` 的
`[package.metadata.ida]` + 开新的分支线（`v0.10_*`）。

## 📄 许可证

本项目以 [Apache License 2.0](LICENSE) 分发。`Cargo.toml` 中的 `license` 字段
声明为 `MIT OR Apache-2.0`，以保持与 [idalib-rs](https://github.com/idalib-rs/idalib)
依赖的兼容；本仓库仅附带 Apache-2.0 许可文本。**严禁提交或再分发 IDA SDK。**

---

**[English](README.md)** · **[中文](README.zh-CN.md)**
