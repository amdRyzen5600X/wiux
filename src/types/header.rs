use super::{Byte, EncodedString, Integer, QOS};

///Represents an MQTT header, consisting of a fixed header and an optional variable header.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Header {
    pub(crate) fixed: FixedHeader,
    pub(crate) variable: Option<VariableHeader>,
}

impl Header {
    ///Creates a new Header instance with the given fixed header type and optional variable header.
    pub fn new(fixed_header_type: FixedHeader, variable_header: Option<VariableHeader>) -> Self {
        Self {
            fixed: fixed_header_type,
            variable: variable_header,
        }
    }
    ///Converts the header to a byte vector.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut res = Vec::new();
        res.extend(self.fixed.to_bytes());
        if let Some(v) = &self.variable {
            res.extend(v.to_bytes());
        }
        res
    }
}

///Represents the fixed header of an MQTT packet.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FixedHeader {
    #[default]
    Connect,
    Connack,
    Publish(bool, QOS, bool),
    Puback,
    Pubrec,
    Pubrel,
    Pubcomp,
    Subscribe,
    Suback,
    Unsubscribe,
    Unsuback,
    Pingreq,
    Pingresp,
    Disconnect,
}

impl FixedHeader {
    ///Converts the fixed header to a byte vector.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut res = Vec::new();
        match self {
            FixedHeader::Connect => {
                res.push(2_u8.pow(4));
            },
            FixedHeader::Publish(dup_flag, qos, retain_flag) => {
                let mut byte1 = 2_u8.pow(5) + 2_u8.pow(4);
                if *dup_flag {byte1 += 2_u8.pow(3)}
                if *retain_flag {byte1 += 2_u8.pow(0)}
                match qos {
                    QOS::One => {byte1 += 2_u8.pow(1)},
                    QOS::Two => {byte1 += 2_u8.pow(2)},
                    QOS::Zero => {},
                }
                res.push(byte1);
            },
            FixedHeader::Subscribe => {
                res.push(2_u8.pow(7) + 2_u8.pow(1));
            },
            FixedHeader::Unsubscribe => {
                res.push(2_u8.pow(7) + 2_u8.pow(5) + 2_u8.pow(1));
            },
            FixedHeader::Pingreq => {
                res.push(2_u8.pow(7) + 2_u8.pow(6));
            },
            FixedHeader::Disconnect => {
                res.push(2_u8.pow(6) + 2_u8.pow(5));
            },
            _ => {}
        }
        res.push(0);
        res
    }
}

///Represents the variable header of an MQTT packet.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum VariableHeader {
    Connect(Connect),
    Conack(ConnectAcknowledge),
    Publish(Publish),
    Puback(PublishAcknowledge),
    Pubrec(PublishRecieved),
    Pubrel(PublishRelease),
    Pubcomp(PublishComplete),
    Subscribe(Subscribe),
    Suback(Subscribe),
    Unsubscribe(Unsubscribe),
    Unsuback(Unsubscribe),
    #[default]
    Default,
}

impl VariableHeader {
    ///Converts the variable header to a byte vector.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut res = Vec::new();
        match self {
            VariableHeader::Connect(h) => {
                res.extend(h.protocol_name.to_bytes());
                res.push(h.protocol_level.to_u8());
                res.push(h.connect_flags.to_u8());
                res.extend(h.keep_alive.to_bytes());
            },
            VariableHeader::Publish(h) => {
                res.extend(h.topic_name.to_bytes());
                res.extend(h.packet_id.to_bytes());
            },
            VariableHeader::Subscribe(h) => {
                res.extend(h.packet_id.to_bytes());
            },
            VariableHeader::Unsubscribe(h) => {
                res.extend(h.packet_id.to_bytes());
            },
            _ => {},
        }
        res
    }
}

///Represents the connect packet variable header.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Connect {
    pub protocol_name: EncodedString,
    pub protocol_level: Byte,
    pub connect_flags: Byte,
    pub keep_alive: Integer,
}

///Represents the connect acknowledge packet variable header.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConnectAcknowledge {
    pub connect_acknowledge_flags: Byte,
    pub connect_return_code: Byte,
}

///Represents the publish packet variable header.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Publish {
    pub topic_name: EncodedString,
    pub packet_id: Integer,
}

///Represents the publish acknowledge packet variable header.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublishAcknowledge {
    pub packet_id: Integer,
}

///Represents the publish received packet variable header.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublishRecieved {
    pub packet_id: Integer,
}
///Represents the publish release packet variable header.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublishRelease {
    pub packet_id: Integer,
}
///Represents the publish complete packet variable header.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublishComplete {
    pub packet_id: Integer,
}

///Represents the subscribe packet variable header.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Subscribe {
    pub packet_id: Integer,
}
///Represents the unsubscribe packet variable header.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Unsubscribe {
    pub packet_id: Integer,
}
