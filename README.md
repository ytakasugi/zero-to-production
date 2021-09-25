# [zero-to-production](https://www.lpalmieri.com/posts/2020-05-24-zero-to-production-0-foreword/)

# Chapter3

## 3.Our First Endpoint: A Basic Health Check

ヘルスチェック・エンドポイントを実装して、まずはスタートしてみましょう。`/health_check`に対するGETリクエストを受信したら、Bodyボディのない200 OKレスポンスを返します。

HEALTH_CHECKを使って、アプリケーションが起動していて、リクエストを受け入れる準備ができているかどうかを確認することができます。これをpingdom.comのようなSaaSサービスと組み合わせれば、APIが暗転したときに警告を受けることができます。これは、副業として運営しているメールマガジンのベースラインとして非常に有効です。

ヘルスチェック・エンドポイントは、アプリケーションの管理にコンテナ・オーケストレーター（KubernetesやNomadなど）を使用している場合にも便利です。オーケストレーターは`/health_check`を呼び出して、APIが応答しなくなったことを検出し、再起動のトリガーとすることができます。

### 3.1.Wiring Up actix-web

```rust
use actix_web::{web, App, HttpRequest, HttpServer, Responder};

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(greet))
            .route("/{name}", web::get().to(greet))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
```

## 3.2.Anatomy of an `actix-web` application

では、先ほど`main.rs`ファイルにコピーペーストした内容を詳しく見てみましょう。

```rust
//! src/main.rs
// [...]

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(greet))
            .route("/{name}", web::get().to(greet))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
```

#### 3.2.1.Server - `HttpServer`

`HttpServer`は、アプリケーションを支えるバックボーンです。これは、次のようなことを行います。

- アプリケーションはどこでリクエストの着信を待つべきでしょうか？TCPソケット（例：127.0.0.1:8000）？Unixドメインソケットですか？
- 同時接続数の最大値を教えてください。単位時間あたりの新規接続数は？
- トランスポート・レベル・セキュリティ（TLS）を有効にすべきか？

などなど。

`HttpServer`は、言い換えれば、トランスポート・レベルの関心事をすべて処理します。
その後はどうなるのでしょうか？APIのクライアントとの新しい接続を確立し、クライアントのリクエストの処理を開始する必要があるとき、`HttpServer`は何をするのでしょうか？
そこで登場するのが`App`です。

#### 3.2.2.Application - `App`

`App`は、ルーティング、ミドルウェア、リクエストハンドラなど、すべてのアプリケーションロジックが存在する場所です。
`App`は、入力されたリクエストを受け取り、レスポンスを出力することを仕事とするコンポーネントです。
コードを見てみましょう。

```rust
App::new()
    .route("/", web::get().to(greet))
    .route("/{name}", web::get().to(greet))
```

Appはビルダーパターンの実用的な例です。`new()`は、流麗なAPIを使って新しい動作を少しずつ追加できるように、白紙の状態を与えてくれます（つまり、メソッド呼び出しを次々と連鎖させていきます）。
本書では、`App`のAPIサーフェスの大部分を必要に応じてカバーしていきます。旅の終わりには、ほとんどのメソッドに一度は触れているはずです。

#### 3.2.3.Endpoint - `Route`

アプリに新しいエンドポイントを追加するには？
Hello World!の例でも使用されているように、`routeメ`ソッドはおそらく最もシンプルな方法です。

routeは2つのパラメータを受け取ります。

- `path`: 文字列で、動的なパスセグメントに対応するためにテンプレート化されている場合があります（例：`"/{name}"`）。
- `route`: Route構造体のインスタンスです。

`Route `はハンドラとガードのセットを組み合わせたものです。
ガードは、リクエストが「マッチ」してハンドラに渡されるために満たすべき条件を指定します。実装上の観点からは、ガードは`Guard`トレイトの実装者です。`Guard::check`は魔法が起こるところです。

今回のスニペットでは

```rust
.route("/", web::get().to(greet))
```
`"/"`は、ベースパスに続くセグメントのないすべてのリクエストに一致します（例：`http://localhost：8000/`）。

`Web::get()はRoute::new()`のショートカットです。

`web::get()`は`Route::new().guard(guard::Get())`の短縮形であり、HTTPメソッドがGETの場合にのみリクエストがハンドラーに渡されます。

新しいリクエストが入ってきたときに何が起こるのか、イメージできるようになってきました。`App`は登録されているすべてのエンドポイントを、マッチするものが見つかるまで（パステンプレートとガードの両方が満たされるまで）繰り返し処理し、リクエストオブジェクトをハンドラーに渡します。
これは100%正確ではありませんが、とりあえずは十分なメンタルモデルです。

