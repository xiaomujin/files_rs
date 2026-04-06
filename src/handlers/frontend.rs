use rust_embed::RustEmbed;
use salvo::prelude::*;

#[derive(RustEmbed)]
#[folder = "static/"]
struct Assets;

fn render_embedded_asset(path: &str, res: &mut Response) -> bool {
    if let Some(content) = Assets::get(path) {
        let mime = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();

        res.headers_mut().insert("content-type", mime.parse().unwrap());
        let _ = res.write_body(content.data.to_vec());
        true
    } else {
        false
    }
}

#[handler]
pub async fn serve_index(res: &mut Response) {
    if render_embedded_asset("index.html", res) {
        return;
    }

    res.status_code(StatusCode::NOT_FOUND);
    res.render(Text::Html("<h1>404 - 页面未找到</h1>".to_string()));
}

#[handler]
pub async fn serve_asset(req: &mut Request, res: &mut Response) {
    let path: String = match req.param("path") {
        Some(path) => path,
        None => {
            res.status_code(StatusCode::NOT_FOUND);
            return;
        }
    };

    if path.is_empty() || path.contains("..") || path.starts_with('/') || path.starts_with('\\') {
        res.status_code(StatusCode::BAD_REQUEST);
        return;
    }

    if render_embedded_asset(&path, res) {
        return;
    }

    res.status_code(StatusCode::NOT_FOUND);
}
