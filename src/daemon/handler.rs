pub trait Handler {
    fn handle(&mut self, buf: &[u8]) -> Option<Box<dyn std::error::Error>>;
}
