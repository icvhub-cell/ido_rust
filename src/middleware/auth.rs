use crate::domain::auth::CurrentUser;
use crate::infrastructure::cache::redis::RedisRepo;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use tokio::runtime::Handle;
use tonic::{Request, Status};

const AUTHORIZATION: &str = "authorization";
const BEARER_PREFIX: &str = "Bearer ";

#[derive(Debug, Deserialize)]
struct JwtClaims {
    sub: String,
    email: String,
    exp: usize,
}

pub fn auth_interceptor(mut req: Request<()>) -> Result<Request<()>, Status> {
    // 读取并校验 Authorization: Bearer <token>
    let metadata = req.metadata();
    let auth_header = metadata
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| Status::unauthenticated("Missing authorization header"))?;

    let token = auth_header
        .strip_prefix(BEARER_PREFIX)
        .ok_or_else(|| Status::unauthenticated("Invalid authorization scheme"))?
        .trim();

    if token.is_empty() {
        return Err(Status::unauthenticated("Empty token"));
    }

    // 校验 JWT，并提取当前用户
    let user = parse_token(token)?;

    if is_blacklisted(token)? {
        return Err(Status::unauthenticated("Token is blacklisted"));
    }

    // 将用户写入 request 上下文，供后续 handler 读取
    req.extensions_mut().insert(user);
    Ok(req)
}

fn parse_token(token: &str) -> Result<CurrentUser, Status> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    let token_data = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_bytes()),
        &validation,
    )
    .map_err(|_| Status::unauthenticated("Invalid or expired token"))?;

    let user_id = token_data
        .claims
        .sub
        .parse::<u64>()
        .map_err(|_| Status::unauthenticated("Invalid user id in token"))?;

    Ok(CurrentUser {
        user_id,
        email: token_data.claims.email,
    })
}

fn jwt_secret() -> String {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".to_string())
}

fn redis_repo() -> Result<&'static RedisRepo, Status> {
    RedisRepo::global().ok_or_else(|| {
        Status::internal("RedisRepo is not initialized, please init in main first")
    })
}

fn is_blacklisted(token: &str) -> Result<bool, Status> {
    let repo = redis_repo()?;

    tokio::task::block_in_place(|| {
        Handle::current()
            .block_on(repo.is_token_blacklisted(token))
            .map_err(|e| Status::internal(format!("Redis check failed: {e}")))
    })
}