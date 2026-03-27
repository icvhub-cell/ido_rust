use crate::pb::{common, order};
use sqlx::{FromRow, MySqlPool};

#[tonic::async_trait]
pub trait OrderRepository: Send + Sync + 'static {
    async fn insert(&self, order: order::Order) -> Result<order::Order, tonic::Status>;
    async fn get(&self, order_id: &str) -> Result<Option<order::Order>, tonic::Status>;
    async fn list_by_user(
        &self,
        user_id: &str,
        page: Option<common::PageRequest>,
    ) -> Result<order::OrderList, tonic::Status>;
}

#[derive(Clone)]
pub struct MySqlOrderRepository {
    pool: MySqlPool,
}

#[derive(Debug, FromRow)]
struct OrderRow {
    order_id: String,
    user_id: String,
    image_urls: String,
    total_amount: i64,
    paid_amount: i64,
    status: i32,
    layout_type: String,
    frame_type: String,
    address_id: String,
    logistics_no: String,
    created_at: i64,
    updated_at: i64,
}

impl OrderRow {
    fn into_proto(self) -> order::Order {
        let image_urls = serde_json::from_str::<Vec<String>>(&self.image_urls)
            .unwrap_or_default();

        order::Order {
            order_id: self.order_id,
            user_id: self.user_id,
            image_urls,
            total_amount: self.total_amount,
            paid_amount: self.paid_amount,
            status: self.status,
            layout_type: self.layout_type,
            frame_type: self.frame_type,
            address_id: self.address_id,
            logistics_no: self.logistics_no,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl MySqlOrderRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[tonic::async_trait]
impl OrderRepository for MySqlOrderRepository {
    async fn insert(&self, order: order::Order) -> Result<order::Order, tonic::Status> {
        let image_urls = serde_json::to_string(&order.image_urls)
            .map_err(|e| tonic::Status::internal(format!("序列化图片地址失败: {e}")))?;

        sqlx::query(
            r#"
            INSERT INTO orders (
                order_id,
                user_id,
                image_urls,
                total_amount,
                paid_amount,
                status,
                layout_type,
                frame_type,
                address_id,
                logistics_no,
                created_at,
                updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&order.order_id)
        .bind(&order.user_id)
        .bind(image_urls)
        .bind(order.total_amount)
        .bind(order.paid_amount)
        .bind(order.status)
        .bind(&order.layout_type)
        .bind(&order.frame_type)
        .bind(&order.address_id)
        .bind(&order.logistics_no)
        .bind(order.created_at)
        .bind(order.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| tonic::Status::internal(format!("创建订单失败: {e}")))?;

        Ok(order)
    }

    async fn get(&self, order_id: &str) -> Result<Option<order::Order>, tonic::Status> {
        let row = sqlx::query_as::<_, OrderRow>(
            r#"
            SELECT
                order_id,
                user_id,
                image_urls,
                total_amount,
                paid_amount,
                status,
                layout_type,
                frame_type,
                address_id,
                logistics_no,
                created_at,
                updated_at
            FROM orders
            WHERE order_id = ?
            "#,
        )
        .bind(order_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| tonic::Status::internal(format!("查询订单失败: {e}")))?;

        Ok(row.map(OrderRow::into_proto))
    }

    async fn list_by_user(
        &self,
        user_id: &str,
        page: Option<common::PageRequest>,
    ) -> Result<order::OrderList, tonic::Status> {
        let page_no = page.as_ref().map(|p| p.page.max(1)).unwrap_or(1);
        let page_size = page.as_ref().map(|p| p.page_size.max(1)).unwrap_or(10);
        let offset = ((page_no - 1) * page_size) as i64;

        let (total,): (i64,) = sqlx::query_as("SELECT COUNT(*) AS total FROM orders WHERE user_id = ?")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| tonic::Status::internal(format!("统计订单失败: {e}")))?;

        let rows = sqlx::query_as::<_, OrderRow>(
            r#"
            SELECT
                order_id,
                user_id,
                image_urls,
                total_amount,
                paid_amount,
                status,
                layout_type,
                frame_type,
                address_id,
                logistics_no,
                created_at,
                updated_at
            FROM orders
            WHERE user_id = ?
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(user_id)
        .bind(page_size as i64)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| tonic::Status::internal(format!("分页查询订单失败: {e}")))?;

        Ok(order::OrderList {
            list: rows.into_iter().map(OrderRow::into_proto).collect(),
            page: Some(common::PageResponse {
                page: page_no,
                page_size,
                total,
            }),
        })
    }
}
