use std::collections::VecDeque;

use header::Header;
use payload::Payload;

pub mod header;
pub mod payload;
pub mod error;

pub type CallbackFunc<'a, T, V> = Option<Box<dyn Fn(&mut T, V) + 'a>>;
pub type LogCollbackFunc<'a, T> = Option<Box<dyn Fn(&mut T, u32, &str) + 'a>>;

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
    pub fn to_u16(&self) -> u16 {
        ((self.msb.to_u8() as u16) << 8) | self.lsb.to_u8() as u16
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
    pub fn from_bytes(bytes: &mut VecDeque<u8>) -> Option<Self> {
        let mut buf = Vec::new();
        let packet_type = bytes.pop_front()?;
        let len = bytes
            .pop_front()
            .expect("unexpected response: expected remaining length of response got None");
        for _ in 0..len as usize {
            buf.push(
                bytes
                    .pop_front()
                    .expect("unexpected response: expected bytes of response got None"),
            );
        }
        match packet_type {
            32_u8 => {
                let header = Header::new(
                    header::FixedHeader::Connack,
                    Some(header::VariableHeader::Conack(header::ConnectAcknowledge {
                        connect_acknowledge_flags: Byte::new(buf[0]),
                        connect_return_code: Byte::new(buf[1]),
                    })),
                );
                return Some(ControlPacket {
                    header,
                    payload: Payload { content: None },
                });
            }
            64_u8 => {
                let header = Header::new(
                    header::FixedHeader::Puback,
                    Some(header::VariableHeader::Puback(header::PublishAcknowledge {
                        packet_id: Integer {
                            msb: Byte::new(buf[0]),
                            lsb: Byte::new(buf[1]),
                        },
                    })),
                );
                return Some(ControlPacket {
                    header,
                    payload: Payload { content: None },
                });
            }
            80_u8 => {
                let header = Header::new(
                    header::FixedHeader::Pubrec,
                    Some(header::VariableHeader::Pubrec(header::PublishRecieved {
                        packet_id: Integer {
                            msb: Byte::new(buf[0]),
                            lsb: Byte::new(buf[1]),
                        },
                    })),
                );
                return Some(ControlPacket {
                    header,
                    payload: Payload { content: None },
                });
            }
            98_u8 => {
                let header = Header::new(
                    header::FixedHeader::Pubrel,
                    Some(header::VariableHeader::Pubrel(header::PublishRelease {
                        packet_id: Integer {
                            msb: Byte::new(buf[0]),
                            lsb: Byte::new(buf[1]),
                        },
                    })),
                );
                return Some(ControlPacket {
                    header,
                    payload: Payload { content: None },
                });
            }
            112_u8 => {
                let header = Header::new(
                    header::FixedHeader::Pubcomp,
                    Some(header::VariableHeader::Pubcomp(header::PublishComplete {
                        packet_id: Integer {
                            msb: Byte::new(buf[0]),
                            lsb: Byte::new(buf[1]),
                        },
                    })),
                );
                return Some(ControlPacket {
                    header,
                    payload: Payload { content: None },
                });
            }
            144_u8 => {
                let header = Header::new(
                    header::FixedHeader::Suback,
                    Some(header::VariableHeader::Suback(header::Subscribe {
                        packet_id: Integer {
                            msb: Byte::new(buf[0]),
                            lsb: Byte::new(buf[1]),
                        },
                    })),
                );
                return Some(ControlPacket {
                    header,
                    payload: Payload {
                        content: Some(payload::Payloads::SubAcknowledge(
                            buf.as_slice()[2..]
                                .to_vec()
                                .iter()
                                .map(|b| match b {
                                    0 => {
                                        Ok(QOS::Zero)
                                    }
                                    1 => {
                                        Ok(QOS::One)
                                    }
                                    2 => {
                                        Ok(QOS::Two)
                                    }
                                    _ => {
                                        Err(crate::types::error::Error::SubscriptionAckhowledgeFailureError)
                                    }
                                })
                                .collect(),
                        )),
                    },
                });
            }
            176_u8 => {
                let header = Header::new(
                    header::FixedHeader::Unsuback,
                    Some(header::VariableHeader::Unsuback(header::Unsubscribe {
                        packet_id: Integer {
                            msb: Byte::new(buf[0]),
                            lsb: Byte::new(buf[1]),
                        },
                    })),
                );
                return Some(ControlPacket {
                    header,
                    payload: Payload { content: None },
                });
            }
            208_u8 => {
                let header = Header::new(header::FixedHeader::Pingresp, None);
                return Some(ControlPacket {
                    header,
                    payload: Payload { content: None },
                });
            }
            _ => {}
        }
        None
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut res = Vec::new();
        res.extend(self.header.to_bytes());
        res.extend(self.payload.to_bytes());
        res[1] = res.len() as u8;
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
