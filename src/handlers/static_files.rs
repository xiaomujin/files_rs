use salvo::prelude::*;
use salvo::serve_static::StaticFile;
use std::path::PathBuf;
use crate::config::get_config;

/// 静态文件服务处理函数
/// 
/// 用于提供前端静态文件服务，返回 index.html 页面
/// 
/// # 参数
/// 
/// - `res`: HTTP 响应对象
#[handler]
pub async fn serve_index(res: &mut Response) {
    let static_path = PathBuf::from("static/index.html");
    
    if static_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&static_path) {
            res.render(Text::Html(content));
            return;
        }
    }
    
    res.status_code(StatusCode::NOT_FOUND);
    res.render(Text::Html("<h1>404 - 页面未找到</h1>".to_string()));
}

/// 创建静态文件服务
/// 
/// 返回一个用于处理静态文件请求的处理器
/// 
/// # 返回值
/// 
/// 返回静态文件处理器
pub fn create_static_handler() -> impl Handler {
    let config = get_config();
    let storage_path = &config.storage_path;
    StaticFile::new(storage_path)
}
