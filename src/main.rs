mod config;
mod filename;
mod handlers;
mod models;
mod storage;

use handlers::{delete_file, handle_download, handle_upload, list_files, rename_file, serve_index};
use salvo::cors::{Any, Cors};
use salvo::http::request::SecureMaxSize;
use salvo::prelude::*;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let config = config::get_config();

    let cors = Cors::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .into_handler();

    let api_router = Router::new()
        .push(Router::with_path("/files").get(list_files))
        .push(
            Router::with_path("/upload")
                .hoop(SecureMaxSize(1024 * 1024 * 1024 * 10))
                .post(handle_upload),
        )
        .push(Router::with_path("/download/{id}").get(handle_download))
        .push(Router::with_path("/files/{id}").delete(delete_file))
        .push(Router::with_path("/files/{id}").put(rename_file));

    let router = Router::new()
        .hoop(cors)
        .get(serve_index)
        .push(Router::with_path("/api").push(api_router))
        .push(
            Router::with_path("static/{**path}").get(
                StaticDir::new(config.storage_path.clone())
                    .auto_list(true) // 开发时方便查看目录结构
                    .include_dot_files(false),
            ),
        );
    println!("{:?}", router);
    let acceptor = TcpListener::new("0.0.0.0:3000").bind().await;
    Server::new(acceptor).serve(router).await;
}
