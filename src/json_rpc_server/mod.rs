pub mod server;

enum ServerReturnType {
    Success,
    Failure(String),
}

impl jsonrpsee::IntoResponse for ServerReturnType {
    type Output = u64;

    fn into_response(self) -> jsonrpsee::ResponsePayload<'static, Self::Output> {
        match self {
            ServerReturnType::Success => jsonrpsee::ResponsePayload::success(1),
            ServerReturnType::Failure(f) => {
                let e = jsonrpsee::types::ErrorObject::owned(0, f, None::<serde_json::Value>);
                jsonrpsee::ResponsePayload::error(e)
            }
        }
    }
}
