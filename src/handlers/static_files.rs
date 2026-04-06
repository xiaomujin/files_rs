use salvo::prelude::*;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "static/"]
struct Assets;

/// 静态文件服务处理函数
///
/// 用于提供前端静态文件服务，返回 index.html 页面
///
/// # 参数
///
/// - `res`: HTTP 响应对象
#[handler]
pub async fn serve_index(res: &mut Response) {
    let path = "index.html";

    if let Some(content) = Assets::get(path) {
        let mime = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();

        res.headers_mut().insert("content-type", mime.parse().unwrap());
        let _ = res.write_body(content.data.to_vec());
        return;
    }

    res.status_code(StatusCode::NOT_FOUND);
    res.render(Text::Html("<h1>404 - 页面未找到</h1>".to_string()));
}