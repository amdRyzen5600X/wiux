use crate::types::ControlPacket;

#[derive(Default, Debug, Clone, Copy)]
pub struct TopicMatcher {
    topic_filter: &'static str,
}

impl TopicMatcher {
    pub fn matches(&self, msg: ControlPacket) -> bool {
        match msg.header.variable {
            Some(crate::types::header::VariableHeader::Publish(h)) => {
                let topic: Vec<_> = h.topic_name.to_str().chars().collect();
                let sub: Vec<_> = self.topic_filter.chars().collect();
                let mut spos;
                let mut sub_p = 0;
                let mut topic_p = 0;

                let mut result = false;

                spos = 0;

                while sub.get(sub_p).is_some() {
                    if sub[sub_p] != topic[topic_p] || topic.get(topic_p).is_none() {
                        if sub[sub_p] == '+' {
                            spos += 1;
                            sub_p += 1;
                            while topic.get(topic_p).is_some() && topic[topic_p] != '/' {
                                topic_p += 1;
                            }
                            if topic.get(topic_p).is_none() && sub.get(sub_p).is_none() {
                                result = true;
                                return result;
                            }
                        } else if sub[sub_p] == '#' {
                            while topic.get(topic_p).is_some() {
                                topic_p += 1;
                            }
                            result = true;
                            return result;
                        } else {
                            if topic.get(topic_p).is_none()
                                && spos > 0
                                && sub[sub_p - 1] == '+'
                                && sub[sub_p] == '/'
                                && sub[sub_p + 1] == '#'
                            {
                                result = true;
                                return result;
                            }

                            while sub.get(sub_p).is_some() {
                                spos += 1;
                                sub_p += 1;
                            }

                            return result;
                        }
                    } else {
                        println!("{:?}\n{:?}", topic_p, sub_p);
                        println!("{:?}\n{:?}", topic, sub);
                        if topic.get(topic_p + 1).is_none()
                            && sub.get(sub_p + 1) == Some(&'/')
                            && sub.get(sub_p + 2) == Some(&'#')
                            && sub.get(sub_p + 3).is_none()
                        {
                            result = true;
                            return result;
                        }

                        spos += 1;
                        sub_p += 1;
                        topic_p += 1;
                        if sub.get(sub_p).is_none() && topic.get(topic_p).is_none() {
                            result = true;
                            return result;
                        } else if topic.get(topic_p).is_none()
                            && sub[sub_p] == '+'
                            && sub.get(sub_p + 1).is_none()
                        {
                            result = true;
                            return result;
                        }
                    }
                }
                if topic.get(topic_p).is_some() || sub.get(sub_p).is_some() {
                    result = false;
                }

                result
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{
        self,
        header::{FixedHeader, VariableHeader},
    };

    use super::*;

    #[test]
    fn matching_test() {
        let matcher = TopicMatcher {
            topic_filter: "some/#/another",
        };
        let msg_topic = "some/one/another";
        let msg = ControlPacket {
            header: crate::types::header::Header::new(
                FixedHeader::Publish(false, crate::types::QOS::Zero, false),
                Some(VariableHeader::Publish(crate::types::header::Publish {
                    topic_name: types::EncodedString::new(msg_topic),
                    packet_id: types::Integer::new(0),
                })),
            ),
            payload: types::payload::Payload { content: None },
        };
        assert!(matcher.matches(msg));
    }
    #[test]
    fn matching_test1() {
        let matcher = TopicMatcher {
            topic_filter: "some/#/another",
        };
        let msg_topic = "some/one/two/another";
        let msg = ControlPacket {
            header: crate::types::header::Header::new(
                FixedHeader::Publish(false, crate::types::QOS::Zero, false),
                Some(VariableHeader::Publish(crate::types::header::Publish {
                    topic_name: types::EncodedString::new(msg_topic),
                    packet_id: types::Integer::new(0),
                })),
            ),
            payload: types::payload::Payload { content: None },
        };
        assert!(matcher.matches(msg));
    }
    #[test]
    fn matching_test2() {
        let matcher = TopicMatcher {
            topic_filter: "some/+/another",
        };
        let msg_topic = "some/one/two/another";
        let msg = ControlPacket {
            header: crate::types::header::Header::new(
                FixedHeader::Publish(false, crate::types::QOS::Zero, false),
                Some(VariableHeader::Publish(crate::types::header::Publish {
                    topic_name: types::EncodedString::new(msg_topic),
                    packet_id: types::Integer::new(0),
                })),
            ),
            payload: types::payload::Payload { content: None },
        };
        assert!(!matcher.matches(msg));
    }
    #[test]
    fn matching_test3() {
        let matcher = TopicMatcher {
            topic_filter: "some/+/another",
        };
        let msg_topic = "some/one/another";
        let msg = ControlPacket {
            header: crate::types::header::Header::new(
                FixedHeader::Publish(false, crate::types::QOS::Zero, false),
                Some(VariableHeader::Publish(crate::types::header::Publish {
                    topic_name: types::EncodedString::new(msg_topic),
                    packet_id: types::Integer::new(0),
                })),
            ),
            payload: types::payload::Payload { content: None },
        };
        assert!(matcher.matches(msg));
    }
    #[test]
    fn matching_test4() {
        let matcher = TopicMatcher {
            topic_filter: "some/#",
        };
        let msg_topic = "some/one/another";
        let msg = ControlPacket {
            header: crate::types::header::Header::new(
                FixedHeader::Publish(false, crate::types::QOS::Zero, false),
                Some(VariableHeader::Publish(crate::types::header::Publish {
                    topic_name: types::EncodedString::new(msg_topic),
                    packet_id: types::Integer::new(0),
                })),
            ),
            payload: types::payload::Payload { content: None },
        };
        assert!(matcher.matches(msg));
    }
    #[test]
    fn matching_test5() {
        let matcher = TopicMatcher {
            topic_filter: "one/some/#",
        };
        let msg_topic = "one/some";
        let msg = ControlPacket {
            header: crate::types::header::Header::new(
                FixedHeader::Publish(false, crate::types::QOS::Zero, false),
                Some(VariableHeader::Publish(crate::types::header::Publish {
                    topic_name: types::EncodedString::new(msg_topic),
                    packet_id: types::Integer::new(0),
                })),
            ),
            payload: types::payload::Payload { content: None },
        };
        assert!(matcher.matches(msg));
    }
    #[test]
    fn matching_test6() {
        let matcher = TopicMatcher {
            topic_filter: "one/some/#",
        };
        let msg_topic = "one/some";
        let msg = ControlPacket {
            header: crate::types::header::Header::new(
                FixedHeader::Publish(false, crate::types::QOS::Zero, false),
                Some(VariableHeader::Publish(crate::types::header::Publish {
                    topic_name: types::EncodedString::new(msg_topic),
                    packet_id: types::Integer::new(0),
                })),
            ),
            payload: types::payload::Payload { content: None },
        };
        assert!(matcher.matches(msg));
    }
    #[test]
    fn matching_test7() {
        let matcher = TopicMatcher {
            topic_filter: "one/+/some/#",
        };
        let msg_topic = "one/two/some";
        let msg = ControlPacket {
            header: crate::types::header::Header::new(
                FixedHeader::Publish(false, crate::types::QOS::Zero, false),
                Some(VariableHeader::Publish(crate::types::header::Publish {
                    topic_name: types::EncodedString::new(msg_topic),
                    packet_id: types::Integer::new(0),
                })),
            ),
            payload: types::payload::Payload { content: None },
        };
        assert!(matcher.matches(msg));
    }
    #[test]
    fn matching_test8() {
        let matcher = TopicMatcher {
            topic_filter: "one/+/some/#",
        };
        let msg_topic = "one/two/some/another";
        let msg = ControlPacket {
            header: crate::types::header::Header::new(
                FixedHeader::Publish(false, crate::types::QOS::Zero, false),
                Some(VariableHeader::Publish(crate::types::header::Publish {
                    topic_name: types::EncodedString::new(msg_topic),
                    packet_id: types::Integer::new(0),
                })),
            ),
            payload: types::payload::Payload { content: None },
        };
        assert!(matcher.matches(msg));
    }
    #[test]
    fn matching_test9() {
        let matcher = TopicMatcher {
            topic_filter: "one/+/some/#",
        };
        let msg_topic = "one/two/three/some/another";
        let msg = ControlPacket {
            header: crate::types::header::Header::new(
                FixedHeader::Publish(false, crate::types::QOS::Zero, false),
                Some(VariableHeader::Publish(crate::types::header::Publish {
                    topic_name: types::EncodedString::new(msg_topic),
                    packet_id: types::Integer::new(0),
                })),
            ),
            payload: types::payload::Payload { content: None },
        };
        assert!(!matcher.matches(msg));
    }
}
