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

### 3.2.Anatomy of an `actix-web` application

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

## 4.Our First Integration Test

`/health_check`が最初のエンドポイントで、アプリケーションを起動して`curl`で手動でテストすることで、すべてが期待通りに動作していることを確認しました。

アプリケーションの規模が大きくなると、変更を加えるたびに、アプリケーションの動作に関するすべての前提条件が有効であるかどうかを手動でチェックするのは、ますますコストがかかります。
可能な限り自動化したいと考えています。リグレッションを防ぐために、変更をコミットするたびにCIパイプラインでこれらのチェックを実行する必要があります。

私たちのヘルスチェックの動作は、私たちの旅の過程であまり進化しないかもしれませんが、テストの足場を適切に設定するための良い出発点となります。

### 4.1.How Do You Test An Endpoint?

APIとは、ある目的を達成するための手段であり、ある種のタスク（ドキュメントの保存、電子メールの発行など）を実行するために外界に公開されるツールである。
APIで公開するエンドポイントは、私たちとクライアントの間の契約を定義します。つまり、システムの入力と出力、そのインターフェースに関する共有の合意です。

コントラクトは時間の経過とともに変化する可能性があり、大まかに2つのシナリオを想定することができます。

- 下位互換性のある変更（例：新しいエンドポイントの追加）。
- 下位互換性のある変更（例：新しいエンドポイントの追加）と、破壊的な変更（例：エンドポイントの削除や出力のスキーマからのフィールドの削除）。

1つ目のケースでは、既存のAPIクライアントはそのまま動作します。
2つ目のケースでは、契約の違反部分に依存していた場合、既存の統合が壊れる可能性があります。

意図的にAPIコントラクトに違反する変更を加えることはあっても、誤って違反しないようにすることは非常に重要です。

ユーザーに見えるリグレッションを導入していないことを確認する最も確実な方法は何でしょうか？
ユーザーとまったく同じ方法でAPIを操作してテストすることです。つまり、APIに対してHTTPリクエストを実行し、受け取ったレスポンスで仮定を検証します。

これは、ブラックボックステストと呼ばれています。システムの内部実装の詳細にアクセスすることなく、一連の入力に対する出力を調べることで、システムの動作を検証します。

この原則に従うと、ハンドラ関数を直接呼び出すテストでは満足できません。

```rust
#[cfg(test)]
mod tests {
    use crate::health_check;

    #[actix_rt::test]
    async fn health_check_succeeds() {
        let response = health_check().await;
        // This requires changing the return type of `health_check`
        // from `impl Responder` to `HttpResponse` to compile
        assert!(response.status().is_success())
    }
}
```

ハンドラがGETリクエストで起動されることを確認していません。
ハンドラが`/health_check`をパスとして起動されることをチェックしていません。

これらの2つのプロパティのいずれかを変更すると、API契約が破棄されますが、テストはまだ通過します。

`actix-web`は、ルーティング・ロジックをスキップせずにアプリと対話するための便利な機能を提供していますが、そのアプローチには重大な欠点があります。

他のウェブフレームワークに移行すると、統合テストスイート全体を書き直さなければなりません。可能な限り、統合テストはAPIの実装を支える技術から高度に切り離されたものにしたいと考えています（例えば、フレームワークに依存しない統合テストを持つことは、大規模な書き換えやリファクタリングを行う際に命を救うことになります！）。
actix-webの制限により、アプリの起動ロジックを本番コードとテストコードで共有することができず、時間の経過とともに乖離が発生するリスクがあるため、テストスイートが提供する保証に対する信頼性が損なわれる。
私たちは、完全なブラックボックス・ソリューションを選択します。各テストの開始時にアプリを起動し、市販のHTTPクライアント（例：reqwest）を使用して対話します。

### 4.2.Where Should I Put My Tests?

Rustでは、テストを書く際に3つの選択肢があります。

- 埋め込みテストモジュールのコードの横に表示

```rust
// Some code I want to test

#[cfg(test)]
mod tests {
    // Import the code I want to test
    use super::*;
    
    // My tests
}
```

- テストコードを`tests`フォルダに入れる

```
> ls

src/
tests/
Cargo.toml
Cargo.lock
```

- テストコードを公開ドキュメント（docテスト）の一部として使用することができます。

