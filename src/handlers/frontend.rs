use rust_embed::RustEmbed;
use salvo::prelude::*;

#[derive(RustEmbed)]
#[folder = "static/"]
struct Assets;

#[handler]
pub async fn serve_static_file(req: &mut Request, res: &mut Response) {
    let path = req.param::<String>("path").unwrap_or_default();
    let path = if path.is_empty() { "index.html" } else { &path };

    if path.contains("..") || path.starts_with('/') || path.starts_with('\\') {
        res.status_code(StatusCode::BAD_REQUEST);
        return;
    }

    if let Some(content) = Assets::get(path) {
        let mime = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();
        res.headers_mut().insert("content-type", mime.parse().unwrap());
        let _ = res.write_body(content.data.to_vec());
    } else {
        res.status_code(StatusCode::NOT_FOUND);
    }
}
