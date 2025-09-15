# `lindera-tantivy` を使った日本語全文検索サンプル

[`lindera-tantivy`](https://github.com/lindera/lindera-tantivy)を使った日本語の全文検索を実行するサンプルです。   
作者の方によると他のトークナイザーがtantivyで使えなくなったので`lindera-tantivy`を作成したとのこと。  
感謝ですね。　

## 使い方

### 1. ビルド

Rustのビルド環境がなくてもDockerがあればビルドできます。  
アーキティクチャは自分の環境に合わせて変更してください。

```sh
docker run --rm -it --mount type=bind,source="$(pwd)",target=/project -w /project messense/rust-musl-cross:aarch64-musl cargo build --release
```

### 2. 実行

ビルドしたバイナリには全てのライブラリが含まれています。  
ですので、ほどんどの環境(例えshellがなくても)で実行できます。  
下記は、`distroless`という非常に小さいコンテナで実行する例です。
```
docker run --rm -it --mount type=bind,source="$(pwd)",target=/project -w /project gcr.io/distroless/static-debian12 /project/target/aarch64-unknown-linux-musl/release/full-search-jp
```


辞書ファイルについてはライセンスがよく解らなかったので成果物から削除しました。   

辞書の作成方法は、[```lindera-cli```](https://crates.io/crates/lindera-cli)のcrates.ioに記載があります。

これを元にdockerで作成する方法は[こちら](https://github.com/2bitcpu/documents/tree/main/container/make-linedra-dic)にまとめておきました。

