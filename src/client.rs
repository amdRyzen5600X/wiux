use std::{
    io::{Read, Write},
    net::TcpStream,
    sync::Arc,
    time::Instant,
};

use crate::{
    topic_matcher::TopicMatcher,
    types::{
        header::{self, Header, VariableHeader},
        payload::{self, ConnectPayload, Payload, SubscribePayload},
        CallbackFunc, ControlPacket, EncodedString, Integer, LogCollbackFunc,
        ServerConnection, Will, QOS,
    },
};

///Represents an MQTT client, with fields for client ID, server connection, clean session, will, TCP stream, and intent to disconnect.
#[derive(Debug)]
pub struct Client {
    client_id: String,
    server_connection: ServerConnection,
    clean_session: bool,
    will: Option<Will>,
    tcp_stream: Arc<TcpStream>,
    intent_disconnect: bool,
}

///Represents a set of callbacks for the client.
pub struct Callbacks<'a, T> {
    pub data: T,
    message_callback: CallbackFunc<'a, T, ControlPacket>,
    connect_callback: CallbackFunc<'a, T, i32>,
    publish_callback: CallbackFunc<'a, T, i32>,
    subscribe_callback: CallbackFunc<'a, T, i32>,
    unsubscribe_callback: CallbackFunc<'a, T, i32>,
    disconnect_callback: CallbackFunc<'a, T, i32>,
    log_callback: LogCollbackFunc<'a, T>,
}

impl<'a, T> Callbacks<'a, T> {
    ///Creates a new Callbacks instance.
    pub fn new(data: T) -> Self {
        Self {
            data,
            message_callback: None,
            connect_callback: None,
            publish_callback: None,
            subscribe_callback: None,
            unsubscribe_callback: None,
            disconnect_callback: None,
            log_callback: None,
        }
    }
    ///Sets the message callback.
    pub fn on_message<C: Fn(&mut T, ControlPacket) + 'a>(&mut self, callback: C) {
        self.message_callback = Some(Box::new(callback));
    }
    ///Sets the connect callback.
    pub fn on_connect<C: Fn(&mut T, i32) + 'a>(&mut self, callback: C) {
        self.connect_callback = Some(Box::new(callback));
    }
    ///Sets the publish callback.
    pub fn on_publish<C: Fn(&mut T, i32) + 'a>(&mut self, callback: C) {
        self.publish_callback = Some(Box::new(callback));
    }
    ///Sets the subscribe callback.
    pub fn on_subscribe<C: Fn(&mut T, i32) + 'a>(&mut self, callback: C) {
        self.subscribe_callback = Some(Box::new(callback));
    }
    ///Sets the unsubscribe callback.
    pub fn on_unsubscribe<C: Fn(&mut T, i32) + 'a>(&mut self, callback: C) {
        self.unsubscribe_callback = Some(Box::new(callback));
    }
    ///Sets the disconnect callback.
    pub fn on_disconnect<C: Fn(&mut T, i32) + 'a>(&mut self, callback: C) {
        self.disconnect_callback = Some(Box::new(callback));
    }
    ///Sets the log callback.
    pub fn on_log<C: Fn(&mut T, u32, &str) + 'a>(&mut self, callback: C) {
        self.log_callback = Some(Box::new(callback));
    }
}

