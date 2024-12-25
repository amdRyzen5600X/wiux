use super::{EncodedString, QOS};

pub struct Payload {
    content: Option<Payloads>,
}

pub enum Payloads {
    Connect(ConnectPayload),
    Publish(Vec<u8>),
    Subscribe(Vec<SubscribePayload>),
    SubAcknowledge(Vec<Result<QOS, ()>>),
    Unsubscribe(Vec<EncodedString>),
}

pub struct ConnectPayload {
    client_id: EncodedString,
    will_topic: Option<EncodedString>,
    username: Option<EncodedString>,
    password: Option<EncodedString>,
}

pub struct SubscribePayload {
    topic_filter: EncodedString,
    qos: QOS,
}
