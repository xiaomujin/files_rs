use chrono::Utc;
use std::path::Path;

pub async fn generate_unique_filename(
    storage_path: &Path,
    original_name: &str,
) -> std::io::Result<String> {
    let file_path = storage_path.join(original_name);
    if !tokio::fs::try_exists(&file_path).await? {
        return Ok(original_name.to_string());
    }

    let (stem, extension) = if let Some(dot_pos) = original_name.rfind('.') {
        (&original_name[..dot_pos], &original_name[dot_pos..])
    } else {
        (original_name, "")
    };

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let new_name = format!("{}_{}{}", stem, timestamp, extension);

    if !tokio::fs::try_exists(&storage_path.join(&new_name)).await? {
        return Ok(new_name);
    }

    let mut counter = 1;
    loop {
        let unique_name = format!("{}_{}_{}", stem, timestamp, counter);
        let final_name = if extension.is_empty() {
            unique_name
        } else {
            format!("{}{}", unique_name, extension)
        };

        if !tokio::fs::try_exists(&storage_path.join(&final_name)).await? {
            return Ok(final_name);
        }
        counter += 1;
    }
}
