use crate::config::get_config;
use crate::models::{FileInfo, UploadResponse, generate_unique_filename};
use futures_util::stream::once;
use multer::Multipart;
use salvo::prelude::*;

/// 文件上传处理函数
/// 
/// 处理 multipart/form-data 格式的文件上传请求，
/// 将文件保存到配置的存储目录中，使用原文件名存储，
/// 如果文件名已存在则自动添加时间戳后缀
/// 
/// # 参数
/// 
/// - `req`: HTTP 请求对象
/// - `res`: HTTP 响应对象
#[handler]
pub async fn upload(req: &mut Request, res: &mut Response) {
    let config = get_config();
    
    let content_type = req.header::<String>("content-type")
        .unwrap_or_default();
    
    let boundary = match extract_boundary(&content_type) {
        Some(b) => b,
        None => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(UploadResponse::error("无效的 Content-Type".to_string())));
            return;
        }
    };
    
    let body = match req.payload().await {
        Ok(b) => b,
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(UploadResponse::error(format!("读取请求体失败: {}", e))));
            return;
        }
    };
    
    let body_bytes = body.to_vec();
    
    let stream = once(async move { Ok::<_, std::io::Error>(body_bytes) });
    let mut multipart = Multipart::new(stream, boundary);
    
    while let Ok(Some(field)) = multipart.next_field().await {
        let field_name = field.name().map(|s| s.to_string()).unwrap_or_default();
        
        if field_name != "file" {
            continue;
        }
        
        let original_name = field.file_name().map(|s| s.to_string()).unwrap_or_else(|| "unknown".to_string());

        let content_type = field.content_type().map(|s| s.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());
        
        let data = match field.bytes().await {
            Ok(d) => d,
            Err(e) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Json(UploadResponse::error(format!("读取文件数据失败: {}", e))));
                return;
            }
        };
        
        let size = data.len() as u64;
        
        let unique_name = generate_unique_filename(&config.storage_path, &original_name);
        
        let file_path = config.storage_path.join(&unique_name);
        
        if let Err(e) = std::fs::write(&file_path, &data) {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(UploadResponse::error(format!("保存文件失败: {}", e))));
            return;
        }
        
        let file_info = FileInfo::new(unique_name, size, content_type);
        
        res.render(Json(UploadResponse::success(file_info)));
        return;
    }
    
    res.status_code(StatusCode::BAD_REQUEST);
    res.render(Json(UploadResponse::error("未找到上传的文件".to_string())));
}

/// 从 Content-Type header 中提取 boundary
/// 
/// # 参数
/// 
/// - `content_type`: Content-Type header 值
/// 
/// # 返回值
/// 
/// 返回 boundary 字符串
fn extract_boundary(content_type: &str) -> Option<String> {
    for part in content_type.split(';') {
        let part = part.trim();
        if part.starts_with("boundary=") {
            let boundary = part["boundary=".len()..].to_string();
            return Some(boundary.trim_matches('"').to_string());
        }
    }
    None
}
