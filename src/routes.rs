use crate::config::Config;
use crate::handlers::{
    delete_file, handle_download, handle_upload, list_files, rename_file, serve_index,
};
use salvo::cors::{Any, Cors};
use salvo::http::request::SecureMaxSize;
use salvo::prelude::*;

pub fn build_router(config: &Config) -> Router {
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

    Router::new()
        .hoop(cors)
        .get(serve_index)
        .push(Router::with_path("/api").push(api_router))
        .push(
            Router::with_path("static/{**path}").get(
                StaticDir::new(config.storage_path.clone())
                    .auto_list(true)
                    .include_dot_files(false),
            ),
        )
}