impl Client {
    ///Returns the host of the server connection.
    pub fn host(&self) -> &str {
        &self.server_connection.host
    }
    ///Returns the port of the server connection.
    pub fn port(&self) -> u32 {
        self.server_connection.port
    }
    ///Subscribes to a topic with a specified QoS.
    pub fn subscribe(&self, topic: &'static str, qos: QOS) -> crate::types::error::Result<TopicMatcher> {
        let pid = Instant::now().elapsed().subsec_millis() as u16;
        let packet = ControlPacket {
            header: Header {
                fixed: header::FixedHeader::Subscribe,
                variable: Some(VariableHeader::Subscribe(header::Subscribe {
                    packet_id: Integer::new(pid),
                })),
            },
            payload: Payload {
                content: Some(payload::Payloads::Subscribe(vec![SubscribePayload {
                    topic_filter: EncodedString::new(topic),
                    qos,
                }])),
            },
        };
        let tm = TopicMatcher::new(topic)?;
        self.tcp_stream
            .as_ref()
            .write_all(&packet.to_bytes())
            .map_err(|_| {
                crate::types::error::Error::RequestError
            })?;
        Ok(tm)
    }
    ///Unsubscribes from a topic.
    pub fn unsubscribe(&self, topic: &'static str) -> crate::types::error::Result<i32> {
        let pid = Instant::now().elapsed().subsec_millis() as u16;
        let packet = ControlPacket {
            header: Header {
                fixed: header::FixedHeader::Unsubscribe,
                variable: Some(VariableHeader::Unsubscribe(header::Unsubscribe {
                    packet_id: Integer::new(pid),
                })),
            },
            payload: Payload {
                content: Some(payload::Payloads::Unsubscribe(vec![EncodedString::new(
                    topic,
                )])),
            },
        };
        self.tcp_stream
            .as_ref()
            .write_all(&packet.to_bytes())
            .map_err(|_| {
                crate::types::error::Error::RequestError
            })?;
        Ok(pid.into())
    }
    ///Disconnects from the server.
    pub fn disconnect(&mut self) -> crate::types::error::Result<()>{
        let packet = ControlPacket {
            header: Header::new(header::FixedHeader::Disconnect, None),
            payload: Payload { content: None },
        };
        self.intent_disconnect = true;
        let res = self
            .tcp_stream
            .as_ref()
            .write_all(&packet.to_bytes())
            .map_err(|_| {
                crate::types::error::Error::RequestError
            });
        if res.is_err() {
            self.intent_disconnect = false;
        }
        res
    }
    ///Publishes a message to a topic with a specified QoS and retain flag.
    pub fn publish(
        &self,
        topic: &str,
        message_text: &str,
        qos: QOS,
        retain: bool,
    ) -> crate::types::error::Result<i32> {
        let pid = Instant::now().elapsed().subsec_millis() as u16;
        let header = Header::new(
            header::FixedHeader::Publish(false, qos, retain),
            Some(VariableHeader::Publish(header::Publish {
                topic_name: EncodedString::new(topic),
                packet_id: Integer::new(pid),
            })),
        );
        let payload = Payload {
            content: Some(payload::Payloads::Publish(message_text.as_bytes().to_vec())),
        };
        let packet = ControlPacket { header, payload };
        self.tcp_stream
            .as_ref()
            .write_all(&packet.to_bytes())
            .map_err(|_| {
                crate::types::error::Error::RequestError
            })?;
        Ok(pid as i32)
    }
    ///Creates a new Client instance.
    pub fn new(
        client_id: String,
        will: Option<Will>,
        clean_session: bool,
        host: &str,
        port: u32,
        username: Option<String>,
        pass: Option<String>,
    ) -> crate::types::error::Result<Self> {
        let mut tcp_stream = TcpStream::connect(format!("{}:{}", host, port)).map_err(|_| {
            crate::types::error::Error::ConnectionError
        })?;
        let mut flags = 0_u8;
        if username.is_some() { flags+=2_u8.pow(7)}
        if pass.is_some() && username.is_some() { flags+=2_u8.pow(6)}
        if clean_session { flags+=2_u8 }
        if let Some(will) = &will {
            let will_qos_flags = match will.qos {
                QOS::One => [0, 1],
                QOS::Two => [1, 0],
                QOS::Zero => [0, 0],
            };
            if will.retain { flags+=2_u8.pow(5) }
            if will_qos_flags[0] == 1 { flags+=2_u8.pow(4) }
            if will_qos_flags[1] == 1 { flags+=2_u8.pow(3) }
            flags+=2_u8.pow(2);
        }

        let header = Header::new(
            header::FixedHeader::Connect,
            Some(VariableHeader::Connect(header::Connect {
                protocol_name: EncodedString::new("MQTT"),
                protocol_level: 4_u8,
                connect_flags: flags,
                keep_alive: Integer::new(0),
            })),
        );
        let will_payload = will.clone();
        let payload = Payload {
            content: Some(payload::Payloads::Connect(ConnectPayload::new(
                &client_id,
                will_payload.clone().map(|w| w.topic).as_deref(),
                will_payload.map(|w| w.message).as_deref(),
                username.clone(),
                pass.clone(),
            ))),
        };
        let packet = ControlPacket { header, payload };
        let server_connection = ServerConnection {
            username: username.as_deref().map(EncodedString::new),
            password: pass.as_deref().map(EncodedString::new),
            host: host.to_owned(),
            port,
        };
        tcp_stream.write_all(&packet.to_bytes()).map_err(|_| {
            crate::types::error::Error::RequestError
        })?;
        Ok(Client {
            client_id,
            clean_session,
            server_connection,
            will,
            tcp_stream: Arc::new(tcp_stream),
            intent_disconnect: false,
        })
    }

