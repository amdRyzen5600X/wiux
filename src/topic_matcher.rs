use crate::types::ControlPacket;

///Represents a topic matcher, with a topic_filter field.
#[derive(Default, Debug, Clone, Copy)]
pub struct TopicMatcher {
    topic_filter: &'static str,
}

impl TopicMatcher {
    pub(crate) fn new(topic_filter: &'static str) -> crate::types::error::Result<Self> {
        let mut topic = topic_filter.split('/').map(|v| {
            if (v.contains("#") || v.contains("+")) && v.len() > 1 {
                return false;
            }
            true
        });
        if topic_filter.contains("#") && !topic_filter.ends_with("#") {
            return Err(crate::types::error::Error::InvalidTopicMatcherError(topic_filter));
        }
        if topic.all(|v| v) {
            return Ok(Self { topic_filter });
        }
        Err(crate::types::error::Error::InvalidTopicMatcherError(topic_filter))
    }
    ///Checks if a control packet matches the topic filter.
    ///
    ///##Implementation Details
    ///###The TopicMatcher implementation uses a simple state machine to match the topic filter against the topic name in the control packet. The state machine handles the following cases:
    ///
    ///+ wildcard: matches any single level of the topic hierarchy
    ///\# wildcard: matches any remaining levels of the topic hierarchy
    ///exact matches: matches the exact topic name
    ///
    ///The matches method returns true if the control packet matches the topic filter, and false otherwise.
    ///
    ///#Example
    ///
    ///```ignore
    ///let matcher = TopicMatcher {
    ///    topic_filter: "one/+/some/#",
    ///};
    ///let msg_topic = "one/two/some/another/twonother";
    ///let msg = ControlPacket {
    ///    header: crate::types::header::Header::new(
    ///        FixedHeader::Publish(false, crate::types::QOS::Zero, false),
    ///        Some(VariableHeader::Publish(crate::types::header::Publish {
    ///            topic_name: types::EncodedString::new(msg_topic),
    ///            packet_id: types::Integer::new(0),
    ///        })),
    ///    ),
    ///    payload: types::payload::Payload { content: None },
    ///};
    ///assert!(matcher.matches(msg));
    ///```
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
                        if (sub.get(sub_p).is_none() && topic.get(topic_p).is_none())
                            || (topic.get(topic_p).is_none()
                                && sub[sub_p] == '+'
                                && sub.get(sub_p + 1).is_none())
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
        let msg_topic = "one/two/some/another/twonother";
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
