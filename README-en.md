# rmall

[简体中文](README.md) | [English](README-en.md)

Remember all words in terminal **offline** / **online**!

## Prerequisites

- sqlite3
- openssl

## Installation

### Compile from source

```console
cargo install --git https://github.com/vaaandark/rmall
```

### Prebuilt binary

You can download binaries at [Github Release](https://github.com/vaaandark/rmall/releases)

## Usage

![demo](images/demo.svg)

> Characters are slightly misplaced after asciinema recording and svg-term conversion

### Look up word meaning

```console
$ rmall lookup [OPTIONS] [WORD]
```

The subcommand `lookup` can be omitted:

```console
$ rmall [OPTIONS] [WORD]
```

When there is no word to be searched in the parameter, it will enter the interactive mode, and can search infinitely until `Ctrl+D` is pressed.

Supports and uses fuzzy search by default. When no word is found in the dictionary, it will output the most similar definition of one or more words.

Use `-e` or `--exact-search` to turn off fuzzy search. You can also turn fuzzy search on or off by prefixing a word with `/` or `|`, and use web dictionaries with `@` before a word.

```console
$ rmall /terraria   # Fuzzy search
$ rmall '|terraria' # Non-fuzzy search, pay attention to use quotation marks
$ rmall @terraria   # Online search
```

The local dictionary is used by default, and the local dictionary directory should be stored in:

|Platform | Value                                             | Example                                        |
| ------- | ------------------------------------------------- | ---------------------------------------------- |
| Linux   | `$XDG_CONFIG_HOME/rmall` or `$HOME`/.config/rmall | /home/alice/.config/rmall                      |
| macOS   | `$HOME`/Library/Application Support/rmall         | /Users/Alice/Library/Application Support/rmall |
| Windows | `{FOLDERID_RoamingAppData}`/rmall                 | C:\Users\Alice\AppData\Roaming/rmall           |

> Only stardict dictionary format is supported

> You can download dictionaries in stardict format at http://download.huzheng.org/

```plain
~/.config/rmall
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
$ rmall -x <DICTDIR> <WORD>
```

The dictionary file path can be specified with the `-l` or `--local` option.

Use the `-L` or `--local-first` option to use the network dictionary after a local lookup fails. It is recommended to add `alias rl='rmall -l'` in the shell configuration file.

### Multiple dictionary support

As in the above example, the dictionary directories can be named in the format of `00-XXX`, `01-YYY`, ..., `99-ZZZ` to achieve priority.

### List records

> Note: Only the word type will be searched and recorded when searching online

```console
$ rmall list [OPTIONS] [TYPE]
```

The following word types are supported:

CET4 | CET6 | CET8 | TOEFL | IELTS | GMAT | GRE | SAT
--- | --- | --- | --- | --- | --- | --- | ---

List all records when type is missing.

### Statistical data

Count the number of various words that have been queried:

```console
$ rmall count
```