代わりにハンドラーはどのようなものでしょうか？その関数のシグネチャは何でしょうか？
今のところ、`greet`という1つの例しかありません。

```rust
async fn greet(req: HttpRequest) -> impl Responder {
    [...]
}
```

`greet`は、`HttpRequest`を入力とし、`Responder`tトレイトを実装した何らかの型を返す非同期の関数です。レスポンダ特性は、`HttpResponse`に変換できる型であれば実装されます。`Responder`トレイトは、様々な一般的な型(文字列、ステータスコード、バイト、`HttpResponse`など)に対して標準で実装されており、必要に応じて独自の実装を行うことができます。

すべてのハンドラーが`greet`という同じ関数シグネチャを持つ必要がありますか？
いいえ！`actix-web`は、禁断の特性を持つ黒魔術を使って、特に入力引数に関しては、ハンドラーの関数シグネチャを幅広く変えることができます。これについては、またすぐに説明します。

#### 3.2.4.Runtime - `actix-web`

`HttpServer`全体から`Route`へとドリルダウンしました。もう一度、main関数全体を見てみましょう。

```rust
//! src/main.rs
// [...]

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(greet))
            .route("/{name}", web::get().to(greet))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
```

ここで`#[actix_web::main]`は何をしているのでしょうか？さて、これを削除して何が起こるか見てみましょう！`cargo check`は次のようなエラーで我々に悲鳴を上げています。

```
error[E0277]: `main` has invalid return type `impl std::future::Future`
 --> src/main.rs:8:20
  |
8 | async fn main() -> std::io::Result<()> {
  |                    ^^^^^^^^^^^^^^^^^^^ `main` can only return types that implement `std::process::Termination`
  |
  = help: consider using `()`, or a `Result`

error[E0752]: `main` function is not allowed to be `async`
 --> src/main.rs:8:1
  |
8 | async fn main() -> std::io::Result<()> {
  | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ `main` function is not allowed to be `async`

error: aborting due to 2 previous errors
```

`HttpServer::run`は非同期メソッドなので、`main`は非同期である必要がありますが、バイナリのエントリーポイントである`main`は非同期関数にはできません。これはなぜでしょうか?

Rustの非同期プログラミングは`Future`の上に構築されています: `future`はまだ存在しないかもしれない値を表します。すべての`futures`は`poll`メソッドを公開しており、`future`が進行して最終的な値に解決するために呼ばれなければなりません。Rustの`futures`は怠惰だと考えることができます。ポーリングされない限り、完了まで実行される保証はありません。これは、他の言語で採用されているプッシュモデルと比較して、プルモデルと表現されることが多いです4。

Rustの標準ライブラリは、設計上、非同期ランタイムを含んでいません。`Cargo.toml`の`[dependencies]`の下にあるクレート(外部ライブラリ)を使って、依存関係としてプロジェクトに組み込むことになっています。このアプローチは、非常に汎用性が高く、ユーザーケースの特定の要求に応じて最適化された独自のランタイムを自由に実装することができます(`Fuchsiaプロジェクト`や`bastion`のアクターフレームワークを参照)。

これは、`main`が非同期関数ではない理由を説明しています。誰が`poll`を呼び出すのでしょうか？
Rustには、非同期ランタイムをRustコンパイラーに伝えるような特別な設定構文はありませんし(例：アロケータのように)、公平に見ても、ランタイムとは何か(例：`Executor`トレイト)についての標準的な定義すらありません。
そのため、メイン関数の先頭で非同期ランタイムを起動し、それを使ってフューチャーを完了させることが求められます。
ここまでで`#[actix_web::main]`の目的がわかったかもしれませんが、推測だけでは満足できません、私たちはそれを見たいのです。

どうやって？
`actix_web::main`は手続き型マクロで、Rust開発のためのスイスアーミーナイフに追加された素晴らしい機能である`cargo expand`を紹介する絶好の機会です。

```
cargo install cargo-expand
```

Rustのマクロは、トークンレベルで動作します。シンボルのストリーム（例えば、この例では`main`関数全体）を取り込み、新しいシンボルのストリームを出力して、コンパイラに渡します。言い換えれば、Rustのマクロの主な目的はコード生成です。
特定のマクロで何が起こっているのか、どうやってデバッグや検査をするのでしょうか？それは、マクロが出力するトークンを検査することです。

`cargo expand`は、出力をコンパイラに渡すことなく、コード内のすべてのマクロを展開するので、何が起こっているのかを段階的に理解することができます。
それでは cargo expand を使って`#[actix_web::main]`を紐解いてみましょう。

```
cargo expand
```