```rust
/// Check if a number is even.
/// ```rust
/// use zero2prod::is_even;
/// 
/// assert!(is_even(2));
/// assert!(!is_even(1));
/// ```
pub fn is_even(x: u64) -> bool {
    x % 2 == 0
}
```

違いは何ですか？
埋め込みテストモジュールはプロジェクトの一部であり、設定条件のチェックである #[cfg(test)] の後ろに隠れています。`tests`フォルダ以下のものやドキュメントのテストは、それぞれ別のバイナリとしてコンパイルされます。
このことは、可視性のルールに関しても影響を及ぼします。

埋め込まれたテストモジュールは、その隣にあるコードに特権的にアクセスすることができます。構造体、メソッド、フィールド、関数を操作することができますが、これらは`public`としてマークされておらず、通常は私たちのコードのユーザーが自分のプロジェクトの依存関係としてインポートしても利用することはできません。
埋め込みテストモジュールは、私が「氷山プロジェクト」と呼んでいるものに非常に役立ちます。つまり、公開されている表面は非常に限られていますが（例：いくつかのパブリック関数）、基本的な機械ははるかに大きく、かなり複雑です（例：数十個のルーチン）。公開されている関数を使ってすべての可能なエッジケースを実行するのは簡単ではないかもしれませんが、埋め込みテストモジュールを活用してプライベートなサブコンポーネントのユニットテストを書くことで、プロジェクト全体の正しさに対する全体的な信頼性を高めることができます。

一方、外部の`tests`フォルダやドキュメントテストにあるテストは、他のプロジェクトに`crate`を依存関係として追加した場合に得られるのとまったく同じレベルで、コードにアクセスすることができます。そのため、これらのテストは主に統合テストに使用されます。つまり、ユーザーと同じ方法でコードを呼び出してテストするのです。

私たちのメールマガジンはライブラリではないので、境界線は少し曖昧です。Rustのクレートとして世界に公開しているわけではなく、ネットワーク経由でアクセスできるAPIとして公開しています。
しかし、私たちはAPI統合テストのために`tests`フォルダを使用するつもりです。これはより明確に分離されており、テストヘルパーを外部テストバイナリのサブモジュールとして管理するのがより簡単だからです。

### 4.3.Changing Our Project Structure For Easier Testing

実際に最初のテストを`/tests`の下に書く前に、ちょっとした整理をしておきましょう。
先ほど述べたように、`tests`以下のものはすべて独自のバイナリにコンパイルされます。つまり、テスト対象のコードはすべて crate としてインポートされます。しかし、私たちのプロジェクトは、現時点ではバイナリです。そのため、今のままではメインの関数をテストでインポートすることができません。

私の言葉を信じてもらえないのであれば、簡単な実験をしてみましょう。

```
mkdir -p tests
```

`tests/health_check.rs`ファイルを作成します。

```
//! tests/health_check.rs

use zero2prod::main;

#[test]
fn dummy_test() {
    main()
}
```

`cargo test`は、以下のような内容で失敗するはずです。

```
error[E0432]: unresolved import `zero2prod`
 --> tests/health_check.rs:1:5
  |
1 | use zero2prod::main;
  |     ^^^^^^^^^ use of undeclared type or module `zero2prod`

error: aborting due to previous error

For more information about this error, try `rustc --explain E0432`.
error: could not compile `zero2prod`.
```

プロジェクトをライブラリとバイナリにリファクタリングする必要があります。すべてのロジックはライブラリクレートに格納され、バイナリ自体は非常にスリムなメイン関数を持つ単なるエントリーポイントになります。
まず最初に、`Cargo.toml`を変更します。
現在は次のようになっています。

```toml
[package]
name = "zero2prod"
version = "0.1.0"
authors = ["Luca Palmieri <contact@lpalmieri.com>"]
edition = "2018"

[dependencies]
# [...]
```

ここでは、`cargo`のデフォルトの動作に依存しています。つまり、何も明記されていなければ、`src/main.rs`ファイルをバイナリのエントリーポイントとして探し、`package.name`フィールドをバイナリ名として使用します。
[マニフェストのターゲット仕様](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#cargo-targets)を見ると、プロジェクトにライブラリを追加するために`lib`セクションを追加する必要があります。

```toml
[package]
name = "zero2prod"
version = "0.1.0"
authors = ["Luca Palmieri <contact@lpalmieri.com>"]
edition = "2018"

[lib]
# We could use any path here, but we are following the community convention
# We could specify a library name using the `name` field. If unspecified,
# cargo will default to `package.name`, which is what we want.
path = "src/lib.rs"

[dependencies]
# [...]
```

`lib.rs`ファイルはまだ存在しませんし、`cargo`が作成してくれるわけでもありません。

```
cargo check
```

```
error: couldn't read src/lib.rs: No such file or directory (os error 2)

error: aborting due to previous error

error: could not compile `zero2prod`
```

空の`src/lib.rs`を作成しましょう。

```
touch src/lib.rs
```

これですべてがうまくいくはずです。`cargo check`が通過し、`cargo run`でアプリケーションが起動します。
このように動作していますが、`Cargo.toml`ファイルは一見して全体像を示していません。ライブラリは見えますが、そこに私たちのバイナリはありません。厳密に必要ではないにしても、自動生成されたバニラ構成から抜け出すときには、すべてが明示されているほうがいいですね。

```toml
[package]
name = "zero2prod"
version = "0.1.0"
authors = ["Luca Palmieri <contact@lpalmieri.com>"]
edition = "2018"

[lib]
path = "src/lib.rs"