    ///Reconnects to the server.
    pub fn reconnect(&self) -> crate::types::error::Result<()> {
        let mut flags = 0_u8;
        if self.server_connection.username.is_some() { flags+=2_u8.pow(7)}
        if self.server_connection.password.is_some() && self.server_connection.username.is_some() { flags+=2_u8.pow(6)}
        if self.clean_session { flags+=2_u8 }
        if let Some(will) = &self.will {
            let will_qos_flags = match will.qos {
                QOS::One => [0, 1],
                QOS::Two => [1, 0],
                QOS::Zero => [0, 0],
            };
            if will.retain { flags+=2_u8.pow(5) }
            if will_qos_flags[0] == 1 { flags+=2_u8.pow(4) }
            if will_qos_flags[1] == 1 { flags+=2_u8.pow(3) }
            flags+=2_u8.pow(2);
        }
        let header = Header::new(
            header::FixedHeader::Connect,
            Some(VariableHeader::Connect(header::Connect {
                protocol_name: EncodedString::new("MQTT"),
                protocol_level: 4_u8,
                connect_flags: flags,
                keep_alive: Integer::new(0),
            })),
        );
        let will = self.will.clone();
        let payload = Payload {
            content: Some(payload::Payloads::Connect(ConnectPayload::new(
                &self.client_id,
                will.clone().map(|w| w.topic).as_deref(),
                will.map(|w| w.message).as_deref(),
                self.server_connection.username.clone().map(|u| u.value),
                self.server_connection.password.clone().map(|u| u.value),
            ))),
        };
        let packet = ControlPacket { header, payload };
        self.tcp_stream
            .as_ref()
            .write_all(&packet.to_bytes())
            .map_err(|_| {
                crate::types::error::Error::ConnectionError
            })?;
        Ok(())
    }
    ///Runs the client loop with the provided callbacks.
    pub fn do_loop<T>(&self, mut callbacks: Callbacks<T>) {
        let mut bytes = std::collections::VecDeque::new();
        'outer: loop {
            let mut buf = [0_u8; 64];
            while let Ok(n) = self.tcp_stream.as_ref().read(&mut buf) {
                if n < buf.len() && n != 0 {
                    bytes.extend(buf[..n].to_vec());
                    break;
                }
                if n == 0 && self.intent_disconnect {
                    if let Some(ref cb) = callbacks.disconnect_callback {
                        cb(&mut callbacks.data, 0);
                    }
                    return;
                }
                if n == 0 && !self.intent_disconnect {
                    let _ = self.reconnect();
                    continue 'outer;
                }
                bytes.extend(buf[..n].to_vec());
            }
            let response = ControlPacket::from_bytes(&mut bytes);
            if let Some(resp) = response {
                match resp.header.fixed {
                    header::FixedHeader::Unsuback => {
                        if let Some(ref cb) = callbacks.unsubscribe_callback {
                            let header::VariableHeader::Unsuback(unsub) = resp
                                .header
                                .variable
                                .expect("FATAL: that should not appear in any circumstances")
                            else {
                                continue;
                            };
                            cb(&mut callbacks.data, unsub.packet_id.to_u16() as i32);
                        }
                    }
                    header::FixedHeader::Suback => {
                        if let Some(ref cb) = callbacks.subscribe_callback {
                            let header::VariableHeader::Suback(sub) = resp
                                .header
                                .variable
                                .expect("FATAL: that should not appear in any circumstances")
                            else {
                                continue;
                            };
                            cb(&mut callbacks.data, sub.packet_id.to_u16() as i32);
                        }
                    }
                    header::FixedHeader::Pubcomp => {
                        if let Some(ref cb) = callbacks.publish_callback {
                            let header::VariableHeader::Pubcomp(publ) = resp
                                .header
                                .variable
                                .expect("FATAL: that should not appear in any circumstances")
                            else {
                                continue;
                            };
                            cb(&mut callbacks.data, publ.packet_id.to_u16() as i32);
                        }
                    }
                    header::FixedHeader::Pubrel => {
                        if let Some(ref cb) = callbacks.publish_callback {
                            let header::VariableHeader::Pubrec(publ) = resp
                                .header
                                .variable
                                .expect("FATAL: that should not appear in any circumstances")
                            else {
                                continue;
                            };
                            cb(&mut callbacks.data, publ.packet_id.to_u16() as i32);
                        }
                    }
                    header::FixedHeader::Pubrec => {
                        if let Some(ref cb) = callbacks.publish_callback {
                            let header::VariableHeader::Pubrec(publ) = resp
                                .header
                                .variable
                                .expect("FATAL: that should not appear in any circumstances")
                            else {
                                continue;
                            };
                            cb(&mut callbacks.data, publ.packet_id.to_u16() as i32);
                        }
                    }
                    header::FixedHeader::Puback => {
                        if let Some(ref cb) = callbacks.publish_callback {
                            let header::VariableHeader::Puback(publ) = resp
                                .header
                                .variable
                                .expect("FATAL: that should not appear in any circumstances")
                            else {
                                continue;
                            };
                            cb(&mut callbacks.data, publ.packet_id.to_u16() as i32);
                        }
                    }
                    header::FixedHeader::Connack => {
                        if let Some(ref cb) = callbacks.connect_callback {
                            let header::VariableHeader::Conack(conn) = resp
                                .header
                                .variable
                                .expect("FATAL: that should not appear in any circumstances")
                            else {
                                continue;
                            };
                            cb(&mut callbacks.data, conn.connect_return_code as i32);
                        }
                    }
                    header::FixedHeader::Publish(_, _, _) => {
                        if let Some(ref cb) = callbacks.message_callback {
                            cb(&mut callbacks.data, resp);
                        }
                    }
                    header::FixedHeader::Pingresp => {}
                    _ => {}
                }
            } else {
                continue;
            }
        }
    }
}
