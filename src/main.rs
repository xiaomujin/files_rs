mod config;
mod handlers;
mod models;
mod routes;
mod services;
mod utils;

use salvo::prelude::*;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let config = config::get_config();
    let router = routes::build_router(config);

    let acceptor = TcpListener::new("0.0.0.0:3000").bind().await;
    Server::new(acceptor).serve(router).await;
}