# Notice the double square brackets: it's an array in TOML's syntax.
# We can only have one library in a project, but we can have multiple binaries!
# If you want to manage multiple libraries in the same repository
# have a look at the workspace feature - we'll cover it later on.
[[bin]]
path = "src/main.rs"
name = "zero2prod"

[dependencies]
# [...]
```

すっきりしたので、次に進みましょう。
とりあえず、`main.rs`の関数をそのまま`lib.rs`に移動させます（衝突を避けるために`run`という名前にしています）。

```rust
//! main.rs

use zero2prod::run;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    run().await
}
```

```rust
//! lib.rs

use actix_web::{web, App, HttpResponse, HttpServer};

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

// We need to mark `run` as public.
// It is no longer a binary entrypoint, therefore we can mark it as async
// without having to use any proc-macro incantation.
pub async fn run() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/health_check", web::get().to(health_check)))
        .bind("127.0.0.1:8000")?
        .run()
        .await
}
```

さてさて、肝心の統合テストを書く準備ができました。

### 4.4. Implementing Our First Integration Test

ヘルスチェックのエンドポイントの仕様は以下の通りです。

> `health_checl`に対するGETリクエストを受信すると、ボディのない`200 OK`レスポンスを返します。

それをテストに変換して、できるだけ多くのことを埋めていきましょう。

```rust
//! tests/health_check.rs

// `actix_rt::test` is the testing equivalent of `actix_web::main`.
// It also spares you from having to specify the `#[test]` attribute.
//
// Use `cargo add actix-rt --dev --vers 2` to add `actix-rt`
// under `[dev-dependencies]` in Cargo.toml
//
// You can inspect what code gets generated using 
// `cargo expand --test health_check` (<- name of the test file)
#[actix_rt::test]
async fn health_check_works() {
    // Arrange
    spawn_app().await.expect("Failed to spawn our app.");
    // We need to bring in `reqwest` 
    // to perform HTTP requests against our application.
    //
    // Use `cargo add reqwest --dev --vers 0.11` to add
    // it under `[dev-dependencies]` in Cargo.toml
    let client = reqwest::Client::new();

    // Act
    let response = client
            .get("http://127.0.0.1:8000/health_check")
            .send()
            .await
            .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

// Launch our application in the background ~somehow~
async fn spawn_app() -> std::io::Result<()> {
    todo!()
}
```

このテストケースをよく見てみましょう。
`spawn_app`は、アプリケーション コードに依存する唯一の部分です。
明日、Rustを捨ててRuby on Railsでアプリケーションを書き直すことにしても、`spawn_app`を適切なトリガー (Railsアプリを起動するbashコマンドなど) に置き換えれば、同じテストスイートを使って新しいスタックのリグレッションをチェックすることができます。

このテストは、私たちがチェックしたいと考えているプロパティをすべて網羅しています。

- ヘルスチェックは/health_checkで公開されています。
- ヘルスチェックはGETメソッドの背後にある。
- ヘルスチェックは常に200を返す。
- ヘルスチェックのレスポンスにはボディがない。

これに合格すれば完了です。

統合テストのパズルの最後のピースであるspawn_appがないのです。
ここで`run`を呼び出してはどうでしょうか？つまり、次のようになります。

```rust
//! tests/health_check.rs
// [...]

async fn spawn_app() -> std::io::Result<()> {
    zero2prod::run().await
}
```

ぜひ試してみてください。

```rust
cargo test
```

```
     Running target/debug/deps/health_check-fc74836458377166

running 1 test
test health_check_works ... test health_check_works has been running for over 60 seconds
```

いくら待ってもテスト実行が終了しません。何が起こっているのでしょうか？

`zero2prod::run`では、`HttpServer::run`を呼び出しています（そして`await`しています）。`HttpServer::run`は`Server`のインスタンスを返します。`.await`を呼び出すと、指定したアドレスで無期限に待ち受けを開始します。到着したリクエストを処理しますが、自分でシャットダウンしたり「完了」することはありません。
つまり、`spawn_app`が戻ることはなく、テストロジックが実行されることはないということです。

そこで、アプリケーションをバックグラウンド・タスクとして実行する必要があります。
`tokio::spawn`はここで非常に便利です。`tokio::spawn`は未来を受け取り、その完了を待たずにランタイムにポーリングのために渡します。そのため、下流の未来やタスク（テストロジックなど）と同時に実行されます。

`zero2prod::run`をリファクタリングして、サーバーを待たずに返すようにしましょう。

```rust
//! src/lib.rs

use actix_web::{web, App, HttpResponse, HttpServer};
use actix_web::dev::Server;

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

// Notice the different signature!
// We return `Server` on the happy path and we dropped the `async` keyword
// We have no .await call, so it is not needed anymore.
pub fn run() -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| App::new().route("/health_check", web::get().to(health_check)))
        .bind("127.0.0.1:8000")?
        .run();
    // No .await here!
    Ok(server)
}
```

それに合わせて、`main.rs`を修正する必要があります。

```rust
//! src/main.rs

