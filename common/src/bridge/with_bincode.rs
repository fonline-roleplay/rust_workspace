use super::*;
use bincode::deserialize_from;
pub fn reader<M: DeserializeOwned + Debug>(
    mut stream: TcpStream,
    sender: Sender<BridgeMessage<M>>,
) -> Result<(), BridgeError> {
    loop {
        let msg_in = deserialize_from(&mut stream).map_err(BridgeError::BinCode)?;
        let hang = if let &BridgeMessage::Hang = &msg_in {
            true
        } else {
            false
        };
        //println!("{:?}", msg_in);
        sender
            .send(msg_in)
            .map_err(|_| BridgeError::ChannelDropped)?;
        if hang {
            println!("Bridge reader: hang");
            return Ok(());
        }
    }
}
