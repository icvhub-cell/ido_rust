use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::{
    pb::order,
    service::order_service::OrderService,
};

pub struct OrderController {
    service: Arc<dyn OrderService>,
}

impl OrderController {
    pub fn new(service: Arc<dyn OrderService>) -> Self {
        Self { service }
    }
}

#[tonic::async_trait]
impl order::order_service_server::OrderService for OrderController {
    async fn create_order(
        &self,
        request: Request<order::CreateOrderRequest>,
    ) -> Result<Response<order::Order>, Status> {
        let req = request.into_inner();
        let o = self.service.create_order(req).await?;
        Ok(Response::new(o))
    }

    async fn get_order(
        &self,
        request: Request<order::GetOrderRequest>,
    ) -> Result<Response<order::Order>, Status> {
        let req = request.into_inner();
        let o = self.service.get_order(req.order_id).await?;
        Ok(Response::new(o))
    }

    async fn list_orders(
        &self,
        request: Request<order::ListOrdersRequest>,
    ) -> Result<Response<order::OrderList>, Status> {
        let req = request.into_inner();
        let list = self.service.list_orders(req).await?;
        Ok(Response::new(list))
    }
}