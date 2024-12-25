use header::Header;
use payload::Payload;

pub mod header;
pub mod payload;

pub struct Byte {
    bits: [u8; 8],
}

pub struct Integer {
    msb: Byte,
    lsb: Byte,
}

pub struct EncodedString {
    len: Integer,
    value: Vec<u8>,
}

pub enum QOS {
    Zero,
    One,
    Two,
}

pub struct ControlPacket {
    header: Header,
    payload: Payload,

}

pub struct Client {
    client_id: String
}
