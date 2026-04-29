use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 配置文件名
const CONFIG_FILE: &str = "config.json5";

/// 应用程序配置结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 服务端口号
    #[serde(default = "default_port")]
    pub port: u16,
    /// 文件存储路径
    #[serde(default = "default_storage_path")]
    pub storage_path: PathBuf,
}

fn default_port() -> u16 {
    3000
}

fn default_storage_path() -> PathBuf {
    PathBuf::from("./uploads")
}

impl Default for Config {
    fn default() -> Self {
        Config {
            port: default_port(),
            storage_path: default_storage_path(),
        }
    }
}

impl Config {
    /// 获取可执行文件同目录下的 config.json5 路径
    fn config_path() -> PathBuf {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));
        exe_dir.join(CONFIG_FILE)
    }

    /// 从 config.json5 加载配置，不存在则创建默认配置文件
    pub fn load() -> Self {
        let path = Self::config_path();

        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => match json5::from_str::<Config>(&content) {
                    Ok(config) => {
                        println!("已加载配置文件: {:?}", path);
                        return config;
                    }
                    Err(e) => {
                        eprintln!("解析配置文件失败: {}，将使用默认配置", e);
                    }
                },
                Err(e) => {
                    eprintln!("读取配置文件失败: {}，将使用默认配置", e);
                }
            }
        }

        // 文件不存在或解析失败，创建默认配置
        let config = Config::default();
        if let Err(e) = config.save(&path) {
            eprintln!("创建默认配置文件失败: {}", e);
        } else {
            println!("已创建默认配置文件: {:?}", path);
        }
        config
    }

    /// 将配置保存到指定路径（带注释的 JSON5 格式）
    fn save(&self, path: &PathBuf) -> std::io::Result<()> {
        let content = format!(
            r#"{{
    // 服务端口号
    port: {},
    // 文件上传存储路径
    storage_path: "{}",
}}"#,
            self.port,
            self.storage_path.to_string_lossy(),
        );
        std::fs::write(path, content)
    }

    /// 确保存储目录存在
    pub fn ensure_storage_dir(&self) -> std::io::Result<()> {
        if !self.storage_path.exists() {
            std::fs::create_dir_all(&self.storage_path)?;
        }
        Ok(())
    }
}

/// 全局配置实例
pub static CONFIG: std::sync::OnceLock<Config> = std::sync::OnceLock::new();

/// 获取全局配置实例
pub fn get_config() -> &'static Config {
    CONFIG.get_or_init(|| {
        let config = Config::load();
        if let Err(e) = config.ensure_storage_dir() {
            eprintln!("创建存储目录失败: {}", e);
        }
        config
    })
}
