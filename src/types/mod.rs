use std::{io::Write, net::TcpStream};

use header::{Header, VariableHeader};
use payload::{ConnectPayload, Payload};

pub mod header;
pub mod payload;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Byte {
    pub(crate) bits: [u8; 8],
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
        vec![self.msb.to_u8(), self.lsb.to_u8()]
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
    pub fn to_str(&self) -> &'static str {
        return Box::leak(Box::new(
            String::from_utf8(self.value.clone()).expect("error while trying decode utf8 string"),
        ));
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
    pub(crate) header: Header,
    pub(crate) payload: Payload,
}

impl ControlPacket {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut res = Vec::new();
        res.extend(self.header.to_bytes());
        res.extend(self.payload.to_bytes());
        res
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct ServerConnection {
    pub(crate) host: String,
    pub(crate) port: u32,
    pub(crate) username: Option<EncodedString>,
    pub(crate) password: Option<EncodedString>,
}

#[derive(Debug, Default, Clone)]
pub struct Will {
    pub topic: String,
    pub message: String,
    pub qos: QOS,
    pub retain: bool,
}

