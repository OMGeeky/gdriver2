use serde::{Deserialize, Serialize};

#[tarpc::service]
pub trait GDriverService {
    async fn do_something2(
        req: BackendActionRequest,
    ) -> std::result::Result<String, BackendActionError>;
}
#[derive(Debug, Serialize, Deserialize)]
pub enum BackendActionRequest {
    ShutdownGracefully,
    UpdateChanges,
    Ping,
    RunLong,
    StartLong,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum BackendActionError {
    Unknown,
    CouldNotComplete,
}
