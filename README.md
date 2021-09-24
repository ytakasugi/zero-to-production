# [zero-to-production](https://www.lpalmieri.com/posts/2020-05-24-zero-to-production-0-foreword/)

# Chapter3

### 3.Our First Endpoint: A Basic Health Check

ヘルスチェック・エンドポイントを実装して、まずはスタートしてみましょう。`/health_check`に対するGETリクエストを受信したら、Bodyボディのない200 OKレスポンスを返します。

HEALTH_CHECKを使って、アプリケーションが起動していて、リクエストを受け入れる準備ができているかどうかを確認することができます。これをpingdom.comのようなSaaSサービスと組み合わせれば、APIが暗転したときに警告を受けることができます。これは、副業として運営しているメールマガジンのベースラインとして非常に有効です。

ヘルスチェック・エンドポイントは、アプリケーションの管理にコンテナ・オーケストレーター（KubernetesやNomadなど）を使用している場合にも便利です。オーケストレーターは/health_checkを呼び出して、APIが応答しなくなったことを検出し、再起動のトリガーとすることができます。

#### 3.1.Wiring Up actix-web

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