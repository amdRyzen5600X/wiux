pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Error {
    RequestError,
    SubscriptionAckhowledgeFailureError,
    InvalidTopicMatcherError(&'static str),
    PublicationError,
    ConnectionError,
    #[default]
    Default,
}
