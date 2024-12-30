use std::collections::VecDeque;

use header::Header;
use payload::Payload;

pub mod error;
pub mod header;
pub mod payload;

pub type CallbackFunc<'a, T, V> = Option<Box<dyn Fn(&mut T, V) + 'a>>;
pub type LogCollbackFunc<'a, T> = Option<Box<dyn Fn(&mut T, u32, &str) + 'a>>;

///Represents a 16-bit integer.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Integer {
    msb: u8,
    lsb: u8,
}

impl Integer {
    ///Creates a new Integer instance from a u16 value.
    pub fn new(val: u16) -> Self {
        let val = val.to_be_bytes();
        Self {
            msb: val[0],
            lsb: val[1],
        }
    }
    ///Converts the Integer instance to a byte vector.
    pub fn to_bytes(&self) -> Vec<u8> {
        vec![self.msb, self.lsb]
    }
    ///Converts the Integer instance to a u16 value.
    pub fn to_u16(&self) -> u16 {
        ((self.msb as u16) << 8) | self.lsb as u16
    }
}

///Represents a string encoded in utf-8 format expected by MQTT.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EncodedString {
    len: Integer,
    pub value: String,
}

impl EncodedString {
    ///Creates a new EncodedString instance from a &str value.
    pub fn new(s: &str) -> Self {
        Self {
            len: Integer::new(s.len() as u16),
            value: s.to_owned(),
        }
    }
    ///Converts the EncodedString instance to a byte vector.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut res = Vec::new();
        res.extend(self.len.to_bytes());
        res.extend(self.value.as_bytes());
        res
    }
}

///Represents the Quality of Service (QoS) level.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum QOS {
    ///At most once delivery.
    #[default]
    Zero,
    ///At least once delivery.
    One,
    ///Exactly least once delivery.
    Two,
}

///Represents an MQTT control packet.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ControlPacket {
    pub(crate) header: Header,
    pub(crate) payload: Payload,
}

impl ControlPacket {
    ///Creates a new ControlPacket instance from a byte vector.
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
                        connect_acknowledge_flags: buf[0],
                        connect_return_code: buf[1],
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
                            msb: buf[0],
                            lsb: buf[1],
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
                            msb: buf[0],
                            lsb: buf[1],
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
                            msb: buf[0],
                            lsb: buf[1],
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
                            msb: buf[0],
                            lsb: buf[1],
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
                            msb: buf[0],
                            lsb: buf[1],
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
                            msb: buf[0],
                            lsb: buf[1],
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
    ///Converts the ControlPacket instance to a byte vector.
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

///Represents a will message, with fields for topic, message, QoS, and retain.
#[derive(Debug, Default, Clone)]
pub struct Will {
    pub topic: String,
    pub message: String,
    pub qos: QOS,
    pub retain: bool,
}
