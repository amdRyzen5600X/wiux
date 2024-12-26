use std::{
    io::{Read, Write},
    net::TcpStream,
    sync::Arc,
    time::Instant,
};

use crate::types::{
    header::{self, Header, VariableHeader}, payload::{self, ConnectPayload, Payload}, Byte, CallbackFunc, ControlPacket, EncodedString, Integer, LogCollbackFunc, ServerConnection, Will, QOS
};

#[derive(Debug)]
pub struct Client {
    client_id: String,
    server_connection: ServerConnection,
    clean_session: bool,
    will: Option<Will>,
    tcp_stream: Arc<TcpStream>,
    intent_disconnect: bool,
}

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
    pub fn on_message<C: Fn(&mut T, ControlPacket) + 'a>(&mut self, callback: C) {
        self.message_callback = Some(Box::new(callback));
    }
    pub fn on_connect<C: Fn(&mut T, i32) + 'a>(&mut self, callback: C) {
        self.connect_callback = Some(Box::new(callback));
    }
    pub fn on_publish<C: Fn(&mut T, i32) + 'a>(&mut self, callback: C) {
        self.publish_callback = Some(Box::new(callback));
    }
    pub fn on_subscribe<C: Fn(&mut T, i32) + 'a>(&mut self, callback: C) {
        self.subscribe_callback = Some(Box::new(callback));
    }
    pub fn on_unsubscribe<C: Fn(&mut T, i32) + 'a>(&mut self, callback: C) {
        self.unsubscribe_callback = Some(Box::new(callback));
    }
    pub fn on_disconnect<C: Fn(&mut T, i32) + 'a>(&mut self, callback: C) {
        self.disconnect_callback = Some(Box::new(callback));
    }
    pub fn on_log<C: Fn(&mut T, u32, &str) + 'a>(&mut self, callback: C) {
        self.log_callback = Some(Box::new(callback));
    }
}

impl Client {
    pub fn disconnect(&mut self) -> Result<(), ()> {
        let packet = ControlPacket {
            header: Header::new(header::FixedHeader::Disconnect, None),
            payload: Payload { content: None },
        };
        self.intent_disconnect = true;
        let res = self.tcp_stream
            .as_ref()
            .write_all(&packet.to_bytes())
            .map_err(|_| {});
        if res.is_err() {
            self.intent_disconnect = false;
        }
        res
    }
    pub fn publish(
        &self,
        topic: &str,
        message_text: &str,
        qos: QOS,
        retain: bool,
    ) -> Result<i32, ()> {
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
            .map_err(|_| {})?;
        Ok(pid as i32)
    }
    pub fn new(
        client_id: String,
        will: Option<Will>,
        clean_session: bool,
        host: &str,
        port: u32,
        username: Option<&str>,
        pass: Option<&str>,
    ) -> Result<Self, ()> {
        let mut tcp_stream = TcpStream::connect(format!("{}:{}", host, port)).map_err(|err| {
            eprintln!("ERROR: unnable to connect to {}{}\n{}", host, port, err);
        })?;
        let flags: [u8; 8];
        if let Some(will) = &will {
            let will_qos_flags = match will.qos {
                QOS::One => [0, 1],
                QOS::Two => [1, 0],
                QOS::Zero => [0, 0],
            };
            flags = [
                if username.is_some() { 1 } else { 0 },
                if pass.is_some() && username.is_some() {
                    1
                } else {
                    0
                },
                if will.retain { 1 } else { 0 },
                will_qos_flags[0],
                will_qos_flags[1],
                1_u8, //will flag
                if clean_session { 1 } else { 0 },
                0,
            ];
        } else {
            flags = [
                if username.is_some() { 1 } else { 0 },
                if pass.is_some() && username.is_some() {
                    1
                } else {
                    0
                },
                0,
                0,
                0,
                0,
                if clean_session { 1 } else { 0 },
                0,
            ];
        }

        let header = Header::new(
            header::FixedHeader::Connect,
            Some(VariableHeader::Connect(header::Connect {
                protocol_name: EncodedString::new("MQTT"),
                protocol_level: Byte::new(4),
                connect_flags: Byte { bits: flags },
                keep_alive: Integer::new(0),
            })),
        );
        let will_payload = will.clone();
        let payload = Payload {
            content: Some(payload::Payloads::Connect(ConnectPayload::new(
                &client_id,
                will_payload.clone().map(|w| w.topic).as_deref(),
                will_payload.map(|w| w.message).as_deref(),
                username,
                pass,
            ))),
        };
        let packet = ControlPacket { header, payload };
        let server_connection = ServerConnection {
            username: username.map(EncodedString::new),
            password: pass.map(EncodedString::new),
            host: host.to_owned(),
            port,
        };
        tcp_stream.write_all(&packet.to_bytes()).map_err(|err| {
            eprintln!("ERROR: could not send {:?} {}", packet, err);
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

    pub fn reconnect(&self) -> Result<(), ()> {
        let flags: [u8; 8];
        if let Some(will) = &self.will {
            let will_qos_flags = match will.qos {
                QOS::One => [0, 1],
                QOS::Two => [1, 0],
                QOS::Zero => [0, 0],
            };
            flags = [
                if self.server_connection.username.is_some() {
                    1
                } else {
                    0
                },
                if self.server_connection.password.is_some()
                    && self.server_connection.username.is_some()
                {
                    1
                } else {
                    0
                },
                if will.retain { 1 } else { 0 },
                will_qos_flags[0],
                will_qos_flags[1],
                1_u8, //will flag
                if self.clean_session { 1 } else { 0 },
                0,
            ];
        } else {
            flags = [
                if self.server_connection.username.is_some() {
                    1
                } else {
                    0
                },
                if self.server_connection.password.is_some()
                    && self.server_connection.username.is_some()
                {
                    1
                } else {
                    0
                },
                0,
                0,
                0,
                0,
                if self.clean_session { 1 } else { 0 },
                0,
            ];
        }
        let header = Header::new(
            header::FixedHeader::Connect,
            Some(VariableHeader::Connect(header::Connect {
                protocol_name: EncodedString::new("MQTT"),
                protocol_level: Byte::new(4),
                connect_flags: Byte { bits: flags },
                keep_alive: Integer::new(0),
            })),
        );
        let will = self.will.clone();
        let payload = Payload {
            content: Some(payload::Payloads::Connect(ConnectPayload::new(
                &self.client_id,
                will.clone().map(|w| w.topic).as_deref(),
                will.map(|w| w.message).as_deref(),
                self.server_connection.username.clone().map(|u| u.to_str()),
                self.server_connection.password.clone().map(|u| u.to_str()),
            ))),
        };
        let packet = ControlPacket { header, payload };
        self.tcp_stream
            .as_ref()
            .write_all(&packet.to_bytes())
            .map_err(|_| {})?;
        Ok(())
    }
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
                            cb(&mut callbacks.data, conn.connect_return_code.to_u8() as i32);
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
