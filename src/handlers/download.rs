use salvo::prelude::*;
use salvo::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use salvo::http::ResBody;
use crate::config::get_config;
use bytes::Bytes;

/// 文件下载处理函数
/// 
/// 根据文件名下载文件，文件名作为文件唯一标识符
/// 
/// # 参数
/// 
/// - `req`: HTTP 请求对象
/// - `res`: HTTP 响应对象
/// - `_depot`: 用于存储请求上下文的仓库
#[handler]
pub async fn download(req: &mut Request, res: &mut Response, _depot: &mut Depot) {
    let config = get_config();
    
    let file_name: String = match req.param("id") {
        Some(id) => id,
        None => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "success": false,
                "message": "缺少文件名"
            })));
            return;
        }
    };
    
    match validate_filename(&file_name) {
        ValidationResult::Invalid(msg) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "success": false,
                "message": msg
            })));
            return;
        }
        ValidationResult::Valid => {}
    }
    
    let file_path = config.storage_path.join(&file_name);
    
    if !file_path.exists() {
        res.status_code(StatusCode::NOT_FOUND);
        res.render(Json(serde_json::json!({
            "success": false,
            "message": "文件不存在"
        })));
        return;
    }
    
    let file_data = match std::fs::read(&file_path) {
        Ok(d) => d,
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "success": false,
                "message": format!("读取文件失败: {}", e)
            })));
            return;
        }
    };
    
    res.add_header(CONTENT_TYPE, "application/octet-stream", true)
        .unwrap();
    res.add_header(
        CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", file_name),
        true,
    )
    .unwrap();
    
    res.body = ResBody::Once(Bytes::from(file_data));
}

/// 文件名验证结果
/// 
/// 用于返回文件名验证的结果
enum ValidationResult {
    Valid,
    Invalid(String),
}

/// 验证文件名是否合法
/// 
/// 检查文件名是否包含路径遍历等非法字符
/// 
/// # 参数
/// 
/// - `file_name`: 要验证的文件名
/// 
/// # 返回值
/// 
/// 返回验证结果
fn validate_filename(file_name: &str) -> ValidationResult {
    if file_name.is_empty() {
        return ValidationResult::Invalid("文件名不能为空".to_string());
    }
    
    if file_name.contains('/') || file_name.contains('\\') || file_name.contains("..") {
        return ValidationResult::Invalid("文件名包含非法字符".to_string());
    }
    
    ValidationResult::Valid
}
