# `lindera-tantivy` を使った日本語全文検索サンプル

`tantivy` と `lindera` を使い、コマンドラインで日本語の全文検索を実行するサンプルです。

## 使い方

### 1. ビルド

Rustのビルド環境がなくてもDockerがあればビルドできます。

```sh
docker run --rm -it --mount type=bind,source="$(pwd)",target=/project -w /project messense/rust-musl-cross:aarch64-musl cargo build --release
```

### ビルドしたバイナリには全てのライブラリが含まれているのでほどんどの環境(例えshellが実行できなくても)で実行できます。
```
docker run --rm -it --mount type=bind,source="$(pwd)",target=/project -w /project gcr.io/distroless/static-debian12 /project/target/aarch64-unknown-linux-musl/release/full-search-jp
```