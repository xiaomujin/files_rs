use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub original_name: String,
    pub size: u64,
    pub content_type: String,
    pub created_at: DateTime<Utc>,
}

impl FileInfo {
    pub fn new(original_name: String, size: u64, content_type: String) -> Self {
        FileInfo {
            id: original_name.clone(),
            original_name,
            size,
            content_type,
            created_at: Utc::now(),
        }
    }

    pub fn from_path(path: &Path, metadata: &std::fs::Metadata) -> Self {
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let content_type = Self::guess_content_type(path);
        let created_at = metadata
            .created()
            .ok()
            .map(DateTime::<Utc>::from)
            .unwrap_or_else(Utc::now);

        FileInfo {
            id: file_name.clone(),
            original_name: file_name,
            size: metadata.len(),
            content_type,
            created_at,
        }
    }

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

#[derive(Debug, Serialize)]
pub struct FileListData {
    pub files: Vec<FileInfo>,
    pub total: usize,
}

impl FileListData {
    pub fn new(files: Vec<FileInfo>) -> Self {
        let total = files.len();
        FileListData { files, total }
    }
}

#[derive(Debug, Serialize)]
pub struct RenameData {
    pub new_name: String,
}

#[derive(Debug, Deserialize)]
pub struct RenameRequest {
    pub new_name: String,
}
