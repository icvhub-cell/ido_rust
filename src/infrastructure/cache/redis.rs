use deadpool_redis::{Config, Pool, Runtime};
use deadpool_redis::redis::AsyncCommands;
use std::sync::OnceLock;

pub type RedisPool = Pool;

#[derive(Clone)]
pub struct RedisRepo {
    pool: RedisPool,
}

static REDIS_REPO: OnceLock<RedisRepo> = OnceLock::new();

impl RedisRepo {
    pub const TOKEN_BLACKLIST_PREFIX: &'static str = "auth:blacklist:";

    /// 初始化 Redis 连接池
    pub fn new(redis_url: &str) -> anyhow::Result<Self> {
        let cfg = Config::from_url(redis_url);

        let pool = cfg.create_pool(Some(Runtime::Tokio1))?;

        Ok(Self { pool })
    }

    /// 启动时初始化全局 RedisRepo（只允许初始化一次）
    pub fn init_global(redis_url: &str) -> anyhow::Result<()> {
        let repo = Self::new(redis_url)?;
        REDIS_REPO
            .set(repo)
            .map_err(|_| anyhow::anyhow!("RedisRepo has already been initialized"))?;
        Ok(())
    }

    /// 获取启动时初始化好的全局 RedisRepo
    pub fn global() -> Option<&'static Self> {
        REDIS_REPO.get()
    }

    /// 获取底层连接（一般不直接用）
    pub async fn get_conn(
        &self,
    ) -> anyhow::Result<deadpool_redis::Connection> {
        let conn = self.pool.get().await?;
        Ok(conn)
    }

    /// SET
    pub async fn set<K, V>(&self, key: K, value: V) -> anyhow::Result<()>
    where
        K: ToString,
        V: ToString,
    {
        let mut conn = self.pool.get().await?;
        conn.set::<_, _, ()>(key.to_string(), value.to_string())
            .await?;
        Ok(())
    }

    /// GET
    pub async fn get<K>(&self, key: K) -> anyhow::Result<Option<String>>
    where
        K: ToString,
    {
        let mut conn = self.pool.get().await?;
        let val: Option<String> = conn.get(key.to_string()).await?;
        Ok(val)
    }

    /// SET 带过期时间（秒）
    pub async fn set_ex<K, V>(
        &self,
        key: K,
        value: V,
        ttl_secs: usize,
    ) -> anyhow::Result<()>
    where
        K: ToString,
        V: ToString,
    {
        let mut conn = self.pool.get().await?;
        conn.set_ex::<_, _, ()>(
            key.to_string(),
            value.to_string(),
            ttl_secs as u64,
        )
            .await?;
        Ok(())
    }

    /// DEL
    pub async fn del<K>(&self, key: K) -> anyhow::Result<()>
    where
        K: ToString,
    {
        let mut conn = self.pool.get().await?;
        conn.del::<_, ()>(key.to_string()).await?;
        Ok(())
    }

    /// 原子递增（常用于限流）
    pub async fn incr<K>(&self, key: K) -> anyhow::Result<i64>
    where
        K: ToString,
    {
        let mut conn = self.pool.get().await?;
        let val: i64 = conn.incr(key.to_string(), 1).await?;
        Ok(val)
    }

    /// 设置过期时间
    pub async fn expire<K>(
        &self,
        key: K,
        ttl_secs: i64,
    ) -> anyhow::Result<()>
    where
        K: ToString,
    {
        let mut conn = self.pool.get().await?;
        conn.expire::<_, ()>(key.to_string(), ttl_secs).await?;
        Ok(())
    }

    /// EXISTS
    pub async fn exists<K>(&self, key: K) -> anyhow::Result<bool>
    where
        K: ToString,
    {
        let mut conn = self.pool.get().await?;
        let exists: bool = conn.exists(key.to_string()).await?;
        Ok(exists)
    }

    /// 黑名单 token 的 Redis key
    pub fn token_blacklist_key(token: &str) -> String {
        format!("{}{}", Self::TOKEN_BLACKLIST_PREFIX, token)
    }

    /// 写入 token 黑名单，ttl 秒后自动过期
    pub async fn blacklist_token(
        &self,
        token: &str,
        ttl_secs: usize,
    ) -> anyhow::Result<()> {
        let key = Self::token_blacklist_key(token);
        self.set_ex(key, 1, ttl_secs).await
    }

    /// 判断 token 是否在黑名单
    pub async fn is_token_blacklisted(
        &self,
        token: &str,
    ) -> anyhow::Result<bool> {
        let key = Self::token_blacklist_key(token);
        self.exists(key).await
    }
}