use zero2prod::run;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Bubble up the io::Error if we failed to bind the address
    // Otherwise call .await on our Server
    run()?.await
}
```

`cargo check`してみると、何も問題がないことがわかります。
これで`spawn_app`を書くことができます。

```rust
//! tests/health_check.rs
// [...]

// No .await call, therefore no need for `spawn_app` to be async now.
// We are also running tests, so it is not worth it to propagate errors:
// if we fail to perform the required setup we can just panic and crash
// all the things.
fn spawn_app() {
    let server = zero2prod::run().expect("Failed to bind address");
    // Launch the server as a background task
    // tokio::spawn returns a handle to the spawned future,
    // but we have no use for it here, hence the non-binding let
    //
    // New dev dependency - let's add tokio to the party with
    // `cargo add tokio --dev --vers 1`
    let _ = tokio::spawn(server);
}
```

`spawn_app`のシグネチャーの変更に対応するためのテストの迅速な調整

```rust
//! tests/health_check.rs
// [...]

#[actix_rt::test]
async fn health_check_works() {
    // No .await, no .expect
    spawn_app();
    // [...]
}
```

`cargo test`を実行してみましょう

```
     Running target/debug/deps/health_check-a1d027e9ac92cd64

running 1 test
test health_check_works ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

最初の統合テストが成功しました。

### 4.5.Polishing

このようにして動作するようになったのですから、あとはもう一度見直して、必要ならば、あるいは可能ならば改善する必要があります。

#### 4.5.1.Clean Up

テストが終了すると、バックグラウンドで動作しているアプリはどうなりますか？シャットダウンされますか？どこかにゾンビとして残っているのでしょうか？

これは、8000番台のポートがテスト終了時に解放され、アプリケーションが正しくシャットダウンされたことを示唆しています。
`actix_rt::test`は、各テストケースの最初に新しいランタイムを起動し、各テストケースの最後にシャットダウンします。
言い換えれば、良いニュースです。テスト実行の間にリソースが漏れるのを防ぐために、クリーンアップ・ロジックを実装する必要はありません。

#### 4.5.2.Choosing A Random Port

`spawn_app`は常にポート8000でアプリケーションを実行しようとしますが、これは理想的ではありません。

- ポート8000 がマシン上の別のプログラム (たとえば自分のアプリケーション!) によって使われていると、テストは失敗します。
- 2つ以上のテストを並行して実行しようとすると、1つのテストだけがポートのバインドに成功し、他のテストはすべて失敗する。
- 
もっといい方法があります。テストでは、バックグラウンドのアプリケーションをランダムに利用可能なポートで実行しなければなりません。
まず最初に、`run`関数を変更する必要があります。ハードコードされた値に頼るのではなく、 アプリケーションのアドレスを引数として受け取るようにします。

```rust
//! src/lib.rs
// [...]

pub fn run(address: &str) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| App::new().route("/health_check", web::get().to(health_check)))
        .bind(address)?
        .run();
    Ok(server)
}
```

同じ動作を維持してプロジェクトを再コンパイルするには、すべての`zero2prod::run()`の呼び出しを`zero2prod::run("127.0.0.1:8000")`に変更する必要があります。

では、どうやってテスト用のポートをランダムに見つけるのでしょうか？
ここではポート`0`を使用します。
ポート`0`は、OSレベルでは特別なケースです。ポート`0`をバインドしようとすると、OSが利用可能なポートをスキャンし、アプリケーションにバインドされます。

したがって、`spawn_app`を次のように変更するだけで十分です。

```rust
//! tests/health_check.rs
// [...]

fn spawn_app() {
    let server = zero2prod::run("127.0.0.1:0").expect("Failed to bind address");
    let _ = tokio::spawn(server);
}
```

これで、`cargo test`を起動するたびに、バックグラウンドアプリがランダムなポートで実行されるようになりました。
ただ、ちょっとした問題があります...テストが失敗しています!

```
running 1 test
test health_check_works ... FAILED

failures:

---- health_check_works stdout ----
thread 'health_check_works' panicked at 'Failed to execute request.: reqwest::Error { kind: Request, url: "http://localhost:8000/health_check", source: hyper::Error(Connect, ConnectError("tcp connect error", Os { code: 111, kind: ConnectionRefused, message: "Connection refused" })) }', tests/health_check.rs:10:20
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
Panic in Arbiter thread.


failures:
    health_check_works

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
```

HTTPクライアントは`127.0.0.1:8000`を呼び出していますが、ここに何を置けばいいのかわかりません。アプリケーションのポートはランタイムに決定されるので、ハードコードすることはできません。
アプリケーションのポートは実行時に決定されるので、そこにハードコードすることはできません。どうにかして、OSが私たちのアプリケーションに与えたポートを見つけ出し、`spawn_app`からそれを返す必要があります。

