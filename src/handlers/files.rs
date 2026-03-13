use salvo::prelude::*;
use serde::Deserialize;
use crate::config::get_config;
use crate::models::{FileInfo, FileListResponse, generate_unique_filename};

/// 文件重命名请求结构体
/// 
/// 用于接收客户端发送的文件重命名请求
#[derive(Debug, Deserialize)]
pub struct RenameRequest {
    /// 新的文件名
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
    
    if file_name.is_empty() || file_name.contains('/') || file_name.contains('\\') || file_name.contains("..") {
        res.status_code(StatusCode::BAD_REQUEST);
        res.render(Json(serde_json::json!({
            "success": false,
            "message": "文件名无效"
        })));
        return;
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
/// 根据文件名重命名指定文件，如果新文件名已存在则自动添加时间戳后缀
/// 
/// # 参数
/// 
/// - `req`: HTTP 请求对象
/// - `res`: HTTP 响应对象
/// 
/// # 处理流程
/// 
/// 1. 从路径参数获取原文件名
/// 2. 从请求体获取新文件名
/// 3. 验证文件名合法性
/// 4. 检查原文件是否存在
/// 5. 生成唯一的新文件名
/// 6. 执行重命名操作
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
    
    if old_name.is_empty() || old_name.contains('/') || old_name.contains('\\') || old_name.contains("..") {
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
    
    let old_path = config.storage_path.join(&old_name);
    
    if !old_path.exists() {
        res.status_code(StatusCode::NOT_FOUND);
        res.render(Json(serde_json::json!({
            "success": false,
            "message": "文件不存在"
        })));
        return;
    }
    
    let unique_new_name = generate_unique_filename(&config.storage_path, new_name);
    let new_path = config.storage_path.join(&unique_new_name);
    
    match std::fs::rename(&old_path, &new_path) {
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
