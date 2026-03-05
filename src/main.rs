mod config;
mod handlers;
mod models;

use handlers::{delete_file, handle_download, handle_upload, list_files, rename_file, serve_index};
use salvo::cors::{Any, Cors};
use salvo::prelude::*;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    // 初始化配置
    let _config = config::get_config();

    // 配置 CORS
    let cors = Cors::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .into_handler();

    // 创建 API 路由
    let api_router = Router::new()
        .push(Router::with_path("/files").get(list_files))
        .push(Router::with_path("/upload").post(handle_upload))
        .push(Router::with_path("/download/<id>").get(handle_download))
        .push(Router::with_path("/files/<id>").delete(delete_file))
        .push(Router::with_path("/files/<id>").put(rename_file));

    // 创建主路由
    let router = Router::new()
        .hoop(cors)
        .get(serve_index)
        .push(Router::with_path("/api").push(api_router));
    println!("{:?}", router);
    // 启动服务器
    let acceptor = TcpListener::new("0.0.0.0:3000").bind().await;
    Server::new(acceptor).serve(router).await;
}
