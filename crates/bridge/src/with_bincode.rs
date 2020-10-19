use super::*;
use bincode::deserialize_from;
pub fn reader<M: DeserializeOwned + Debug, S: Channel<M>>(
    mut stream: BufReader<TcpStream>,
    sender: S,
) -> Result<(), BridgeError> {
    loop {
        /*use byteorder::{ReadBytesExt, BE};
        stream.fill_buf()
        let len = stream.read_u32::<BE>().map_err(BridgeError::Io)?;*/

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
