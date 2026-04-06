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
    pub fn new(original_name: String, size: u64, content_type: String) -> Self {
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
        let created_at = metadata
            .created()
            .ok()
            .map(chrono::DateTime::<chrono::Utc>::from)
            .unwrap_or_else(Utc::now);

        FileInfo {
            id: file_name.clone(),
            original_name: file_name,
            size: metadata.len(),
            content_type,
            created_at,
        }
    }

    /// 根据文件扩展名猜测 MIME 类型
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
        }
        .to_string()
    }
}

/// 文件列表响应结构体
#[derive(Debug, Serialize)]
pub struct FileListResponse {
    /// 文件列表
    pub files: Vec<FileInfo>,

    /// 文件总数
    pub total: usize,
}

impl FileListResponse {
    pub fn new(files: Vec<FileInfo>) -> Self {
        let total = files.len();
        FileListResponse { files, total }
    }
}

/// 上传响应结构体
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
    pub fn success(file: FileInfo) -> Self {
        UploadResponse {
            success: true,
            message: "文件上传成功".to_string(),
            file: Some(file),
        }
    }

    pub fn error(message: String) -> Self {
        UploadResponse {
            success: false,
            message,
            file: None,
        }
    }
}

/// 文件重命名请求结构体
#[derive(Debug, Deserialize)]
pub struct RenameRequest {
    /// 新的文件名
    pub new_name: String,
}
