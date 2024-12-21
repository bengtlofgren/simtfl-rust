use std::hash::{Hash, Hasher};
use tokio::sync::oneshot;  // For Event-like functionality

/// Type alias for process effects, similar to Python's Generator[Event, None, None]
pub type ProcessEffect = oneshot::Receiver<()>;

/// Does nothing, equivalent to Python's skip()
pub async fn skip() -> ProcessEffect {
    let (tx, rx) = oneshot::channel();
    tx.send(()).unwrap();  // Send immediately
    rx
}

/// Represents a unique value, similar to Python's Unique class
#[derive(Debug)]
pub struct Unique {
    // We'll use the memory address of this field for uniqueness
    _marker: std::marker::PhantomData<()>,
}

impl Unique {
    pub fn new() -> Self {
        Unique {
            _marker: std::marker::PhantomData,
        }
    }
}

// Implement equality based on memory address
impl PartialEq for Unique {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Eq for Unique {}

// Implement hashing based on memory address
impl Hash for Unique {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self as *const Self).hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_unique() {
        let u1 = Unique::new();
        let u2 = Unique::new();
        
        assert_ne!(u1, u2);
        
        let mut set = HashSet::new();
        set.insert(u1);
        set.insert(u2);
        assert_eq!(set.len(), 2);
    }
}