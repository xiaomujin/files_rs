use salvo::prelude::*;
use serde::Serialize;

use crate::utils::FilenameValidationError;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub code: u32,
    pub message: String,
    pub data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: Some(data),
        }
    }
}

impl ApiResponse<()> {
    pub fn ok_empty() -> Self {
        ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: None,
        }
    }
}

#[async_trait]
impl<T: Serialize + Send> Writer for ApiResponse<T> {
    async fn write(self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        res.render(Json(self));
    }
}

#[derive(Debug)]
pub enum ApiError {
    // 100-199 General
    MissingParam,
    InvalidBody(String),
    InvalidContentType,
    // 200-299 File Validation
    FilenameEmpty,
    FilenameInvalidChars,
    FileTooLarge,
    // 300-399 File Operations
    FileNotFound,
    SaveFailed(String),
    DeleteFailed(String),
    RenameFailed(String),
    GenerateNameFailed(String),
}

impl ApiError {
    pub fn code(&self) -> u32 {
        match self {
            ApiError::MissingParam => 100,
            ApiError::InvalidBody(_) => 101,
            ApiError::InvalidContentType => 102,
            ApiError::FilenameEmpty => 200,
            ApiError::FilenameInvalidChars => 201,
            ApiError::FileTooLarge => 202,
            ApiError::FileNotFound => 300,
            ApiError::SaveFailed(_) => 301,
            ApiError::DeleteFailed(_) => 302,
            ApiError::RenameFailed(_) => 303,
            ApiError::GenerateNameFailed(_) => 304,
        }
    }

    pub fn message(&self) -> String {
        match self {
            ApiError::MissingParam => "缺少必要参数".to_string(),
            ApiError::InvalidBody(detail) => format!("无效的请求体: {}", detail),
            ApiError::InvalidContentType => "无效的 Content-Type".to_string(),
            ApiError::FilenameEmpty => "文件名不能为空".to_string(),
            ApiError::FilenameInvalidChars => "文件名包含非法字符".to_string(),
            ApiError::FileTooLarge => "文件过大".to_string(),
            ApiError::FileNotFound => "文件不存在".to_string(),
            ApiError::SaveFailed(detail) => format!("保存文件失败: {}", detail),
            ApiError::DeleteFailed(detail) => format!("删除文件失败: {}", detail),
            ApiError::RenameFailed(detail) => format!("重命名文件失败: {}", detail),
            ApiError::GenerateNameFailed(detail) => format!("生成文件名失败: {}", detail),
        }
    }
}

#[async_trait]
impl Writer for ApiError {
    async fn write(self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        #[derive(Serialize)]
        struct ErrorBody {
            code: u32,
            message: String,
            data: Option<()>,
        }
        res.render(Json(ErrorBody {
            code: self.code(),
            message: self.message(),
            data: None,
        }));
    }
}

impl From<FilenameValidationError> for ApiError {
    fn from(e: FilenameValidationError) -> Self {
        match e {
            FilenameValidationError::Empty => ApiError::FilenameEmpty,
            FilenameValidationError::InvalidChars => ApiError::FilenameInvalidChars,
        }
    }
}
