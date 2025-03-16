# dioxionary

[![dependency status](https://deps.rs/repo/github/vaaandark/dioxionary/status.svg)](https://deps.rs/repo/github/vaaandark/dioxionary)
[![Build Status](https://github.com/vaaandark/dioxionary/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/vaaandark/dioxionary/actions/workflows/rust.yml)

[简体中文](README.md) | [English](README-en.md)

Look up and memorize all words in terminal **offline** / **online**!

## Prerequisites

- openssl

> If using musl build, there is no openssl dependency.

## Installation

### Download Prebuilt Binaries

It is recommended to download the prebuilt binary file for your platform from the [Github Release](https://github.com/vaaandark/dioxionary/releases) on the right side.

Alternatively, you can also download the latest build binaries, including many versions, from the [GitHub Actions](https://github.com/vaaandark/dioxionary/actions?query=workflow%3A%22CI+build%22+actor%3Avaaandark+branch%3Amaster+event%3Apush+is%3Asuccess).

### Compile from source

```console
cargo install dioxionary
```

If you want to enable the pronunciation feature, use:

```console
cargo install dioxionary --features pronunciation
```

## Usage

![demo](images/demo.gif)

### Enable argument completion

```console
$ eval "$(dioxionary completion bash)" # for bash
$ eval "$(dioxionary completion zsh)"  # for zsh
$ eval "$(dioxionary completion fish)" # for fish
```

You can write the above commands directly into the configuration file of your shell.

### Look up word meaning

```console
$ dioxionary lookup [OPTIONS] [WORD]
```

The subcommand `lookup` can be omitted:

```console
$ dioxionary [OPTIONS] [WORD]
```

When there is no word to be searched in the parameter, it will enter the interactive mode, and can search infinitely until `Ctrl+D` is pressed.

Supports and uses fuzzy search by default. When no word is found in the dictionary, it will output the most similar definition of one or more words.

Use `-e` or `--exact-search` to turn off fuzzy search. You can also turn fuzzy search on or off by prefixing a word with `/` or `|`, and use web dictionaries with `@` before a word.

```console
$ dioxionary /terraria   # Fuzzy search
$ dioxionary '|terraria' # Non-fuzzy search, pay attention to use quotation marks
$ dioxionary @terraria   # Online search
```

The local dictionary is used by default, and the local dictionary directory should be stored in:

|Platform | Value                                             | Example                                        |
| ------- | ------------------------------------------------- | ---------------------------------------------- |
| Linux   | `$XDG_CONFIG_HOME/dioxionary` or `$HOME`/.config/dioxionary | /home/alice/.config/dioxionary                      |
| macOS   | `$HOME`/Library/Application Support/dioxionary         | /Users/Alice/Library/Application Support/dioxionary |
| Windows | `{FOLDERID_RoamingAppData}`/dioxionary                 | C:\Users\Alice\AppData\Roaming/dioxionary           |

> Only stardict dictionary format is supported

> You can download dictionaries in stardict format at http://download.huzheng.org/

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

Using the `-x` option will use an online dictionary lookup:

```console
$ dioxionary -x <DICTDIR> <WORD>
```

The dictionary file path can be specified with the `-l` or `--local` option.

Use the `-L` or `--local-first` option to use the network dictionary after a local lookup fails. It is recommended to add `alias rl='dioxionary -L'` in the shell configuration file.

Use -r or prefix the word with ~ to pronounce the word.

> Prerequisite: The pronunciation feature was enabled during build.

### Multiple dictionary support

As in the above example, the dictionary directories can be named in the format of `00-XXX`, `01-YYY`, ..., `99-ZZZ` to achieve priority.

### List records

> Note: Only the word type will be searched and recorded when searching online

```console
$ dioxionary list [OPTIONS] [TYPE]
```

The following word types are supported:

CET4 | CET6 | TOEFL | IELTS | GMAT | GRE | SAT
--- | --- | --- | --- | --- | --- | ---

List all records when type is missing.

### Statistical data

Count the number of various words that have been queried:

```console
$ dioxionary count
```
