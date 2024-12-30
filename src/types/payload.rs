use super::{EncodedString, QOS};

///Represents an MQTT payload, with an optional Payloads enum value.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Payload {
    pub content: Option<Payloads>,
}

impl Payload {
    ///Converts the Payload instance to a byte vector.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut res = Vec::new();
        if let Some(content) = &self.content {
            res.extend(content.to_bytes());
        }
        res

    }
}

///Represents the different types of MQTT payloads.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Payloads {
    Connect(ConnectPayload),
    Publish(Vec<u8>),
    Subscribe(Vec<SubscribePayload>),
    SubAcknowledge(Vec<crate::types::error::Result<QOS>>),
    Unsubscribe(Vec<EncodedString>),
    #[default]
    Default,
}

impl Payloads {
    ///Converts the Payloads instance to a byte vector.
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Payloads::Connect(p) => { p.to_bytes()},
            Payloads::Publish(p) => {p.to_vec()},
            Payloads::Subscribe(p) => {
                let mut res = Vec::new();
                for t in p {
                    res.extend(t.to_bytes());
                }
                res
            },
            Payloads::Unsubscribe(p) => {
                let mut res = Vec::new();
                for s in p {
                    res.extend(s.to_bytes());
                }
                res
            },
            _ => {vec![]},
        }
    }
}

///Represents the payload for a CONNECT packet.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConnectPayload {
    client_id: EncodedString,
    will_topic: Option<EncodedString>,
    will_message: Option<EncodedString>,
    username: Option<EncodedString>,
    password: Option<EncodedString>,
}

impl ConnectPayload {
    ///Creates a new ConnectPayload instance.
    pub fn new(
        client_id: &str,
        will_topic: Option<&str>,
    will_message: Option<&str>,
        username: Option<String>,
        password: Option<String>,
    ) -> Self {
        let will_topic = will_topic.map(EncodedString::new);
        let will_message = will_message.map(EncodedString::new);
        let username = username.as_deref().map(EncodedString::new);
        let password = password.as_deref().map(EncodedString::new);
        Self {
            client_id: EncodedString::new(client_id),
            will_topic,
            will_message,
            username,
            password,
        }
    }
    ///Converts the ConnectPayload instance to a byte vector.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut res = Vec::new();
        res.extend(self.client_id.to_bytes());
        if let Some(v) = &self.will_topic {
            res.extend(v.to_bytes());
        }
        if let Some(v) = &self.will_message {
            res.extend(v.to_bytes());
        }
        if let Some(v) = &self.username {
            res.extend(v.to_bytes());
        }
        if let Some(v) = &self.password {
            res.extend(v.to_bytes());
        }
        res

    }
}

///Represents the payload for a SUBSCRIBE packet.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SubscribePayload {
    pub(crate) topic_filter: EncodedString,
    pub(crate) qos: QOS,
}

impl SubscribePayload {
    ///Creates a new SubscribePayload instance.
    pub fn new(topic_filter: &str, qos: QOS) -> Self {
        Self {
            topic_filter: EncodedString::new(topic_filter),
            qos,
        }
    }
    ///Converts the SubscribePayload instance to a byte vector.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut res = Vec::new();
        res.extend(self.topic_filter.to_bytes());
        match self.qos {
            QOS::One => {
                res.push(1_u8);
            },
            QOS::Two => {
                res.push(2_u8);
            },
            QOS::Zero => {
                res.push(0_u8);
            },
        }
        res

    }
}
