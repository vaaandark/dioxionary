# rmall

Remember all words in terminal!

在终端中查单词、背单词！

## 已完成

- [x] 查单词
- [ ] 背单词

## 依赖

- sqlite3

## 安装

### 从源码安装

```console
make && make install
```

### 使用预构建版本

推荐在 [Github Release](https://github.com/vaaandark/rmall/releases) 下载预构建二进制文件。

## 使用方法

```bash
rmall [WORD]
# or show history
rmall -l
```

例如：

```console
% rmall rust
rust
/ rʌst / / rʌst /
锈，铁锈；（植物的）锈病，锈菌；铁锈色，赭色
（使）生锈；成铁锈色；（因疏忽或不用而）衰退，变迟钝；（因长时间不用或有害使用而）损害，腐蚀
【名】 （Rust）（英）拉斯特，（德、捷、瑞典）鲁斯特，（法）吕斯特（人名）
<CET6> <考研> <TOEFL> <SAT>
% rmall 铁锈
rust
锈，铁锈；（植物的）锈病，锈菌；铁锈色，赭色；（使）生锈；成铁锈色；（因疏忽或不用而）衰退，变迟钝；（因长时间不用或有害使用而）损害，腐蚀；【名】 （Rust）（英）拉斯特，（德、捷、瑞典）鲁斯特，（法）吕斯特（人名）；
corrosion
腐蚀，侵蚀；腐蚀产生的物质；
```