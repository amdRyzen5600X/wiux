use super::{EncodedString, Integer, QOS};

pub struct Header {
    fixed: FixedHeader,
    variable: Option<VariableHeader>,
}
pub enum FixedHeader {
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

pub enum VariableHeader {
    Publish(Publish),
    Puback(PublishAcknowledge),
    Pubrec(PublishRecieved),
    Pubrel(PublishRelease),
    Pubcomp(PublishComplete),
    Subscribe(Subscribe),
    Suback(Subscribe),
    Unsubscribe(Unsubscribe),
    Unsuback(Unsubscribe),
}

pub struct Publish {
    topic_name: EncodedString,
    packet_id: Integer,
}

pub struct PublishAcknowledge {
    packet_id: Integer,
}

pub struct PublishRecieved {
    packet_id: Integer,
}
pub struct PublishRelease {
    packet_id: Integer,
}
pub struct PublishComplete {
    packet_id: Integer,
}

pub struct Subscribe {
    packet_id: Integer,
}
pub struct Unsubscribe {
    packet_id: Integer,
}
