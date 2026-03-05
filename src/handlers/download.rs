use salvo::prelude::*;
use salvo::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use salvo::http::ResBody;
use crate::config::get_config;
use uuid::Uuid;
use bytes::Bytes;

/// 文件下载处理函数
/// 
/// 根据文件 ID 下载文件，支持通过路径参数指定文件
/// 
/// # 参数
/// 
/// - `req`: HTTP 请求对象
/// - `res`: HTTP 响应对象
/// - `depot`: 用于存储请求上下文的仓库
#[handler]
pub async fn download(req: &mut Request, res: &mut Response, _depot: &mut Depot) {
    let config = get_config();
    
    // 从路径参数获取文件 ID
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
    
    // 解析 UUID
    let _uuid = match Uuid::parse_str(&file_id) {
        Ok(u) => u,
        Err(_) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "success": false,
                "message": "无效的文件 ID 格式"
            })));
            return;
        }
    };
    
    // 查找文件（遍历存储目录查找匹配的文件）
    let storage_path = &config.storage_path;
    if !storage_path.exists() {
        res.status_code(StatusCode::NOT_FOUND);
        res.render(Json(serde_json::json!({
            "success": false,
            "message": "存储目录不存在"
        })));
        return;
    }
    
    // 查找以该 UUID 开头的文件
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
    
    // 读取文件内容
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
    
    // 获取文件名（去除 UUID 前缀）
    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("download");
    
    // 设置响应头
    res.add_header(CONTENT_TYPE, "application/octet-stream", true)
        .unwrap();
    res.add_header(
        CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", file_name),
        true,
    )
    .unwrap();
    
    // 返回文件内容
    res.body = ResBody::Once(Bytes::from(file_data));
}
