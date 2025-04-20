# base64

入力データを base64 形式にエンコード、デコードするツールです。

## 使い方

パイプで入力データを渡して、リダイレクトでファイルに保存します。

### エンコード

```
$ cat <file> | ./target/release/base64 encode > encoded
```

### デコード

```
$ cat <file> | ./target/release/base64 decode > decoded
```
