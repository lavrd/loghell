use async_trait::async_trait;

#[async_trait]
pub trait Handler {
    async fn handle(&mut self, buf: &[u8]) -> Option<Box<dyn std::error::Error>>;
}
