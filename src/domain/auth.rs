// src/domain/auth.rs
#[derive(Clone, Debug)]
pub struct CurrentUser {
    pub user_id: u64,
    pub email: String,
}