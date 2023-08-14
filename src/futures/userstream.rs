use crate::client::*;
use crate::errors::*;
use crate::futures::rest_model::*;

static FUTURE_USER_DATA_STREAM: &str = "/fapi/v1/listenKey";

#[derive(Clone)]
pub struct FuturesUserStream {
    pub client: Client,
    pub recv_window: u64,
}

impl FuturesUserStream {

    pub async fn start(&self) -> Result<FuturesUserDataStream> { self.client.post(FUTURE_USER_DATA_STREAM, None).await }

    pub async fn keep_alive(&self, listen_key: &str) -> Result<Success> {
        self.client.put(FUTURE_USER_DATA_STREAM, listen_key, None).await
    }

    pub async fn close(&self, listen_key: &str) -> Result<Success> {
        self.client.delete(FUTURE_USER_DATA_STREAM, listen_key, None).await
    }
}
