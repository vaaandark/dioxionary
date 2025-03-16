# dioxionary

[![dependency status](https://deps.rs/repo/github/vaaandark/dioxionary/status.svg)](https://deps.rs/repo/github/vaaandark/dioxionary)
[![Build Status](https://github.com/vaaandark/dioxionary/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/vaaandark/dioxionary/actions/workflows/rust.yml)

[简体中文](README.md) | [English](README-en.md)

使用 **离线** / **在线** 词典在终端中查单词、背单词！

## 依赖

- openssl

> 如果使用 musl 构建则没有 openssl 依赖

## 安装

### 下载预构建二进制

推荐从右侧的 [Github Release](https://github.com/vaaandark/dioxionary/releases) 下载对应平台的二进制文件。

也可以到 [GitHub Actions](https://github.com/vaaandark/dioxionary/actions?query=workflow%3A%22CI+build%22+actor%3Avaaandark+branch%3Amaster+event%3Apush+is%3Asuccess) 下载最新构建的二进制，包含多种版本。

### 自行编译

```console
cargo install dioxionary
```

如果想开启朗读功能，则使用：

```console
cargo install dioxionary --features pronunciation
```

## 使用

![demo](images/demo.gif)

### 启用参数补全

```console
$ eval "$(dioxionary -c bash)" # for bash
$ eval "$(dioxionary -c zsh)"  # for zsh
$ eval "$(dioxionary -c fish)" # for fish
```

可以将上述命令直接写到 shell 的配置文件中。

### 查询单词

```console
$ dioxionary lookup [OPTIONS] [WORD]
```

子命令 `lookup` 可以省略：

```console
$ dioxionary [OPTIONS] [WORD]
```

当参数中没有待查单词时，将进入交互模式，可以无限查询，直至按下 `Ctrl+D` 。

支持并默认使用模糊搜索(fuzzy search)，在词典中没有找到单词时会输出最相似的一个或多个单词的释义。

使用 `-e` 或者 `--exact-search` 可以关闭模糊搜索。也可以通过在单词前添加 `/` 或者 `|` 来打开或关闭模糊搜索，在单词前添加 `@` 使用网络词典。

```console
$ dioxionary /terraria   # 模糊搜索
$ dioxionary '|terraria' # 非模糊搜索，注意使用引号
$ dioxionary @terraria   # 使用网络词典
```

默认使用本地词典，本地词典目录应当存放在：

|Platform | Value                                             | Example                                        |
| ------- | ------------------------------------------------- | ---------------------------------------------- |
| Linux   | `$XDG_CONFIG_HOME/dioxionary` or `$HOME`/.config/dioxionary | /home/alice/.config/dioxionary                      |
| macOS   | `$HOME`/Library/Application Support/dioxionary         | /Users/Alice/Library/Application Support/dioxionary |
| Windows | `{FOLDERID_RoamingAppData}`/dioxionary                 | C:\Users\Alice\AppData\Roaming/dioxionary           |

> 只支持 stardict 的词典格式

> 可以在 http://download.huzheng.org/ 下载 stardict 格式词典

```plain
~/.config/dioxionary
├── 00-cdict-gb
├── 01-kdic-computer-gb
├── 02-langdao-ec-gb
├── 03-oxford-gb
└── 04-powerword2011_1_900

    00-cdict-gb
    ├── cdict-gb.dict
    ├── cdict-gb.dict.dz
    ├── cdict-gb.idx
    └── cdict-gb.ifo
```

使用 `-x` 选项会使用在线词典查询：

```console
$ dioxionary -x <DICTDIR> <WORD>
```

可以使用 `-l` 或 `--local` 选项指定词典文件路径。

使用 `-L` 或 `--local-first` 选项则会在本地查询失败后使用网络词典。推荐在 shell 配置文件中加入 `alias rl='dioxionary -l'`。

使用 `-r` 或者输入单词时前缀包含 `~` 可以朗读单词。

> 前提是构建时打开了 pronunciation 特性。

### 多字典支持

如上文示例中，可以将词典目录分别命名为 `00-XXX`, `01-YYY`, ..., `99-ZZZ` 这样的格式来实现优先级。

### 列出记录

> 注意：只有在线查词时会查得并记录单词类型

```console
$ dioxionary list [OPTIONS] [TYPE]
```

以下为支持的单词类型：

CET4 | CET6 | TOEFL | IELTS | GMAT | GRE | SAT
--- | --- | --- | --- | --- | --- | ---

缺少类型时列出所有记录。

### 统计数据

统计查询过的各类单词的个数：

```console
$ dioxionary count
```
