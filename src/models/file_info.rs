use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// 文件信息结构体
/// 
/// 用于存储和传输文件的元数据信息，
/// 包括文件名、大小、类型等
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    /// 文件唯一标识符（文件名）
    pub id: String,
    
    /// 原始文件名（与 id 相同）
    pub original_name: String,
    
    /// 文件大小（字节）
    pub size: u64,
    
    /// 文件 MIME 类型
    pub content_type: String,
    
    /// 文件上传时间
    pub created_at: DateTime<Utc>,
}

impl FileInfo {
    /// 创建新的文件信息实例
    /// 
    /// # 参数
    /// 
    /// - `original_name`: 原始文件名
    /// - `size`: 文件大小
    /// - `content_type`: 文件 MIME 类型
    /// 
    /// # 返回值
    /// 
    /// 返回新创建的 FileInfo 实例
    pub fn new(
        original_name: String,
        size: u64,
        content_type: String,
    ) -> Self {
        FileInfo {
            id: original_name.clone(),
            original_name,
            size,
            content_type,
            created_at: Utc::now(),
        }
    }

    /// 从文件路径和元数据创建文件信息
    /// 
    /// # 参数
    /// 
    /// - `path`: 文件路径
    /// - `metadata`: 文件元数据
    /// 
    /// # 返回值
    /// 
    /// 返回新创建的 FileInfo 实例
    pub fn from_path(path: &Path, metadata: &std::fs::Metadata) -> Self {
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let content_type = Self::guess_content_type(path);
        
        let created_at = metadata.created().ok().map(|t| {
            chrono::DateTime::<chrono::Utc>::from(t)
        }).unwrap_or_else(Utc::now);

        FileInfo {
            id: file_name.clone(),
            original_name: file_name,
            size: metadata.len(),
            content_type,
            created_at,
        }
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
    fn guess_content_type(path: &Path) -> String {
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
}

/// 生成唯一的文件名
/// 
/// 如果目标路径已存在同名文件，则添加时间戳后缀
/// 
/// # 参数
/// 
/// - `storage_path`: 存储目录路径
/// - `original_name`: 原始文件名
/// 
/// # 返回值
/// 
/// 返回唯一的文件名
pub fn generate_unique_filename(storage_path: &Path, original_name: &str) -> String {
    let file_path = storage_path.join(original_name);
    
    if !file_path.exists() {
        return original_name.to_string();
    }
    
    let (stem, extension) = if let Some(dot_pos) = original_name.rfind('.') {
        (&original_name[..dot_pos], &original_name[dot_pos..])
    } else {
        (original_name, "")
    };
    
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let new_name = format!("{}_{}{}", stem, timestamp, extension);
    
    let new_path = storage_path.join(&new_name);
    if new_path.exists() {
        let mut counter = 1;
        loop {
            let unique_name = format!("{}_{}_{}", stem, timestamp, counter);
            let final_name = if extension.is_empty() {
                unique_name
            } else {
                format!("{}{}", unique_name, extension)
            };
            
            if !storage_path.join(&final_name).exists() {
                return final_name;
            }
            counter += 1;
        }
    }
    
    new_name
}

/// 文件列表响应结构体
/// 
/// 用于返回文件列表查询结果
#[derive(Debug, Serialize)]
pub struct FileListResponse {
    /// 文件列表
    pub files: Vec<FileInfo>,
    
    /// 文件总数
    pub total: usize,
}

impl FileListResponse {
    /// 创建新的文件列表响应
    /// 
    /// # 参数
    /// 
    /// - `files`: 文件列表
    /// 
    /// # 返回值
    /// 
    /// 返回新创建的 FileListResponse 实例
    pub fn new(files: Vec<FileInfo>) -> Self {
        let total = files.len();
        FileListResponse { files, total }
    }
}

/// 上传响应结构体
/// 
/// 用于返回文件上传结果
#[derive(Debug, Serialize)]
pub struct UploadResponse {
    /// 是否成功
    pub success: bool,
    
    /// 响应消息
    pub message: String,
    
    /// 文件信息（上传成功时返回）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<FileInfo>,
}

impl UploadResponse {
    /// 创建成功的上传响应
    /// 
    /// # 参数
    /// 
    /// - `file`: 上传的文件信息
    /// 
    /// # 返回值
    /// 
    /// 返回成功响应实例
    pub fn success(file: FileInfo) -> Self {
        UploadResponse {
            success: true,
            message: "文件上传成功".to_string(),
            file: Some(file),
        }
    }

    /// 创建失败的上传响应
    /// 
    /// # 参数
    /// 
    /// - `message`: 错误消息
    /// 
    /// # 返回值
    /// 
    /// 返回失败响应实例
    pub fn error(message: String) -> Self {
        UploadResponse {
            success: false,
            message,
            file: None,
        }
    }
}
