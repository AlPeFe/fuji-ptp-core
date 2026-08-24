use super::{Transport, TransportError};

pub struct MockTransport {
    pub sent_data: Vec<Vec<u8>>,
    /// Packets returned by `receive`, in FIFO order.
    pub received_data: Vec<Vec<u8>>,
    pub receive_error: Option<TransportError>,
}

impl MockTransport {
    pub fn new() -> Self {
        Self {
            sent_data: Vec::new(),
            received_data: Vec::new(),
            receive_error: None,
        }
    }

    pub fn queue_received(&mut self, data: Vec<u8>) {
        self.received_data.push(data);
    }

    pub fn fail_receive(&mut self, error: TransportError) {
        self.receive_error = Some(error);
    }
}

impl Transport for MockTransport {
    fn send(&mut self, data: &[u8]) -> Result<(), TransportError> {
        self.sent_data.push(data.to_vec());
        Ok(())
    }

    fn receive(&mut self) -> Result<Vec<u8>, TransportError> {
        if let Some(error) = self.receive_error.take() {
            return Err(error);
        }
        if self.received_data.is_empty() {
            Ok(vec![])
        } else {
            Ok(self.received_data.remove(0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_mock_transport() {
        let _transport = MockTransport::new();
    }

    #[test]
    fn can_store_sent_data() {
        let mut transport = MockTransport::new();

        transport.send(&[0x01, 0x02]).unwrap();
        transport.send(&[0x03, 0x04]).unwrap();

        assert_eq!(transport.sent_data.len(), 2);
        assert_eq!(transport.sent_data[0], vec![0x01, 0x02]);
        assert_eq!(transport.sent_data[1], vec![0x03, 0x04]);
    }
}
