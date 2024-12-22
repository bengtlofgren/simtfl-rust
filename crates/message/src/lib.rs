pub trait Message: std::fmt::Debug + Clone + Send + 'static {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(bytes: &[u8]) -> Self;
}

#[derive(Debug, Clone)]
pub struct MessageString {
    pub message: String,
}

impl MessageString {
    pub fn new(message: String) -> Self {
        MessageString { message }
    }
}

impl Message for MessageString {
    fn serialize(&self) -> Vec<u8> {
        self.message.as_bytes().to_vec()
    }

    fn deserialize(bytes: &[u8]) -> Self {
        MessageString {
            message: String::from_utf8(bytes.to_vec()).unwrap(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PayloadMessage<T>
where
    T: std::fmt::Debug + Clone + Eq + PartialEq, // Required by the derive macros
{
    payload: T,
}

impl<T> PayloadMessage<T>
where
    T: std::fmt::Debug + Clone + Eq + PartialEq, // Same bounds needed here
{
    pub fn new(payload: T) -> Self {
        PayloadMessage { payload }
    }

    // Getter method
    pub fn payload(&self) -> &T {
        &self.payload
    }
}
