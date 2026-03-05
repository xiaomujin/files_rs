use std::path::PathBuf;

/// 应用程序配置结构体
/// 
/// 该结构体用于存储应用程序的全局配置信息，
/// 支持通过环境变量进行配置
#[derive(Debug, Clone)]
pub struct Config {
    /// 文件存储路径
    /// 默认值为 "./uploads"
    pub storage_path: PathBuf,
}

impl Config {
    /// 从环境变量加载配置
    /// 
    /// 如果环境变量未设置，则使用默认值：
    /// - STORAGE_PATH: 默认为 "./uploads"
    /// 
    /// # 返回值
    /// 
    /// 返回初始化后的 Config 实例
    pub fn from_env() -> Self {
        let storage_path = std::env::var("STORAGE_PATH")
            .unwrap_or_else(|_| "./uploads".to_string());
        
        Config {
            storage_path: PathBuf::from(storage_path),
        }
    }

    /// 创建默认配置
    /// 
    /// 使用默认值初始化配置
    /// 
    /// # 返回值
    /// 
    /// 返回使用默认值的 Config 实例
    pub fn default() -> Self {
        Config {
            storage_path: PathBuf::from("./uploads"),
        }
    }

    /// 确保存储目录存在
    /// 
    /// 如果存储目录不存在，将自动创建
    /// 
    /// # 错误
    /// 
    /// 如果创建目录失败，返回 IO 错误
    pub fn ensure_storage_dir(&self) -> std::io::Result<()> {
        if !self.storage_path.exists() {
            std::fs::create_dir_all(&self.storage_path)?;
        }
        Ok(())
    }
}

/// 全局配置实例
/// 
/// 使用 lazy_static 宏创建全局配置实例，
/// 在程序启动时从环境变量加载配置
pub static CONFIG: std::sync::OnceLock<Config> = std::sync::OnceLock::new();

/// 获取全局配置实例
/// 
/// 如果全局配置未初始化，将从环境变量加载配置
/// 
/// # 返回值
/// 
/// 返回全局配置的引用
pub fn get_config() -> &'static Config {
    CONFIG.get_or_init(|| {
        let config = Config::from_env();
        if let Err(e) = config.ensure_storage_dir() {
            eprintln!("创建存储目录失败: {}", e);
        }
        config
    })
}
