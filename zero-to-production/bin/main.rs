use zero_to_production::run;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    run()?.await
}