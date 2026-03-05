use std::path::Path;
use crate::config::get_config;
use crate::models::{FileInfo, UploadResponse};
use futures_util::stream::once;
use multer::Multipart;
use salvo::prelude::*;

/// 文件上传处理函数
/// 
/// 处理 multipart/form-data 格式的文件上传请求，
/// 将文件保存到配置的存储目录中
/// 
/// # 参数
/// 
/// - `req`: HTTP 请求对象
/// - `res`: HTTP 响应对象
#[handler]
pub async fn upload(req: &mut Request, res: &mut Response) {
    let config = get_config();
    
    // 获取 Content-Type header
    let content_type = req.header::<String>("content-type")
        .unwrap_or_default();
    
    // 解析 boundary
    let boundary = match extract_boundary(&content_type) {
        Some(b) => b,
        None => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(UploadResponse::error("无效的 Content-Type".to_string())));
            return;
        }
    };
    
    // 获取请求体数据
    let body = match req.payload().await {
        Ok(b) => b,
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(UploadResponse::error(format!("读取请求体失败: {}", e))));
            return;
        }
    };
    
    // 克隆数据用于 multipart 解析
    let body_bytes = body.to_vec();
    
    // 创建 Multipart 解析器
    let stream = once(async move { Ok::<_, std::io::Error>(body_bytes) });
    let mut multipart = Multipart::new(stream, boundary);
    
    // 处理 multipart 字段
    while let Ok(Some(field)) = multipart.next_field().await {
        // 获取字段名称
        let field_name = field.name().map(|s| s.to_string()).unwrap_or_default();
        
        // 只处理文件字段
        if field_name != "file" {
            continue;
        }
        
        // 获取原始文件名
        let original_name = field.file_name().map(|s| s.to_string()).unwrap_or_else(|| "unknown".to_string());

        // 获取内容类型
        let content_type = field.content_type().map(|s| s.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());
        
        // 读取文件数据
        let data = match field.bytes().await {
            Ok(d) => d,
            Err(e) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Json(UploadResponse::error(format!("读取文件数据失败: {}", e))));
                return;
            }
        };
        
        let size = data.len() as u64;
        
        // 创建文件信息
        let file_info = FileInfo::new(
            original_name,
            config.storage_path.to_string_lossy().to_string(),
            size,
            content_type,
        );
        
        // 保存文件
        let file_path = file_info.full_path();
        if let Err(e) = std::fs::write(&file_path, &data) {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(UploadResponse::error(format!("保存文件失败: {}", e))));
            return;
        }
        
        // 返回成功响应
        res.render(Json(UploadResponse::success(file_info)));
        return;
    }
    
    // 没有找到文件字段
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
            // 移除可能的引号
            return Some(boundary.trim_matches('"').to_string());
        }
    }
    None
}
