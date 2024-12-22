use std::any::Any;

pub trait Message: std::fmt::Debug + Send + 'static + Any {
    fn box_clone(&self) -> Box<dyn Message>;
    fn as_any_ref(&self) -> &dyn Any;

}

impl<T> Message for T 
where 
    T: Send + Sync + std::fmt::Debug + Clone + 'static + Any
{
    fn box_clone(&self) -> Box<dyn Message> {
        Box::new(self.clone())
    }

    fn as_any_ref(&self) -> &dyn Any {
        self
    }

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
