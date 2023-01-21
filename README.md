# rmall

Remember all words in terminal!

在终端中查单词、背单词！

## 已完成

- [x] 查单词
- [ ] 背单词

## 依赖

- sqlite3

## 安装

### 自行编译

```console
cargo build --releases
```

### 下载预构建二进制

推荐在 [Github Release](https://github.com/vaaandark/rmall/releases) 下载预构建二进制文件。

## 使用

![demo](images/demo.svg)

### 查询单词

```console
$ rmall lookup <WORD>
```

### 列出记录

```console
$ rmall list [TYPE]
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
