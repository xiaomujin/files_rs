use salvo::prelude::*;
use serde::Deserialize;
use crate::config::get_config;
use crate::models::{FileInfo, FileListResponse};
use uuid::Uuid;

/// 文件重命名请求结构体
/// 
/// 用于接收客户端发送的文件重命名请求
#[derive(Debug, Deserialize)]
pub struct RenameRequest {
    /// 新的文件名（显示名称）
    pub new_name: String,
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
    if !storage_path.exists() {
        res.render(Json(FileListResponse::new(files)));
        return;
    }
    
    if let Ok(entries) = std::fs::read_dir(storage_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if !path.is_file() {
                continue;
            }
            
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            
            let file_name = match path.file_name() {
                Some(n) => n.to_string_lossy().to_string(),
                None => continue,
            };
            
            let id = if let Some(dot_pos) = file_name.find('.') {
                &file_name[..dot_pos]
            } else {
                &file_name
            };
            
            // let uuid = match Uuid::parse_str(id) {
            //     Ok(u) => u,
            //     Err(_) => continue,
            // };
            
            let file_info = FileInfo {
                id: id.parse().unwrap(),
                original_name: file_name.clone(),
                stored_name: file_name,
                storage_path: storage_path.to_string_lossy().to_string(),
                size: metadata.len(),
                content_type: guess_content_type(&path),
                created_at: metadata.created().ok().map(|t| {
                    chrono::DateTime::<chrono::Utc>::from(t)
                }).unwrap_or_else(chrono::Utc::now),
            };
            
            files.push(file_info);
        }
    }
    
    files.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    
    res.render(Json(FileListResponse::new(files)));
}

/// 根据文件扩展名猜测 MIME 类型
/// 
/// # 参数
/// 
/// - `path`: 文件路径
/// 
/// # 返回值
/// 
/// 返回猜测的 MIME 类型字符串
fn guess_content_type(path: &std::path::Path) -> String {
    match path.extension().and_then(|e| e.to_str()) {
        Some("txt") => "text/plain",
        Some("html") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("json") => "application/json",
        Some("xml") => "application/xml",
        Some("pdf") => "application/pdf",
        Some("zip") => "application/zip",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("mp3") => "audio/mpeg",
        Some("mp4") => "video/mp4",
        Some("doc") => "application/msword",
        Some("docx") => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        Some("xls") => "application/vnd.ms-excel",
        Some("xlsx") => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        _ => "application/octet-stream",
    }.to_string()
}

/// 删除文件处理函数
/// 
/// 根据文件 ID 删除指定文件
/// 
/// # 参数
/// 
/// - `req`: HTTP 请求对象
/// - `res`: HTTP 响应对象
#[handler]
pub async fn delete_file(req: &mut Request, res: &mut Response) {
    let config = get_config();
    
    let file_id: String = match req.param("id") {
        Some(id) => id,
        None => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "success": false,
                "message": "缺少文件 ID"
            })));
            return;
        }
    };
    
    if Uuid::parse_str(&file_id).is_err() {
        res.status_code(StatusCode::BAD_REQUEST);
        res.render(Json(serde_json::json!({
            "success": false,
            "message": "无效的文件 ID 格式"
        })));
        return;
    }
    
    let storage_path = &config.storage_path;
    let mut found_file: Option<std::path::PathBuf> = None;
    
    if let Ok(entries) = std::fs::read_dir(storage_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(file_name) = path.file_name() {
                let name = file_name.to_string_lossy();
                if name.starts_with(&file_id) {
                    found_file = Some(path);
                    break;
                }
            }
        }
    }
    
    let file_path = match found_file {
        Some(p) => p,
        None => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "success": false,
                "message": "文件不存在"
            })));
            return;
        }
    };
    
    match std::fs::remove_file(&file_path) {
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
/// 根据文件 ID 重命名指定文件，保持 UUID 前缀和扩展名不变
/// 
/// # 参数
/// 
/// - `req`: HTTP 请求对象
/// - `res`: HTTP 响应对象
/// 
/// # 处理流程
/// 
/// 1. 从路径参数获取文件 ID
/// 2. 从请求体获取新文件名
/// 3. 验证 UUID 格式
/// 4. 查找文件（通过 UUID 前缀匹配）
/// 5. 构造新文件名（保持 UUID 和扩展名不变）
/// 6. 检查文件名冲突
/// 7. 执行重命名操作
#[handler]
pub async fn rename_file(req: &mut Request, res: &mut Response) {
    let config = get_config();
    
    let file_id: String = match req.param("id") {
        Some(id) => id,
        None => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "success": false,
                "message": "缺少文件 ID"
            })));
            return;
        }
    };
    
    if Uuid::parse_str(&file_id).is_err() {
        res.status_code(StatusCode::BAD_REQUEST);
        res.render(Json(serde_json::json!({
            "success": false,
            "message": "无效的文件 ID 格式"
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
    if new_name.is_empty() {
        res.status_code(StatusCode::BAD_REQUEST);
        res.render(Json(serde_json::json!({
            "success": false,
            "message": "文件名不能为空"
        })));
        return;
    }
    
    if new_name.contains('/') || new_name.contains('\\') || new_name.contains("..") {
        res.status_code(StatusCode::BAD_REQUEST);
        res.render(Json(serde_json::json!({
            "success": false,
            "message": "文件名包含非法字符"
        })));
        return;
    }
    
    let storage_path = &config.storage_path;
    let mut found_file: Option<std::path::PathBuf> = None;
    let mut file_extension: Option<String> = None;
    
    if let Ok(entries) = std::fs::read_dir(storage_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(file_name) = path.file_name() {
                let name = file_name.to_string_lossy();
                if name.starts_with(&file_id) {
                    let ext = if let Some(pos) = name.rfind('.') {
                        Some(name[pos..].to_string())
                    } else {
                        None
                    };
                    file_extension = ext;
                    found_file = Some(path);
                    break;
                }
            }
        }
    }
    
    let old_path = match found_file {
        Some(p) => p,
        None => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "success": false,
                "message": "文件不存在"
            })));
            return;
        }
    };
    
    let extension = file_extension.unwrap_or_default();
    let new_stored_name_with_ext = format!("{}_{}{}", file_id, new_name, extension);
    
    let new_path = storage_path.join(&new_stored_name_with_ext);
    
    if new_path.exists() && new_path != old_path {
        res.status_code(StatusCode::CONFLICT);
        res.render(Json(serde_json::json!({
            "success": false,
            "message": "目标文件名已存在"
        })));
        return;
    }
    
    match std::fs::rename(&old_path, &new_path) {
        Ok(_) => {
            res.render(Json(serde_json::json!({
                "success": true,
                "message": "文件重命名成功",
                "new_name": new_name,
                "stored_name": new_stored_name_with_ext
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
