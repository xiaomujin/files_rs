use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 文件信息结构体
/// 
/// 用于存储和传输文件的元数据信息，
/// 包括文件名、存储路径、大小、类型等
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    /// 文件唯一标识符
    pub id: String,
    
    /// 原始文件名
    pub original_name: String,
    
    /// 服务器存储的文件名（UUID格式）
    pub stored_name: String,
    
    /// 文件存储路径
    pub storage_path: String,
    
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
    /// - `storage_path`: 文件存储路径
    /// - `size`: 文件大小
    /// - `content_type`: 文件 MIME 类型
    /// 
    /// # 返回值
    /// 
    /// 返回新创建的 FileInfo 实例
    pub fn new(
        original_name: String,
        storage_path: String,
        size: u64,
        content_type: String,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        // let stored_name = format!("{}{}", id, Self::get_extension(&original_name));
        let stored_name = original_name.clone();

        FileInfo {
            id,
            original_name,
            stored_name,
            storage_path,
            size,
            content_type,
            created_at: Utc::now(),
        }
    }

    /// 从文件名获取扩展名
    /// 
    /// # 参数
    /// 
    /// - `filename`: 文件名
    /// 
    /// # 返回值
    /// 
    /// 返回文件扩展名（包含点号），如果没有扩展名则返回空字符串
    fn get_extension(filename: &str) -> String {
        if let Some(pos) = filename.rfind('.') {
            filename[pos..].to_string()
        } else {
            String::new()
        }
    }

    /// 获取文件的完整存储路径
    /// 
    /// # 返回值
    /// 
    /// 返回文件在服务器上的完整路径
    pub fn full_path(&self) -> std::path::PathBuf {
        std::path::PathBuf::from(&self.storage_path).join(&self.stored_name)
    }
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
