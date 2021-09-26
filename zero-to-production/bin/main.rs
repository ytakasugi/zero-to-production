use std::net::TcpListener;

use zero_to_production::run;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let address = TcpListener::bind("127.0.0.1:8080")?;
    run(address)?.await
}