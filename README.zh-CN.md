<div align="center">

# <img src="assets/icon.svg" width="40" valign="middle" alt="idalib-cli logo"/> idalib-cli

**面向 IDA Pro IDALib 的 Agent 原生 CLI — 基于 [idalib-rs](https://github.com/idalib-rs/idalib)**

[![IDA](https://img.shields.io/badge/IDA_Pro-9.1-blue)](https://hex-rays.com/ida-pro)
[![idalib-rs](https://img.shields.io/badge/idalib--rs-0.6.1-orange)](https://github.com/idalib-rs/idalib)
[![Version](https://img.shields.io/badge/version-0.9.1-green)](#版本与分支)
[![License](https://img.shields.io/badge/license-Apache--2.0-lightgrey)](#许可证)

[English](README.md) · **简体中文**

</div>

单一可执行文件的**无状态**命令行工具，把 IDA 分析能力以结构化 JSON 的形式
暴露出来 — 专为代码 Agent（Codex、Claude Code、OpenCode）或人类用户设计。
每条命令都用 `-d/--db <路径>` 指定目标：一个 IDB 文件（`.i64`）或一个二进制
（首次使用时会在其旁自动生成 IDB）。所有状态都在 IDB 文件里 — 注释、书签、
命名、分析结果跨调用保留；多个数据库可跨进程并发分析。

---

## 功能特性

| | |
| --- | --- |
| 💾 **无状态** | 每条命令带 `-d/--db <路径>` — 无守护进程、无注册表、无会话簿记 |
| 🗄️ **IDB 状态持久** | IDB 文件本身就是状态 — 注释、书签、命名、分析结果跨进程保留 |
| 🧩 **全量 IDALib 覆盖** | 段、函数、CFG、反汇编、Hex-Rays 反编译、字符串、命名、交叉引用、入口点、元数据、注释、书签、FLIRT 签名 |
| ⚡ **批量与并发** | `batch` = 多条命令只开一次 IDB；`parallel` = 一条命令扇出到多个数据库（支持列表/glob） |
| 🤖 **Agent 优先** | 所有命令输出 JSON；统一的 `-d` 参数；内置 Agent Skills |

## 安装

> **前置条件**
> 1. 已安装 IDA Pro 9.1 并至少启动过一次（需有效许可证）
> 2. 已解压 IDA 9.1 SDK（从你的 Hex-Rays 账户下载）— 仅**编译期**需要
> 3. Rust 工具链 + LLVM/Clang（bindgen
>    [运行要求](https://rust-lang.github.io/rust-bindgen/requirements.html)）

```sh
export IDADIR="/Applications/IDA Professional 9.1.app/Contents/MacOS"  # IDA 安装目录
export IDASDKDIR=$HOME/idasdk91                                        # 解压后的 SDK

git clone <this-repo> && cd idalib-cli
cargo install --path .

idalib-cli info    # ✅ 验证：工具版本、IDA 版本、许可证
```

<details>
<summary>补充说明 & 无 SDK 的开发检查</summary>

- `IDADIR` 不设时会自动探测常见安装位置。
- 编译产物运行时链接的是**你本机**的 `libida`/`libidalib`；SDK 不会被内嵌或再分发。
- 无 SDK 时的代码级检查（仅开发用）：

  ```sh
  cargo check --no-default-features --features stub-idalib
  cargo test  --no-default-features --features stub-idalib
  cargo fmt --all --check
  ```

</details>

## 快速开始

```sh
idalib-cli -d ./target.bin functions                      # 1. 分析（自动建 IDB）
idalib-cli -d ./target.bin batch -- "meta" "segments" "strings" "functions -u"   # 2. 概览
idalib-cli -d ./target.i64 decompile -a 0x401000          # 3. Hex-Rays 伪代码
idalib-cli -d ./target.bin comments set -a 0x401000 -c "note"    # 4. 标注（持久化）
idalib-cli parallel -d "./a.i64,./b.i64" -- "functions -u"       # 5. 扇出到多个库
```

## 命令参考

> 全局选项：`-d/--db <路径>` 指定数据库（`.i64` 或二进制）— 所有命令必需 ·
> `-j/--json` 强制 JSON（默认即 JSON）· 地址接受 `0x401000` 或 `401000`

<details>
<summary><b>数据库管理</b></summary>

```sh
idalib-cli -d <bin-or-i64> db info      # 路径、IDB 状态、大小
idalib-cli -d <bin-or-i64> db close     # 刷盘（每条命令本就落盘）
idalib-cli db remove -d <bin-or-i64>    # 删除 IDB（绝不删二进制）
idalib-cli -d <bin> db open [--save/--auto-analyse]   # 显式打开 + 分析
```

</details>

<details>
<summary><b>数据库查询</b></summary>

```sh
idalib-cli segments                     # 所有段
idalib-cli segments-by-range -a <ea>    # 地址所在段
idalib-cli functions [-u]               # -u = 仅用户代码（跳过库/桩函数）
idalib-cli function  -a <ea>            # 函数详情：CFG、基本块、xrefs
idalib-cli disasm    -a <ea> [-n N]     # 反汇编 N 条指令（默认 8）
idalib-cli decompile -a <ea> [--all-blocks]   # Hex-Rays 伪代码
idalib-cli strings | names | entries
idalib-cli xrefs [-a <ea>] [--all]      # 到某地址的交叉引用，或全部函数
idalib-cli meta                         # 文件类型 / 编译器 / 位数
idalib-cli processor
idalib-cli insn      -a <ea>            # 单条指令
```

</details>

<details>
<summary><b>数据库编辑</b>（持久化进 IDB）</summary>

```sh
idalib-cli comments get|set|append|remove -a <ea> [-c "文本"]
idalib-cli bookmarks list|add|get|remove  -a <ea> [-d "描述"]
idalib-cli signatures --make [--only-pat]      # 生成 FLIRT 签名
```

</details>

<details>
<summary><b>批量与并发</b></summary>

```sh
# 顺序执行，单进程，IDB 只开一次
idalib-cli -d ./target.bin batch -- "meta" "segments" "decompile -a 0x401000"

# 一条命令，多个数据库（每个库一个子进程）；支持逗号列表或 glob
idalib-cli parallel -d "./a.i64,./b.i64" -- "functions -u"
idalib-cli parallel -d "./samples/*.i64" --jobs 4 -- "strings"
```

</details>

<details>
<summary><b>输出示例</b>（<code>decompile</code>）</summary>

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

面向 Agent 的工作流指南在 [`skills/`](skills/) 目录（对人类同样有用）。
可运行的端到端示例：[`examples/workflow.sh`](examples/workflow.sh)。

## 配置

可选配置文件 `~/.idapro/idalib-cli/config.toml`（基目录可用 `IDALIB_CLI_HOME`
覆盖）：

```toml
[defaults]
idadir = "/Applications/IDA Professional 9.1.app/Contents/MacOS"
idb_dir = "~/ida_out"      # IDB 默认存放位置（否则放在二进制同目录）
default_db = "~/idbs/main.i64"   # 省略 -d 时使用
save = true
auto_analyse = true
```

未显式给 `-d` 指向 IDB 时，IDB 默认生成在被分析二进制的同目录（`<binary>.i64`）。

## 架构说明

执行流：**加载配置 → 解析 `-d/--db` → 从磁盘打开 IDB（不存在则在二进制旁
创建）→ 执行命令 → 保存 → 退出**。

- **无状态**：无守护进程、无注册表、无会话记录。IDB 文件是状态的唯一权威，
  因此各数据库天然独立、跨调用持久、可安全并发。
- `parallel` 以进程为隔离单位（每个数据库一个子进程）— IDALib 不支持同进程
  多线程并发操作数据库。
- 输出契约：stdout 输出一个 JSON 文档；错误走 stderr，退出码非零。
- 已内置的上游问题规避：`EntryPointIter` 死循环修复（0.6.1）、字符串 NUL
  填充清理（保证 JSON 合法）、IDA 告别消息抑制。

## 版本与分支

工具版本 `x.y.z`；每个 minor 维护两个分支：

```
main             最新开发
v0.9_dev         开发分支（工具 0.9.x）
v0.9_release     稳定分支
v0.9.1 tag       发布点
```

适配新的 IDA 版本 = 升级 `idalib` 依赖 + 更新 `[package.metadata.ida]` +
开新的分支线（`v0.10_*`）。

## 分发说明

本仓库不含任何 Hex-Rays 代码。构建时链接 SDK 存根库（链接期空壳）；
运行时产物加载**用户本机**的 IDA 库并校验其许可证。
**严禁提交或再分发 IDA SDK 本体。**

## 许可证

本项目以 [Apache License 2.0](LICENSE) 分发。`Cargo.toml` 中的 `license` 字段
声明为 `MIT OR Apache-2.0`，以保持与 [idalib-rs](https://github.com/idalib-rs/idalib)
依赖的兼容；本仓库仅附带 Apache-2.0 许可文本。