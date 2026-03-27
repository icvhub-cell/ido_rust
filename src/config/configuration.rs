use config::{Config, Environment, File};
use crate::config::settings::Settings;

pub fn load_config() -> Settings {
    // 加载 .env
    dotenvy::dotenv().ok();

    let builder = Config::builder()
        // 默认配置文件
        .add_source(File::with_name("config/default").required(false))
        // 环境变量（SERVER_PORT=50051）
        .add_source(Environment::default().separator("_"));

    builder
        .build()
        .expect("Failed to build config")
        .try_deserialize::<Settings>()
        .expect("Failed to deserialize config")
}