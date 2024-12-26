use std::{io::Write, net::TcpStream, sync::Arc};

use crate::types::{
    header::{self, Header, VariableHeader},
    payload::{self, ConnectPayload, Payload},
    Byte, ControlPacket, EncodedString, Integer, ServerConnection, Will, QOS,
};

#[derive(Debug)]
pub struct Client {
    client_id: String,
    server_connection: ServerConnection,
    clean_session: bool,
    will: Option<Will>,
    tcp_stream: Arc<TcpStream>,
}

pub struct Callbacks<'a, T> {
    pub data: T,
    client: &'a Client,
    message_callback: Option<Box<dyn Fn(&mut T, ControlPacket) + 'a>>,
    connect_callback: Option<Box<dyn Fn(&mut T, i32) + 'a>>,
    publish_callback: Option<Box<dyn Fn(&mut T, i32) + 'a>>,
    subscribe_callback: Option<Box<dyn Fn(&mut T, i32) + 'a>>,
    unsubscribe_callback: Option<Box<dyn Fn(&mut T, i32) + 'a>>,
    disconnect_callback: Option<Box<dyn Fn(&mut T, i32) + 'a>>,
    log_callback: Option<Box<dyn Fn(&mut T, u32, &str) + 'a>>,
}

impl<'a, T> Callbacks<'a, T> {
    pub fn new(client: &'a Client, data: T) -> Self {
        Self {
            client,
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
        })
    }

    pub fn callbacks() {}

    pub fn reconnect(&mut self) -> Result<(), ()> {
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
}
