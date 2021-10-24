use std::net::TcpListener;

use zero2prod::run;

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    // OSから割り当てられたポートを回収する
    let port = listener.local_addr().unwrap().port();
    let server = run(listener).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    // アプリケーションのアドレスを発信者に返します。
    format!("http://127.0.0.1:{}", port)
}

#[actix_rt::test]
async fn health_check_works() {
    let address = spawn_app();
    let client = reqwest::Client::new();

    let response = client
            .get(&format!("{}/health_check", &address))
            .send()
            .await
            .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[actix_rt::test]
async fn subscribe_returns_a_200_for_valid_from_data() {
    let app_address = spawn_app();
    // 空のClientインスタンスを定義
    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = client
        // URLへのPOSTリクエストを行う
        .post(&format!("{}/subscriptions", &app_address))
        // リクエストにヘッダーを追加
        .header("Content-Type", "application/x-www-form-urlencoded")
        // リクエストボディを設定
        .body(body)
        // Requestを構築し、ターゲットのURLに送信し、Responseを返す
        .send()
        .await
        .expect("Failed to execute reauwest");

    assert_eq!(200, response.status().as_u16());
}


#[actix_rt::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app_address = spawn_app();
    let client = reqwest::Client::new();
    let test_case = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email")
    ];

    for (invalid_body, error_message) in test_case {
        let response = client
            .post(&format!("{}/subscriptions", &app_address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");
        
        assert_eq!(
            400,
            response.status().as_u16(),
            // テスト失敗時のカスタマイズされたエラーメッセージの追加
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        )
    }
}