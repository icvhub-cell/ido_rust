use tonic::{Request, Status};

pub fn auth_interceptor(req: Request<()>) -> Result<Request<()>, Status> {
    // 校验 JWT
    Ok(req)
}