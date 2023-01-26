# rmall

Remember all words in terminal **offline** / **online**!

使用 **离线** / **在线** 词典在终端中查单词、背单词！

## 依赖

- sqlite3

## 安装

### 自行编译

```console
cargo install --git https://github.com/vaaandark/rmall
```

### 下载预构建二进制

推荐在 [Github Release](https://github.com/vaaandark/rmall/releases) 下载预构建二进制文件。

## 使用

![demo](images/demo.svg)

### 查询单词

```console
$ rmall lookup [OPTIONS] <WORD>
```

如果需要本地词典，则可以使用 `-l` 或 `--local` 选项指定词典文件目录：

> 只支持 stardict 的词典格式

> 可以在 http://download.huzheng.org/ 下载 stardict 格式词典

```plain
    cdict-gb
    ├── cdict-gb.dict
    ├── cdict-gb.dict.dz
    ├── cdict-gb.idx
    └── cdict-gb.ifo

Their prefixes must be the same as the dirname.
```

在本地词典查询：

```console
$ lookup -l <DICTDIR> <WORD>
```

使用 `-L` 选项则会在本地查询失败后使用网络词典。

### 列出记录

```console
$ rmall list [OPTIONS] [TYPE]
```

以下为支持的单词类型：

CET4 | CET6 | CET8 | TOEFL | IELTS | GMAT | GRE | SAT
--- | --- | --- | --- | --- | --- | --- | ---

缺少类型时列出所有记录。

### 统计数据

统计查询过的各类单词的个数：

```console
$ rmall count
```

### 多字典支持

暂时没有在程序中集成多字典，但是可以通过 Shell 脚本实现类似的效果：

```console
$ trans terraria
Error: WordNotFound("朗道英汉字典5.0")
Error: WordNotFound("牛津现代英汉双解词典")
Error: WordNotFound("CDICT5英汉辞典")
Error: WordNotFound("计算机词汇")
terraria
英 / tɛˈrɛːrɪə / 美 / tɛˈrɛːrɪə /
 泰拉瑞亚（游戏名）
  Xbox Live ArcadeMarch 27, 2013PlayStation VitaLate Fall 2013iOSAugust 29, 2013http://www.terrariaonline.
```

脚本在 `scripts/` 目录。
