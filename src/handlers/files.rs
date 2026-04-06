use crate::config::get_config;
use crate::models::{FileInfo, FileListResponse, RenameRequest, UploadResponse};
use crate::services::generate_unique_filename;
use crate::utils::{validate_filename, FilenameValidationError};
use salvo::fs::NamedFile;
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

/// 获取文件列表处理函数
///
/// 返回存储目录中所有文件的列表
///
/// # 参数
///
/// - `res`: HTTP 响应对象
#[handler]
pub async fn list_files(res: &mut Response) {
    let config = get_config();
    let mut files = Vec::new();

    let storage_path = &config.storage_path;
    if !tokio::fs::try_exists(storage_path).await.unwrap_or(false) {
        res.render(Json(FileListResponse::new(files)));
        return;
    }

    if let Ok(mut entries) = tokio::fs::read_dir(storage_path).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            let metadata = match entry.metadata().await {
                Ok(metadata) => metadata,
                Err(_) => continue,
            };

            if !metadata.is_file() {
                continue;
            }

            let file_info = FileInfo::from_path(&path, &metadata);
            files.push(file_info);
        }
    }

    files.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    res.render(Json(FileListResponse::new(files)));
}

/// 删除文件处理函数
///
/// 根据文件名删除指定文件
///
/// # 参数
///
/// - `req`: HTTP 请求对象
/// - `res`: HTTP 响应对象
#[handler]
pub async fn delete_file(req: &mut Request, res: &mut Response) {
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

    if validate_filename(&file_name).is_err() {
        res.status_code(StatusCode::BAD_REQUEST);
        res.render(Json(serde_json::json!({
            "success": false,
            "message": "文件名无效"
        })));
        return;
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

    match tokio::fs::remove_file(&file_path).await {
        Ok(_) => {
            res.render(Json(serde_json::json!({
                "success": true,
                "message": "文件删除成功"
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "success": false,
                "message": format!("删除文件失败: {}", e)
            })));
        }
    }
}

/// 重命名文件处理函数
///
/// 根据文件名重命名指定文件，如果新文件名已存在则自动添加时间戳后缀
///
/// # 参数
///
/// - `req`: HTTP 请求对象
/// - `res`: HTTP 响应对象
#[handler]
pub async fn rename_file(req: &mut Request, res: &mut Response) {
    let config = get_config();

    let old_name: String = match req.param("id") {
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

    if validate_filename(&old_name).is_err() {
        res.status_code(StatusCode::BAD_REQUEST);
        res.render(Json(serde_json::json!({
            "success": false,
            "message": "文件名无效"
        })));
        return;
    }

    let rename_request: RenameRequest = match req.parse_json().await {
        Ok(r) => r,
        Err(_) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "success": false,
                "message": "无效的请求体"
            })));
            return;
        }
    };

    let new_name = rename_request.new_name.trim();
    match validate_filename(new_name) {
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

    let old_path = config.storage_path.join(&old_name);
    if !tokio::fs::try_exists(&old_path).await.unwrap_or(false) {
        res.status_code(StatusCode::NOT_FOUND);
        res.render(Json(serde_json::json!({
            "success": false,
            "message": "文件不存在"
        })));
        return;
    }

    let unique_new_name = match generate_unique_filename(&config.storage_path, new_name).await {
        Ok(name) => name,
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "success": false,
                "message": format!("生成文件名失败: {}", e)
            })));
            return;
        }
    };
    let new_path = config.storage_path.join(&unique_new_name);

    match tokio::fs::rename(&old_path, &new_path).await {
        Ok(_) => {
            res.render(Json(serde_json::json!({
                "success": true,
                "message": "文件重命名成功",
                "new_name": unique_new_name
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "success": false,
                "message": format!("重命名文件失败: {}", e)
            })));
        }
    }
}
