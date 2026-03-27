use std::sync::Arc;

use crate::{pb::order, infrastructure::repository::order_repository::OrderRepository};

#[tonic::async_trait]
pub trait OrderService: Send + Sync + 'static {
    async fn create_order(
        &self,
        req: order::CreateOrderRequest,
    ) -> Result<order::Order, tonic::Status>;

    async fn get_order(&self, order_id: String) -> Result<order::Order, tonic::Status>;

    async fn list_orders(
        &self,
        req: order::ListOrdersRequest,
    ) -> Result<order::OrderList, tonic::Status>;
}

pub struct OrderServiceImpl<R: OrderRepository> {
    repo: Arc<R>,
}

impl<R: OrderRepository> OrderServiceImpl<R> {
    pub fn new(repo: Arc<R>) -> Arc<Self> {
        Arc::new(Self { repo })
    }
}

#[tonic::async_trait]
impl<R: OrderRepository> OrderService for OrderServiceImpl<R> {
    async fn create_order(
        &self,
        req: order::CreateOrderRequest,
    ) -> Result<order::Order, tonic::Status> {
        if req.user_id.trim().is_empty() {
            return Err(tonic::Status::invalid_argument("user_id 不能为空"));
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| tonic::Status::internal("时间异常"))?
            .as_secs() as i64;

        let order_id = format!("o_{}_{}", req.user_id, now);

        let o = order::Order {
            order_id,
            user_id: req.user_id,
            image_urls: req.image_urls,
            total_amount: 100,
            paid_amount: 0,
            status: order::OrderStatus::Created as i32,
            layout_type: req.layout_type,
            frame_type: req.frame_type,
            address_id: req.address_id,
            logistics_no: "".to_string(),
            created_at: now,
            updated_at: now,
        };

        self.repo.insert(o).await
    }

    async fn get_order(&self, order_id: String) -> Result<order::Order, tonic::Status> {
        let o = self
            .repo
            .get(&order_id)
            .await?
            .ok_or_else(|| tonic::Status::not_found("订单不存在"))?;
        Ok(o)
    }

    async fn list_orders(
        &self,
        req: order::ListOrdersRequest,
    ) -> Result<order::OrderList, tonic::Status> {
        if req.user_id.trim().is_empty() {
            return Err(tonic::Status::invalid_argument("user_id 不能为空"));
        }
        self.repo.list_by_user(&req.user_id, req.page).await
    }
}

