use super::{Byte, EncodedString, QOS};

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Payload {
    pub content: Option<Payloads>,
}

impl Payload {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut res = Vec::new();
        if let Some(content) = &self.content {
            res.extend(content.to_bytes());
        }
        res

    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Payloads {
    Connect(ConnectPayload),
    Publish(Vec<u8>),
    Subscribe(Vec<SubscribePayload>),
    SubAcknowledge(Vec<Result<QOS, ()>>),
    Unsubscribe(Vec<EncodedString>),
    #[default]
    Default,
}

impl Payloads {
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

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConnectPayload {
    client_id: EncodedString,
    will_topic: Option<EncodedString>,
    will_message: Option<EncodedString>,
    username: Option<EncodedString>,
    password: Option<EncodedString>,
}

impl ConnectPayload {
    pub fn new(
        client_id: &str,
        will_topic: Option<&str>,
    will_message: Option<&str>,
        username: Option<&str>,
        password: Option<&str>,
    ) -> Self {
        let will_topic = match will_topic {
            Some(wt) => Some(EncodedString::new(wt)),
            None => None,
        };
        let will_message = match will_message {
            Some(wt) => Some(EncodedString::new(wt)),
            None => None,
        };
        let username = match username {
            Some(wt) => Some(EncodedString::new(wt)),
            None => None,
        };
        let password = match password {
            Some(wt) => Some(EncodedString::new(wt)),
            None => None,
        };
        Self {
            client_id: EncodedString::new(client_id),
            will_topic,
            will_message,
            username,
            password,
        }
    }
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

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SubscribePayload {
    topic_filter: EncodedString,
    qos: QOS,
}

impl SubscribePayload {
    pub fn new(topic_filter: &str, qos: QOS) -> Self {
        Self {
            topic_filter: EncodedString::new(topic_filter),
            qos,
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut res = Vec::new();
        res.extend(self.topic_filter.to_bytes());
        match self.qos {
            QOS::One => {
                res.push(Byte::new(1).to_u8());
            },
            QOS::Two => {
                res.push(Byte::new(2).to_u8());
            },
            QOS::Zero => {
                res.push(Byte::new(0).to_u8());
            },
        }
        res

    }
}
