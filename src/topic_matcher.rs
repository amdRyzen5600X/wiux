#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Zipped<I, J> {
    Both(I, J),
    Left(I),
    Right(J),
}

fn zip_longest<A, B>(a: A, b: B) -> impl Iterator<Item = Zipped<A::Item, B::Item>>
where
    A: IntoIterator,
    B: IntoIterator,
{
    let mut ait = a.into_iter();
    let mut bit = b.into_iter();
    std::iter::from_fn(move || match (ait.next(), bit.next()) {
        (Some(i), Some(j)) => Some(Zipped::Both(i, j)),
        (Some(i), None) => Some(Zipped::Left(i)),
        (None, Some(j)) => Some(Zipped::Right(j)),
        (None, None) => None,
    })
}

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
            return Err(crate::types::error::Error::InvalidTopicMatcherError(
                topic_filter,
            ));
        }
        if topic.all(|v| v) {
            return Ok(Self { topic_filter });
        }
        Err(crate::types::error::Error::InvalidTopicMatcherError(
            topic_filter,
        ))
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
    ///assert!(matcher.matches(msg_topic));
    ///```
    pub fn matches(&self, msg_topic: &str) -> bool {
        for zipped in zip_longest(self.topic_filter.split('/'), msg_topic.split('/')) {
            match zipped {
                Zipped::Both("+", _) => continue,
                Zipped::Both("#", _) | Zipped::Left("#") => return true,
                Zipped::Both(p, i) if p == i => continue,
                _ => return false,
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_test() {
        let matcher = TopicMatcher {
            topic_filter: "some/#/another",
        };
        let msg_topic = "some/one/another";
        assert!(matcher.matches(msg_topic));
    }
    #[test]
    fn matching_test1() {
        let matcher = TopicMatcher {
            topic_filter: "some/#/another",
        };
        let msg_topic = "some/one/two/another";
        assert!(matcher.matches(msg_topic));
    }
    #[test]
    fn matching_test2() {
        let matcher = TopicMatcher {
            topic_filter: "some/+/another",
        };
        let msg_topic = "some/one/two/another";
        assert!(!matcher.matches(msg_topic));
    }
    #[test]
    fn matching_test3() {
        let matcher = TopicMatcher {
            topic_filter: "some/+/another",
        };
        let msg_topic = "some/one/another";
        assert!(matcher.matches(msg_topic));
    }
    #[test]
    fn matching_test4() {
        let matcher = TopicMatcher {
            topic_filter: "some/#",
        };
        let msg_topic = "some/one/another";
        assert!(matcher.matches(msg_topic));
    }
    #[test]
    fn matching_test5() {
        let matcher = TopicMatcher {
            topic_filter: "one/some/#",
        };
        let msg_topic = "one/some";
        assert!(matcher.matches(msg_topic));
    }
    #[test]
    fn matching_test6() {
        let matcher = TopicMatcher {
            topic_filter: "one/some/#",
        };
        let msg_topic = "one/some";
        assert!(matcher.matches(msg_topic));
    }
    #[test]
    fn matching_test7() {
        let matcher = TopicMatcher {
            topic_filter: "one/+/some/#",
        };
        let msg_topic = "one/two/some";
        assert!(matcher.matches(msg_topic));
    }
    #[test]
    fn matching_test8() {
        let matcher = TopicMatcher {
            topic_filter: "one/+/some/#",
        };
        let msg_topic = "one/two/some/another/twonother";
        assert!(matcher.matches(msg_topic));
    }
    #[test]
    fn matching_test9() {
        let matcher = TopicMatcher {
            topic_filter: "one/+/some/#",
        };
        let msg_topic = "one/two/three/some/another";
        assert!(!matcher.matches(msg_topic));
    }
}
