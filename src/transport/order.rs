// transport/order.rs
#[derive(Debug)]
pub struct OrderController {
    service: Arc<OrderService>,
}

#[tonic::async_trait]
impl order::order_service_server::OrderService for OrderController {
    async fn create_order(
        &self,
        request: Request<CreateOrderRequest>,
    ) -> Result<Response<Order>, Status> {
        let req = request.into_inner();

        let order = self.service.create_order(req).await?;

        Ok(Response::new(order))
    }
}