```rust
/// [...]

fn main() -> std::io::Result<()> {
   actix_web::rt::System::new("main").block_on(async move {
      {
         HttpServer::new(|| {
            App::new()
               .route("/", web::get().to(greet))
               .route("/{name}", web::get().to(greet))
         })
         .bind("127.0.0.1:8000")?
         .run()
         .await
      }
   })
}
```

`actix_web::main`が展開された後にRustコンパイラーに渡される`main`関数は確かに同期型であり、これが問題なくコンパイルされる理由です。
重要なのはこの行です。

```rust
actix_web::rt::System::new("main").block_on(async move { ... })
```

`actix`の非同期ランタイム(rt = runtime)を起動して、HttpServer::runが返すfutureを完了させるために使用しています。
言い換えれば、`#[actix_web::main]`の仕事は、非同期の`main`を定義できるような錯覚を与えることですが、ボンネットの中では、メインの非同期ボディを受け取り、それを`actix`のランタイムの上で走らせるために必要な定型文を書いているだけです。

`actix`のランタイムは`tokio`の上に構築されているので、アプリケーションを構築する際に`tokio`のエコシステム全体を活用することができます。

### 3.3.Implementing The Health Check Handler

`actix_web`のHello World! の例では、動く部品をすべて確認しました。`HttpServer`、`App`、`route`、`actix_web::main`です。
ヘルスチェックを期待通りに動作させるために、サンプルを修正するのに十分な知識があります。つまり、`/health_check`でGETリクエストを受け取ったときに、ボディのない`200 OK`レスポンスを返すということです。

もう一度、出発点を見てみましょう。

```rust
//! src/main.rs
use actix_web::{web, App, HttpRequest, HttpServer, Responder};

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(greet))
            .route("/{name}", web::get().to(greet))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
```

まず最初に、リクエストハンドラが必要です。`greet`関数を真似て、次のようなシグネチャから始めましょう。

```rust
async fn health_check(req: HttpRequest) -> impl Responder {
    todo!()
}
```

`Responder`は、`HttpResponse`への変換機能に過ぎないと言いました。つまり、`HttpResponse`のインスタンスを直接返すことができるのです。
ドキュメントを見ると、`HttpResponse::Ok`を使用して、`200`のステータスコードを持つ`HttpResponseBuilder`を取得することができます。`HttpResponseBuilder`は、`HttpResponse`レスポンスを段階的に構築するための豊富で流暢なAPIを公開していますが、ここでは必要ありません。ビルダーの`finish()`を呼び出すことで、空のボディを持つ`HttpResponse`を取得することができます。
すべてを統合します。

```rust
async fn health_check(req: HttpRequest) -> impl Responder {
    HttpResponse::Ok().finish()
}
```

次のステップはハンドラーの登録で、`route`経由で`App`に追加する必要があります。

```rust
App::new()
    .route("/health_check", web::get().to(health_check))
```

全体像を見てみよう

```rust
//! src/main.rs

use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};

async fn health_check(req: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/health_check", web::get().to(health_check)))
        .bind("127.0.0.1:8000")?
        .run()
        .await
}
```

`cargo check`はスムーズに行われますが、1つ注意点があります。

```
warning: unused variable: `req`
 --> src/main.rs:3:23
  |
3 | async fn health_check(req: HttpRequest) -> impl Responder {
  |                       ^^^ help: if this is intentional, prefix it with an underscore: `_req`
  |
  = note: `#[warn(unused_variables)]` on by default
```

ヘルスチェックのレスポンスは確かに静的で、受信したHTTPリクエストにバンドルされているデータは一切使用しません（ルーティングは別として）。コンパイラのアドバイスに従って`req`の前にアンダースコアを付けることもできますし、`health_check`からその入力引数を完全に削除することもできます。

```rust
async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}
```

驚いたことに、コンパイルできました！ `actix-web`は舞台裏でかなり高度な型のマジックが行われていて、リクエスト・ハンドラーとして幅広いシグネチャを受け入れます。

あとは何をすればいいのでしょう？
さて、ちょっとしたテストです。

```
# Launch the application first in another terminal with `cargo run`
curl -v http://127.0.0.1:8000/health_check
```

```
*   Trying 127.0.0.1...
* TCP_NODELAY set
* Connected to localhost (127.0.0.1) port 8000 (#0)
> GET /health_check HTTP/1.1
> Host: localhost:8000
> User-Agent: curl/7.61.0
> Accept: */*
>
< HTTP/1.1 200 OK
< content-length: 0
< date: Wed, 05 Aug 2020 22:11:52 GMT
```

おめでとうございます。これで初めてactix_webエンドポイントが動作するようになりました。


