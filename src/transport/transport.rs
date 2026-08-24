pub trait Transport {
    fn send(&mut self, data: &[u8]) -> Result<(), TransportError>;
    fn receive(&mut self) -> Result<Vec<u8>, TransportError>;
}

#[derive(Debug)]
pub enum TransportError {
    ConnectionError,
    SendError,
    ReceiveError,
}
