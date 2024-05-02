use actix_web::{web, App, HttpServer};
use clap::Parser;
use openapi_mocker::{
    openapi::spec::Spec,
    server::{get_scope, AppState},
    Args,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    println!("Starting server with spec: {}", args.spec.display());

    let port = args.port.unwrap_or(8080);
    let spec = Spec::from_path(args.spec.to_str().unwrap_or("")).expect("Failed to load spec");
    let data = web::Data::new(AppState { spec });

    let server = HttpServer::new(move || App::new().app_data(data.clone()).service(get_scope()))
        .bind(("0.0.0.0", port))
        .expect("Failed to bind to port");

    server.run().await.expect("Failed to run server");

    Ok(())
}
