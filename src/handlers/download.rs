use crate::config::get_config;
use crate::filename::{validate_filename, FilenameValidationError};
use salvo::fs::NamedFile;
use salvo::prelude::*;

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
pub async fn handle_download(req: &mut Request, res: &mut Response, _depot: &mut Depot) {
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
        Ok(()) => {}
        Err(FilenameValidationError::Empty) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "success": false,
                "message": "文件名不能为空"
            })));
            return;
        }
        Err(FilenameValidationError::InvalidChars) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "success": false,
                "message": "文件名包含非法字符"
            })));
            return;
        }
    }

    let file_path = config.storage_path.join(&file_name);
    if !tokio::fs::try_exists(&file_path).await.unwrap_or(false) {
        res.status_code(StatusCode::NOT_FOUND);
        res.render(Json(serde_json::json!({
            "success": false,
            "message": "文件不存在"
        })));
        return;
    }

    match NamedFile::builder(&file_path)
        .attached_name(file_name.clone())
        .content_type("application/octet-stream".parse().unwrap())
        .build()
        .await
    {
        Ok(file) => file.send(req.headers(), res).await,
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "success": false,
                "message": format!("读取文件失败: {}", e)
            })));
        }
    }
}
