use crate::config::get_config;
use crate::models::{FileInfo, UploadResponse};
use crate::storage::generate_unique_filename;
use salvo::http::ParseError;
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
pub async fn handle_upload(req: &mut Request, res: &mut Response) {
    let config = get_config();

    let (temp_path, original_name, content_type, size) = match req.try_file("file").await {
        Ok(Some(file)) => (
            file.path().clone(),
            file.name().unwrap_or("unknown").to_string(),
            file.content_type()
                .map(|mime| mime.to_string())
                .unwrap_or_else(|| "application/octet-stream".to_string()),
            file.size(),
        ),
        Ok(None) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(UploadResponse::error("未找到上传的文件".to_string())));
            return;
        }
        Err(ParseError::PayloadTooLarge) => {
            res.status_code(StatusCode::PAYLOAD_TOO_LARGE);
            res.render(Json(UploadResponse::error("上传文件过大".to_string())));
            return;
        }
        Err(ParseError::InvalidContentType | ParseError::NotMultipart | ParseError::NotFormData) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(UploadResponse::error("无效的 Content-Type".to_string())));
            return;
        }
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(UploadResponse::error(format!("读取请求体失败: {}", e))));
            return;
        }
    };

    let unique_name = match generate_unique_filename(&config.storage_path, &original_name).await {
        Ok(name) => name,
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(UploadResponse::error(format!("生成文件名失败: {}", e))));
            return;
        }
    };

    let file_path = config.storage_path.join(&unique_name);

    if let Err(e) = tokio::fs::copy(&temp_path, &file_path).await {
        res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
        res.render(Json(UploadResponse::error(format!("保存文件失败: {}", e))));
        return;
    }

    let file_info = FileInfo::new(unique_name, size, content_type);
    res.render(Json(UploadResponse::success(file_info)));
}