これにはいくつかの方法がありますが、ここでは`std::net::TcpListener`を使用します。
今、私たちの`HttpServer`は二重の役割を果たしています：アドレスが与えられると、それをバインドして、アプリケーションを起動します。`TcpListener`を使ってポートをバインドし、`listen`を使って`HttpServer`にそれを渡します。

その利点は何でしょうか？
`TcpListener::local_addr`は`SocketAddr`を返し、`.port()`でバインドした実際のポートを公開します。

まず、`run`関数から始めましょう。

```rust
//! src/lib.rs

use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer};
use std::net::TcpListener;

// [...]

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| App::new().route("/health_check", web::get().to(health_check)))
        .listen(listener)?
        .run();
    Ok(server)
}
```

この変更により、`main`と`spawn_app`関数の両方が壊れました。`main`はお任せして、`spawn_app`に注目してみましょう。

```rust
//! tests/health_check.rs
// [...]

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    // We retrieve the port assigned to us by the OS
    let port = listener.local_addr().unwrap().port();
    let server = zero2prod::run(listener).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    // We return the application address to the caller!
    format!("http://127.0.0.1:{}", port)
}
```

これで、テストのアプリケーション・アドレスを利用して、`reqwest::Client`を指定することができます。

```rust
//! tests/health_check.rs
// [...]

#[actix_rt::test]
async fn health_check_works() {
    // Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let response = client
        // Use the returned application address
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
```

`cargo test`はグリーンになりました。私たちのセットアップはより強固になりました。

# Chapter3.5

## 2.Working With HTML forms

### 2.1.Refining Our Requirements

メールマガジンの購読者として登録するためには、訪問者からどのような情報を収集すればよいのでしょうか？

そうですね、確かにメールアドレスは必要です（結局、メールマガジンですからね）。
他には？

通常のビジネス環境であれば、チーム内のエンジニアとプロダクトマネージャーの間で、このような会話が交わされます。この場合、私たちはテクニカルリードであると同時にプロダクトオーナーでもあるので、私たちが主導権を握ることができるのです。

