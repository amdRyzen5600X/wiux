use std::{io::Write, net::TcpStream};

use header::{Header, VariableHeader};
use payload::{ConnectPayload, Payload};

pub mod header;
pub mod payload;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Byte {
    bits: [u8; 8],
}

impl Byte {
    pub fn new(val: u8) -> Self {
        let s = format!("{:b}", val);
        let mut bits = [0_u8; 8];
        for (i, c) in s.chars().enumerate() {
            if c == '1' {
                bits[i] = 1;
            } else {
                bits[i] = 0;
            }
        }
        Self { bits }
    }
    pub fn to_u8(&self) -> u8 {
        let mut res = 0_u8;
        for (i, b) in self.bits.iter().enumerate() {
            res += b * 2_u8.pow(i as u32);
        }
        res
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Integer {
    msb: Byte,
    lsb: Byte,
}

impl Integer {
    pub fn new(val: u16) -> Self {
        let val = val.to_be_bytes();
        Self {
            msb: Byte::new(val[0]),
            lsb: Byte::new(val[1]),
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut res = Vec::new();
        res.push(self.msb.to_u8());
        res.push(self.lsb.to_u8());
        res
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EncodedString {
    len: Integer,
    value: Vec<u8>,
}

impl EncodedString {
    pub fn new(s: &str) -> Self {
        Self {
            len: Integer::new(s.len() as u16),
            value: s.as_bytes().to_vec(),
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut res = Vec::new();
        res.extend(self.len.to_bytes());
        res.extend(self.value.clone());
        res
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum QOS {
    #[default]
    Zero,
    One,
    Two,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ControlPacket {
    header: Header,
    payload: Payload,
}

impl ControlPacket {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut res = Vec::new();
        res.extend(self.header.to_bytes());
        res.extend(self.payload.to_bytes());
        res
    }
}

#[derive(Debug, Default)]
pub struct Client {
    client_id: String,
    server_host: Option<String>,
    clean_session: bool,
    will_flag: bool,
    will_topic: String,
    will_qos: Option<QOS>,
    will_retain: bool,
    tcp_stream: Option<TcpStream>,
}

impl Client {
    pub fn new(client_id: String) -> Self {
        Client {
            client_id,
            server_host: None,
            clean_session: false,
            will_flag: false,
            will_retain: false,
            will_qos: None,
            will_topic: String::new(),
            tcp_stream: None,
        }
    }
    pub fn new_session(client_id: String, clean_session: bool) -> Self {
        Client {
            client_id,
            clean_session,
            server_host: None,
            will_flag: false,
            will_retain: false,
            will_qos: None,
            will_topic: String::new(),
            tcp_stream: None,
        }
    }

    pub fn callbacks() {}

    pub fn conect(
        &mut self,
        host: &str,
        port: u32,
        username: Option<&str>,
        will_message: Option<&str>,
        pass: Option<&str>,
    ) -> Result<(), ()> {
        let mut tcp_stream = TcpStream::connect(&format!("{}:{}", host, port)).map_err(|err| {
            eprintln!("ERROR: unnable to connect to {}{}\n{}", host, port, err);
        })?;
        let will_qos_flags = match &self.will_qos {
            Some(qos) => match qos {
                QOS::One => [0, 1],
                QOS::Two => [1, 0],
                QOS::Zero => [0, 0],
            },
            None => [0, 0],
        };
        let flags = [
            if username.is_some() { 1 } else { 0 },
            if pass.is_some() && username.is_some() {
                1
            } else {
                0
            },
            if self.will_retain { 1 } else { 0 },
            will_qos_flags[0],
            will_qos_flags[1],
            if self.will_flag { 1 } else { 0 },
            if self.will_retain { 1 } else { 0 },
            0,
        ];
        let header = Header::new(
            header::FixedHeader::Connect,
            Some(VariableHeader::Connect(header::Connect {
                protocol_name: EncodedString::new("MQTT"),
                protocol_level: Byte::new(4),
                connect_flags: Byte { bits: flags },
                keep_alive: Integer::new(0),
            })),
        );
        let payload = Payload {
            content: Some(payload::Payloads::Connect(ConnectPayload::new(
                &self.client_id,
                Some(&self.will_topic),
                will_message,
                username,
                pass,
            ))),
        };
        let packet = ControlPacket {
            header,
            payload,
        };
        tcp_stream.write_all(&packet.to_bytes()).map_err(|err| {
            eprintln!("ERROR: could not send {:?} {}", packet, err);
        })?;
        self.tcp_stream = Some(tcp_stream);
        Ok(())
    }
}
