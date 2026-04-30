use crate::config::get_config;
use crate::models::{FileInfo, FileListData, RenameData, RenameRequest};
use crate::response::{ApiError, ApiResponse};
use crate::services::generate_unique_filename;
use crate::utils::validate_filename;
use salvo::fs::NamedFile;
use salvo::http::ParseError;
use salvo::prelude::*;

#[handler]
pub async fn handle_upload(req: &mut Request) -> Result<ApiResponse<FileInfo>, ApiError> {
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
        Ok(None) => return Err(ApiError::MissingParam),
        Err(ParseError::PayloadTooLarge) => return Err(ApiError::FileTooLarge),
        Err(ParseError::InvalidContentType | ParseError::NotMultipart | ParseError::NotFormData) => {
            return Err(ApiError::InvalidContentType);
        }
        Err(e) => return Err(ApiError::InvalidBody(e.to_string())),
    };

    let unique_name = generate_unique_filename(&config.storage_path, &original_name)
        .await
        .map_err(|e| ApiError::GenerateNameFailed(e.to_string()))?;

    let file_path = config.storage_path.join(&unique_name);

    tokio::fs::copy(&temp_path, &file_path)
        .await
        .map_err(|e| ApiError::SaveFailed(e.to_string()))?;

    let file_info = FileInfo::new(unique_name, size, content_type);
    Ok(ApiResponse::ok(file_info))
}

#[handler]
pub async fn handle_download(req: &mut Request, res: &mut Response) -> Result<(), ApiError> {
    let config = get_config();

    let file_name: String = req.param("id").ok_or(ApiError::MissingParam)?;
    validate_filename(&file_name)?;

    let file_path = config.storage_path.join(&file_name);
    if !tokio::fs::try_exists(&file_path).await.unwrap_or(false) {
        return Err(ApiError::FileNotFound);
    }

    let file = NamedFile::builder(&file_path)
        .attached_name(file_name)
        .content_type("application/octet-stream".parse().unwrap())
        .build()
        .await
        .map_err(|e| ApiError::SaveFailed(e.to_string()))?;

    file.send(req.headers(), res).await;
    Ok(())
}

#[handler]
pub async fn list_files() -> Result<ApiResponse<FileListData>, ApiError> {
    let config = get_config();
    let mut files = Vec::new();

    let storage_path = &config.storage_path;
    if !tokio::fs::try_exists(storage_path).await.unwrap_or(false) {
        return Ok(ApiResponse::ok(FileListData::new(files)));
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
    Ok(ApiResponse::ok(FileListData::new(files)))
}

#[handler]
pub async fn delete_file(req: &mut Request) -> Result<ApiResponse<()>, ApiError> {
    let config = get_config();

    let file_name: String = req.param("id").ok_or(ApiError::MissingParam)?;
    validate_filename(&file_name)?;

    let file_path = config.storage_path.join(&file_name);
    if !tokio::fs::try_exists(&file_path).await.unwrap_or(false) {
        return Err(ApiError::FileNotFound);
    }

    tokio::fs::remove_file(&file_path)
        .await
        .map_err(|e| ApiError::DeleteFailed(e.to_string()))?;

    Ok(ApiResponse::ok_empty())
}

#[handler]
pub async fn rename_file(req: &mut Request) -> Result<ApiResponse<RenameData>, ApiError> {
    let config = get_config();

    let old_name: String = req.param("id").ok_or(ApiError::MissingParam)?;
    validate_filename(&old_name)?;

    let rename_request: RenameRequest = req
        .parse_json()
        .await
        .map_err(|e| ApiError::InvalidBody(e.to_string()))?;

    let new_name = rename_request.new_name.trim().to_string();
    validate_filename(&new_name)?;

    let old_path = config.storage_path.join(&old_name);
    if !tokio::fs::try_exists(&old_path).await.unwrap_or(false) {
        return Err(ApiError::FileNotFound);
    }

    let unique_new_name = generate_unique_filename(&config.storage_path, &new_name)
        .await
        .map_err(|e| ApiError::GenerateNameFailed(e.to_string()))?;
    let new_path = config.storage_path.join(&unique_new_name);

    tokio::fs::rename(&old_path, &new_path)
        .await
        .map_err(|e| ApiError::RenameFailed(e.to_string()))?;

    Ok(ApiResponse::ok(RenameData { new_name: unique_new_name }))
}