個人的な経験から言うと、一般的に人々はニュースレターを購読する際に、スローアウェイメールやマスクメールを使用します（少なくとも、『Zero To Production』を購読する際には、ほとんどの方がそうしていました）。
そのため、メールでの挨拶（悪名高い`Hey {subscriber.name}}!`）に使える名前を集めたり、購読者のリストの中から共通の知り合いを見つけたりすることができたらいいと思います。
私たちは警官ではありませんし、名前の欄が本物であることに興味はありません。私たちのニュースレターシステムで自分の識別子として使いたいと思うものを入力してもらいます。[`DenverCoder9`](https://xkcd.com/979/), we welcome you.

つまり、新規登録者にはメールアドレスと名前を入力してもらうことになります。

データはHTMLフォームで収集されるので、POSTリクエストのボディでバックエンドAPIに渡されます。ボディはどのようにエンコードされるのでしょうか？
HTMLフォームを使用する際には、いくつかの[オプション](https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/POST)がありますが、今回のユースケースでは、`application/x-www-form-urlencoded`が最も適しています。

>キーと値は、キーと値の間に「=」を挟み、「&」で区切られたキー・バリュータプルでエンコードされます。キーと値の両方に含まれる英数字以外の文字はパーセントエンコードされます。

例：名前が`Le Guin`でメールが `ursula_le_guin@gmail.com`の場合、POSTリクエストボディは `name=le%20guin&email=ursula_le_guin%40gmail.com`となります（スペースは`%20`に、`@`は`%40`に置き換えられます-参考変換表は[こちら](https://www.w3schools.com/tags/ref_urlencode.ASP)）。

要約すると

- nameとemailの有効なペアが`application/x-www-form-urlencoded`フォーマットで提供された場合、バックエンドは`200 OK`を返します。
- nameとemailのどちらかが欠けている場合、バックエンドは`400 BAD REQUEST`を返します。

### 2.2.Capturing Our Requirements As Tests

さて、何が起こるべきかをよく理解したところで、期待することをいくつかの統合テストで表現してみましょう。

新しいテストを既存の `tests/health_check.rs` ファイルに追加しましょう。テストスイートのフォルダ構造は後で整理します。

```rust
//! tests/health_check.rs
use zero2prod::run;
use std::net::TcpListener;

/// Spin up an instance of our application 
/// and returns its address (i.e. http://localhost:XXXX)
fn spawn_app() -> String {
    [...]
}

#[actix_rt::test]
async fn health_check_works() {
    [...]
}

#[actix_rt::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let app_address = spawn_app();
    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // Act
    let response = client
        .post(&format!("{}/subscriptions", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());
}


#[actix_rt::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let app_address = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email")
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &app_address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}
```

`subscribe_returns_a_400_when_data_is_missing`はテーブル駆動型テストの一例で、パラメトリックテストとしても知られています。
テストロジックを何度も繰り返すのではなく、同じように失敗することが予想される既知の無効なボディの集合に対して同じアサーションを実行すればよいのです。
パラメトリックテストでは、失敗したときに適切なエラーメッセージを表示することが重要です。逆に言えば、パラメータ化されたテストは多くの領域をカバーしているので、素敵な失敗メッセージを生成するためにもう少し時間をかけるのは理にかなっているということです。
他の言語のテストフレームワークでは、このようなテストスタイルをネイティブでサポートしていることがあります(例えば、[`pytest`のパラメトリックテスト](https://docs.pytest.org/en/stable/parametrize.html)や[`C#`のxUnitの`InlineData`](https://andrewlock.net/creating-parameterised-tests-in-xunit-with-inlinedata-classdata-and-memberdata/))- Rustのエコシステムには、似たような機能で基本的なテストフレームワークを拡張するいくつかのクレートがありますが、残念ながら非同期テストを書くのに必要な`#[actix_rt::test]`マクロとの相互運用性は高くありません([`rstest`](https://github.com/la10736/rstest/issues/85)や[`test-case`](https://github.com/frondeus/test-case/issues/36)を参照)。

それでは、テスト・スイートを実行してみましょう。

```
---- health_check::subscribe_returns_a_200_for_valid_form_data stdout ----
thread 'health_check::subscribe_returns_a_200_for_valid_form_data' 
panicked at 'assertion failed: `(left == right)`
  left: `200`,
 right: `404`: 

---- health_check::subscribe_returns_a_400_when_data_is_missing stdout ----
thread 'health_check::subscribe_returns_a_400_when_data_is_missing' 
panicked at 'assertion failed: `(left == right)`
  left: `400`,
 right: `404`: 
 The API did not fail with 400 Bad Request when the payload was missing the email.'
 ```
 
 予想通り、新しいテストはすべて失敗しています。
「ロール・ユア・オーナー方式」のパラメトリックテストの限界がすぐにわかります。 一つのテストケースが失敗するとすぐに実行が停止してしまい、 次のテストケースの結果がわからないのです。

それでは早速、実装を始めてみましょう。

### 2.3.Parsing Form Data From A POST Request

アプリケーションが、`/subscriptions`へのPOSTリクエストに対して`404 NOT FOUND`を返すため、すべてのテストが失敗します。

正当な動作：そのパスに対応するハンドラが登録されていません。

この問題を解決するために、`src/lib.rs`の`App`にマッチするルートを追加しましょう。

```rust
//! src/lib.rs
use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer};
use std::net::TcpListener;

// We were returning `impl Responder` at the very beginning.
// We are now spelling out the type explicitly given that we have become more familiar with `actix-web`.
// There is no performance difference! Just a stylistic choice :)
async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

// Let's start simple: we always return a 200 OK
async fn subscribe() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(health_check))
            // A new entry in our routing table for POST /subscriptions requests
            .route("/subscriptions", web::post().to(subscribe))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
```

テスト・スイートの再実行

```
running 3 tests
test health_check::health_check_works ... ok
test health_check::subscribe_returns_a_200_for_valid_form_data ... ok
test health_check::subscribe_returns_a_400_when_data_is_missing ... FAILED

failures:

---- health_check::subscribe_returns_a_400_when_data_is_missing stdout ----
thread 'health_check::subscribe_returns_a_400_when_data_is_missing' 
panicked at 'assertion failed: `(left == right)`
  left: `400`,
 right: `200`: 
 The API did not fail with 400 Bad Request when the payload was missing the email.'

failures:
    health_check::subscribe_returns_a_400_when_data_is_missing

test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
```

`subscribe_returns_a_200_for_valid_form_data`は、現在パスしています：私たちのハンドラはすべての入力データを有効なものとして受け入れます。
一方、`subscribe_returns_a_400_when_data_is_missing` はまだ赤です。
リクエストボディを実際に解析してみましょう。`actix-web`は私たちに何を提供してくれるのでしょうか？

#### 2.3.1.Extractors

`actix-web`のユーザーガイドの中で、かなり目立つのが「抽出器」のセクションです。
抽出器は、その名の通り、入力されたリクエストから特定の情報を抽出するようにフレームワークに指示するために使用されます。
`actix-web`には、最も一般的な使用例に対応するために、いくつかの抽出器が用意されています。

- [Path](https://docs.rs/actix-web/4.0.0-beta.1/actix_web/web/struct.Path.html)は、リクエストのパスから動的なパスセグメントを取得します。
- [Query](https://docs.rs/actix-web/4.0.0-beta.1/actix_web/web/struct.Query.html)は、クエリパラメータを取得します。
- [Json](https://docs.rs/actix-web/4.0.0-beta.1/actix_web/web/struct.Json.html)は、JSON エンコードされたリクエストボディを解析します。

などです。
幸いなことに、私たちのニーズにぴったりの抽出器があります。[Form](https://docs.rs/actix-web/4.0.0-beta.1/actix_web/web/struct.Form.html)です。
ドキュメントを読んでみましょう。

>フォームデータヘルパー（`application/x-www-form-urlencoded`）。
リクエストボディからURLエンコードされたデータを抽出したり、URLエンコードされたデータをレスポンスとして送信したりするのに使用できます。

それは、私の耳には音楽のように聞こえます。
どうやって使うの？

`actix-web`のユーザーガイドを見ています。

>抽出器は、ハンドラ関数の引数としてアクセスできます。Actix-webでは、ハンドラ関数ごとに最大10個の抽出器をサポートしています。引数の位置は関係ありません。

Example:

```rust
use actix_web::web;

#[derive(serde::Deserialize)]
struct FormData {
    username: String,
}

/// Extract form data using serde.
/// This handler get called only if content type is *x-www-form-urlencoded*
/// and content of the request could be deserialized to a `FormData` struct
fn index(form: web::Form<FormData>) -> String {
    format!("Welcome {}!", form.username)
}
```

つまり、基本的には...ハンドラーの引数としてそれを置くだけで、リクエストが来たときに actix-web があなたのために重労働をしてくれるのです。今のところは一緒にやってみましょう。フードの下で何が起こっているかを理解するために、後でまた戻ってきます。

私たちの `subscribe`ハンドラは現在次のようになっています。

```rust
//! src/lib.rs
// Let's start simple: we always return a 200 OK
async fn subscribe() -> HttpResponse {
    HttpResponse::Ok().finish()
}
```

この例を参考にすると、おそらく次のようなものになるでしょう。

```rust
//! src/lib.rs
// [...]

#[derive(serde::Deserialize)]
struct FormData {
    email: String,
    name: String
}

async fn subscribe(_form: web::Form<FormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
```

`cargo check`がうまくいかない

```
error[E0433]: failed to resolve: use of undeclared type or module `serde`
 --> src/lib.rs:9:10
  |
9 | #[derive(serde::Deserialize)]
  |          ^^^^^ use of undeclared type or module `serde`
```

それはいいとして、依存関係に`serde`を追加する必要があります。それでは、`Cargo.toml`に新しい行を追加してみましょう。

```toml
[dependencies]
# We need the optional `derive` feature to use `serde`'s procedural macros: 
# `#[derive(Serialize)]` and `#[derive(Deserialize)]`.
# The feature is not enabled by default to avoid pulling in 
# unnecessary dependencies for projects that do not need it.
serde = { version = "1", features = ["derive"]}
```

`cargo check`が成功するはずです。`cargo test`はどうですか？

```
running 3 tests
test health_check_works ... ok
test subscribe_returns_a_200_for_valid_form_data ... ok
test subscribe_returns_a_400_when_data_is_missing ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

すべて成功しました。

でも、なぜ？

#### 2.3.2.`Form` and `FromRequest`

それでは早速、`Form`はどのようなものなのでしょうか？
`Form`のソースコードは[ここ](https://github.com/actix/actix-web/blob/master/src/types/form.rs)にあります。

その定義はかなり無邪気なものです。

```rust
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Form<T>(pub T);
```

これは単なるラッパーで、`Form`の唯一のフィールドを生成するために使用される`T`型のジェネリックです。
ここではあまり見るべきものはありません。
抽出のマジックはどこで行われるのでしょうか？

抽出器は、`FromRequest`のトレイトを実装した型です。
Rustは トレイトの定義で`async fn`をまだサポートしていないので、`FromRequest`の定義は少し不自然です。これを少し変えてみると、だいたい以下のようになります。

```rust
/// Trait implemented by types that can be extracted from request.
///
/// Types that implement this trait can be used with `Route` handlers.
pub trait FromRequest: Sized {
    type Error = Into<actix_web::Error>;

    async fn from_request(req: &HttpRequest, payload: &mut Payload) -> Result<Self, Self::Error>;
    
    /// Omitting some ancillary methods that actix-web implements 
    /// out of the box for you and supporting associated types
    ///  [...]
}
```

`from_request`は、入力として、受信したHTTPリクエストのヘッド(すなわち`HttpRequest`)と、そのペイロードのバイト(すなわち`Payload`)を受け取ります。抽出が成功した場合はSelfを、`actix_web::Error`に変換可能なエラータイプを返します。
`actix-web`は各引数に対して`from_request`を起動し、すべての引数の抽出に成功した場合は、実際のハンドラ関数を実行します。
`actix-web`は各引数に対して`from_request`を起動し、すべての引数の抽出に成功した場合は、実際のハンドラ関数を実行します。抽出に失敗した場合は、対応するエラーが呼び出し元に返され、ハンドラは決して起動されません（`actix_web::Error`は`HttpResponse`に変換できます）。

これは非常に便利なことです。ハンドラーは受信した生のリクエストを処理する必要がなく、代わりに強力に型付けされた情報を直接扱うことができるので、リクエストを処理するために書かなければならないコードを大幅に簡素化することができます。

それでは、`Form`の`FromRequest`の実装を見てみましょう：これは何をしているのでしょうか？
繰り返しになりますが、重要な要素を強調するために実際のコードを少し変更し、細かい実装の詳細は無視しています。

```rust
impl<T> FromRequest for Form<T>
where
    T: DeserializeOwned + 'static,
{
    type Error = actix_web::Error;

    async fn from_request(req: &HttpRequest, payload: &mut Payload) -> Result<Self, Self::Error> {
        // Omitted stuff around extractor configuration (e.g. payload size limits)

        match UrlEncoded::new(req, payload).await {
            Ok(item) => Ok(Form(item)),
            // The error handler can be customised.
            // The default one will return a 400, which is what we want.
            Err(e) => Err(error_handler(e))
        }       
    }
}
```

すべての作業は、`UrlEncoded`構造体の中で行われているようです。
`UrlEncoded`は多くのことを行います。圧縮および非圧縮のペイロードを透過的に処理したり、リクエスト ボディが一度にチャンクごとにバイトのストリームとして到着することを処理したりします。
これらすべての処理が終わった後の重要な一節は、次のとおりです。

```rust
serde_urlencoded::from_bytes::<T>(&body).map_err(|_| UrlencodedError::Parse)
```

`serde_urlencoded`は，`application/x-www-form-urlencoded`データフォーマットの（脱）シリアライズをサポートします．
`from_bytes`は入力として連続したバイトのスライスを受け取り、そこから型`T`のインスタンスを URLエンコード形式のルールに従ってデシリアライズします: キーと値は、キーと値の間に`=`を挟んで`&`で区切られたキー・値タプルにエンコードされます; キーと値の両方の非英数字はパーセントエンコードされます。

ジェネリックな型`T`に対して、どうしてその方法がわかるのでしょうか？
それは`T`が`serde`の`DeserializedOwned`トレイトを実装しているからです。

```rust
impl<T> FromRequest for Form<T>
where
    T: DeserializeOwned + 'static,
{
// [...]
}
```

実際に何が行われているのかを理解するためには、`serde`自体をよく見てみる必要があります。

#### 2.3.3.Rustのシリアル化：`serde`

>このserdeのセクションでは、Rustの高度なトピックにいくつか触れています。
初めて読んだときに、すべてがうまくいかなくても大丈夫です。
Rustと`serde`をもう少し使い込んだら、このセクションに戻ってきて、難しい部分を深く掘り下げてみてください。

なぜ`serde`が必要なのか？実際`serde`は私たちのために何をしてくれるのでしょうか？
その[ガイド](https://serde.rs/)から引用します。

>`Serde`は、Rustデータ構造を効率的かつ汎用的にシリアライズ/デシリアライズするためのフレームワークです。

#### 2.3.3.1.Generically

JSON、Avro、MessagePack の仕様を扱うコードは、serde の内部には存在しません。特定のデータ形式をサポートする必要がある場合は、別の木箱 (例: JSON用の `serde_json`やAvro用の`avro-rs`など) を使用する必要があります。
`serde`は一連のインターフェイスを定義しており、彼ら自身が「データモデル」と呼んでいます。

新しいデータ形式のシリアライズをサポートするライブラリを実装したい場合、[`Serializer`トレイト](https://docs.serde.rs/serde/trait.Serializer.html)の実装を提供する必要があります。
`Serializer`トレイトの各メソッドは、`serde`のデータモデルを形成する 29 の型のうちの 1 つに対応します。Serializer の実装では、これらの型のそれぞれが特定のデータ形式にどのように対応するのかを指定します。
たとえば、JSON のシリアライズをサポートする場合、[`serialize_seq`](https://docs.serde.rs/serde/trait.Serializer.html#tymethod.serialize_seq)の実装は、開始角括弧`[`を出力し、シーケンス要素をシリアライズするために使用できる型を返すでしょう。

一方、`Serialize`トレイトがあります。Rust 型に対する Serialize::serialize の実装は、`Serializer`トレイトで利用可能なメソッドを使用して、`serde`のデータモデルに従ってそれを分解する方法を指定することを意図しています。
再び配列の例で説明すると、Rust ベクトルに対する `Serialize`の実装はこのようになります。

```rust
use serde::ser::{Serialize, Serializer, SerializeSeq};

impl<T> Serialize for Vec<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for element in self {
            seq.serialize_element(element)?;
        }
        seq.end()
    }
}
```

これによって、`serde`はデータ形式を問わないことになります。つまり、`crate.io`に`Serializer`の実装があれば、どのようなフォーマットでもシリアライズすることができます（ネタバレ：よく使われるデータフォーマットのほとんど全て）。
デシリアライズについても同様で、`Deserialize`と`Deserializer`を使用し、ゼロコピーデシリアライズをサポートするためにライフタイムに関するいくつかの詳細が追加されています。

#### 2.3.3.2.Efficiently

