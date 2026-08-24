use crate::transport::{Transport, TransportError};

pub const CONTAINER_COMMAND: u16 = 1;
pub const CONTAINER_DATA: u16 = 2;
pub const CONTAINER_RESPONSE: u16 = 3;
pub const RESPONSE_OK: u16 = 0x2001;

#[derive(Debug, PartialEq, Eq)]
pub enum PtpProtocolError {
    MalformedContainer,
    UnexpectedContainerType(u16),
    UnexpectedOperation(u16),
    TransactionMismatch { expected: u32, received: u32 },
    Response(u16),
}

#[derive(Debug)]
pub enum PtpSessionError {
    Transport(TransportError),
    Protocol(PtpProtocolError),
}

pub struct PtpSession<T: Transport> {
    transport: T,
    transaction_id: u32,
}
fn write_u16_le(b: &mut Vec<u8>, v: u16) {
    b.extend_from_slice(&v.to_le_bytes());
}
fn write_u32_le(b: &mut Vec<u8>, v: u32) {
    b.extend_from_slice(&v.to_le_bytes());
}

impl<T: Transport> PtpSession<T> {
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            transaction_id: 0,
        }
    }
    fn next_transaction_id(&mut self) -> u32 {
        self.transaction_id += 1;
        self.transaction_id
    }
    fn build_command(operation: u16, tx: u32, parameter1: u32) -> Vec<u8> {
        let mut p = Vec::new();
        write_u32_le(&mut p, 16);
        write_u16_le(&mut p, CONTAINER_COMMAND);
        write_u16_le(&mut p, operation);
        write_u32_le(&mut p, tx);
        write_u32_le(&mut p, parameter1);
        p
    }
    fn build_command_no_params(operation: u16, tx: u32) -> Vec<u8> {
        let mut p = Vec::new();
        write_u32_le(&mut p, 12);
        write_u16_le(&mut p, CONTAINER_COMMAND);
        write_u16_le(&mut p, operation);
        write_u32_le(&mut p, tx);
        p
    }

    fn build_data(operation: u16, tx: u32, value: &[u8]) -> Vec<u8> {
        let mut p = Vec::new();
        write_u32_le(&mut p, 12 + value.len() as u32);
        write_u16_le(&mut p, CONTAINER_DATA);
        write_u16_le(&mut p, operation);
        write_u32_le(&mut p, tx);
        p.extend_from_slice(value);
        p
    }
    pub fn send(&mut self, data: &[u8]) -> Result<(), TransportError> {
        self.transport.send(data)
    }
    pub fn receive(&mut self) -> Result<Vec<u8>, TransportError> {
        self.transport.receive()
    }
    pub fn transport(&self) -> &T {
        &self.transport
    }

    pub fn get_device_prop_value(&mut self, property: u16) -> Result<Vec<u8>, PtpSessionError> {
        let tx = self.next_transaction_id();
        self.transport
            .send(&Self::build_command(0x1015, tx, property as u32))
            .map_err(PtpSessionError::Transport)?;
        let data = self
            .transport
            .receive()
            .map_err(PtpSessionError::Transport)?;
        let payload = Self::parse_container(&data, CONTAINER_DATA, 0x1015, tx)
            .map_err(PtpSessionError::Protocol)?;
        let response = self
            .transport
            .receive()
            .map_err(PtpSessionError::Transport)?;
        Self::parse_response(&response, tx).map_err(PtpSessionError::Protocol)?;
        Ok(payload)
    }

    fn parse_container(
        packet: &[u8],
        expected: u16,
        operation: u16,
        tx: u32,
    ) -> Result<Vec<u8>, PtpProtocolError> {
        if packet.len() < 12 {
            return Err(PtpProtocolError::MalformedContainer);
        }
        let length = u32::from_le_bytes(packet[0..4].try_into().unwrap()) as usize;
        let kind = u16::from_le_bytes(packet[4..6].try_into().unwrap());
        let op = u16::from_le_bytes(packet[6..8].try_into().unwrap());
        let received_tx = u32::from_le_bytes(packet[8..12].try_into().unwrap());
        if length < 12 || length > packet.len() {
            return Err(PtpProtocolError::MalformedContainer);
        }
        if kind != expected {
            return Err(PtpProtocolError::UnexpectedContainerType(kind));
        }
        if op != operation {
            return Err(PtpProtocolError::UnexpectedOperation(op));
        }
        if received_tx != tx {
            return Err(PtpProtocolError::TransactionMismatch {
                expected: tx,
                received: received_tx,
            });
        }
        Ok(packet[12..length].to_vec())
    }
    fn parse_response(packet: &[u8], tx: u32) -> Result<(), PtpProtocolError> {
        if packet.len() < 12 {
            return Err(PtpProtocolError::MalformedContainer);
        }
        let length = u32::from_le_bytes(packet[0..4].try_into().unwrap()) as usize;
        let kind = u16::from_le_bytes(packet[4..6].try_into().unwrap());
        let code = u16::from_le_bytes(packet[6..8].try_into().unwrap());
        let received_tx = u32::from_le_bytes(packet[8..12].try_into().unwrap());
        if length < 12 || length > packet.len() || kind != CONTAINER_RESPONSE {
            return Err(PtpProtocolError::MalformedContainer);
        }
        if received_tx != tx {
            return Err(PtpProtocolError::TransactionMismatch {
                expected: tx,
                received: received_tx,
            });
        }
        if code != RESPONSE_OK {
            return Err(PtpProtocolError::Response(code));
        }
        Ok(())
    }
    pub fn set_device_prop_value(
        &mut self,
        property: u16,
        value: &[u8],
    ) -> Result<(), TransportError> {
        let tx = self.next_transaction_id();
        self.transport
            .send(&Self::build_command(0x1016, tx, property as u32))?;
        self.transport.send(&Self::build_data(0x1016, tx, value))?;
        Ok(())
    }

    /// SET followed by the camera response. An empty response is accepted for
    /// backwards compatibility with the original MockTransport.
    pub fn set_device_prop_value_wait(
        &mut self,
        property: u16,
        value: &[u8],
    ) -> Result<(), PtpSessionError> {
        self.set_device_prop_value(property, value)
            .map_err(PtpSessionError::Transport)?;
        let response = self
            .transport
            .receive()
            .map_err(PtpSessionError::Transport)?;
        if response.is_empty() {
            return Ok(());
        }
        Self::parse_response(&response, self.transaction_id).map_err(PtpSessionError::Protocol)
    }

    pub fn open_session(&mut self, session_id: u32) -> Result<(), PtpSessionError> {
        let tx = self.next_transaction_id();
        self.transport
            .send(&Self::build_command(0x1002, tx, session_id))
            .map_err(PtpSessionError::Transport)?;
        let response = self
            .transport
            .receive()
            .map_err(PtpSessionError::Transport)?;
        Self::parse_response(&response, tx).map_err(PtpSessionError::Protocol)
    }

    pub fn close_session(&mut self) -> Result<(), PtpSessionError> {
        let tx = self.next_transaction_id();
        self.transport
            .send(&Self::build_command_no_params(0x1003, tx))
            .map_err(PtpSessionError::Transport)?;
        let response = self
            .transport
            .receive()
            .map_err(PtpSessionError::Transport)?;
        Self::parse_response(&response, tx).map_err(PtpSessionError::Protocol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::MockTransport;
    #[test]
    fn can_create_ptp_session() {
        let _ = PtpSession::new(MockTransport::new());
    }
    #[test]
    fn can_generate_transaction_ids() {
        let mut s = PtpSession::new(MockTransport::new());
        assert_eq!(s.next_transaction_id(), 1);
        assert_eq!(s.next_transaction_id(), 2);
    }
    #[test]
    fn can_send_data() {
        let mut s = PtpSession::new(MockTransport::new());
        s.send(&[1, 2, 3]).unwrap();
        assert_eq!(s.transport().sent_data[0], vec![1, 2, 3]);
    }
    #[test]
    fn get_property_sends_command_and_parses_data_and_response() {
        let mut t = MockTransport::new();
        t.queue_received(vec![14, 0, 0, 0, 2, 0, 0x15, 0x10, 1, 0, 0, 0, 0xaa, 0xbb]);
        t.queue_received(vec![12, 0, 0, 0, 3, 0, 1, 0x20, 1, 0, 0, 0]);
        let mut s = PtpSession::new(t);
        assert_eq!(s.get_device_prop_value(0xd18d).unwrap(), vec![0xaa, 0xbb]);
        assert_eq!(
            s.transport().sent_data[0],
            vec![16, 0, 0, 0, 1, 0, 0x15, 0x10, 1, 0, 0, 0, 0x8d, 0xd1, 0, 0]
        );
    }

    #[test]
    fn can_build_ptp_command() {
        assert_eq!(
            PtpSession::<MockTransport>::build_command(0x1016, 1, 0xD18C),
            vec![
                0x10, 0, 0, 0, 1, 0, 0x16, 0x10, 1, 0, 0, 0, 0x8c, 0xd1, 0, 0
            ]
        );
    }
    #[test]
    fn can_build_ptp_data() {
        assert_eq!(
            PtpSession::<MockTransport>::build_data(0x1016, 1, &[3, 0]),
            vec![0x0e, 0, 0, 0, 2, 0, 0x16, 0x10, 1, 0, 0, 0, 3, 0]
        );
    }
}
