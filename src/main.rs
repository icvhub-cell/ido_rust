use tonic::transport::Server;

mod pb;
mod config;
mod domain;
mod infrastructure;
mod interface;
mod middleware;
mod service;
use config::configuration::load_config;
use infrastructure::cache::redis::RedisRepo;
use std::net::SocketAddr;
use std::sync::Arc;
use tonic_health::server::health_reporter;
use crate::pb::order::order_service_server::OrderServiceServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = load_config();
    let redis_url = if settings.redis.password.is_empty() {
        format!("redis://{}:{}", settings.redis.host, settings.redis.port)
    } else {
        format!(
            "redis://:{}@{}:{}",
            settings.redis.password, settings.redis.host, settings.redis.port
        )
    };
    RedisRepo::init_global(&redis_url)?;

    let addr: SocketAddr = format!(
        "{}:{}",
        settings.server.host,
        settings.server.port
    )
    .parse()?;

    println!("🚀 Server running on {}", addr);

    let (_health_reporter, health_service) = health_reporter();

    let mysql_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        if settings.mysql.password.is_empty() {
            format!(
                "mysql://{}@{}:{}/{}",
                settings.mysql.username,
                settings.mysql.host,
                settings.mysql.port,
                settings.mysql.database
            )
        } else {
            format!(
                "mysql://{}:{}@{}:{}/{}",
                settings.mysql.username,
                settings.mysql.password,
                settings.mysql.host,
                settings.mysql.port,
                settings.mysql.database
            )
        }
    });

    let mysql_pool = infrastructure::db::mysql::init_db_pool(&mysql_url).await?;
    let order_repo = Arc::new(
        infrastructure::repository::order_repository::MySqlOrderRepository::new(mysql_pool),
    );
    let order_svc = service::order_service::OrderServiceImpl::new(order_repo);
    let order_controller = interface::grpc::order_handler::OrderController::new(order_svc);

    Server::builder()
        .add_service(health_service)
        .add_service(OrderServiceServer::with_interceptor(
            order_controller,
            middleware::auth::auth_interceptor,
        ))
        .serve(addr)
        .await?;

    Ok(())
}
