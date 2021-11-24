pub trait Handler {
    fn handle(&self, buf: &[u8]) -> Option<Box<dyn std::error::Error>>;
}
