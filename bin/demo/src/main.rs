use std::{str::FromStr, sync::Arc};
use tokio::time::{sleep, Duration};
use tokio::sync::Mutex;

use async_trait::async_trait;
use logging::PrintLogger;
#[allow(unused_imports)]
use message::{Message, MessageString, PayloadMessage};
use network::{Network, Node};
use node::{PassiveNode, SequentialNode};
use utils::{skip, ProcessEffect};
use env_logger;

/// A ping message
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Ping<T: std::fmt::Debug + Clone + Eq + PartialEq>(PayloadMessage<T>);

impl<T: std::fmt::Debug + Clone + Eq + PartialEq + ToString> Ping<T> {
    pub fn new(payload: T) -> Self {
        Ping(PayloadMessage::new(payload))
    }

    pub fn payload(&self) -> &T {
        self.0.payload()
    }

    pub fn to_string(&self) -> String {
        format!("Ping({:?})", self.payload())
    }
}

pub fn is_ping(message: &MessageString) -> bool {
    message.message.starts_with("Ping(")
}

impl FromStr for Ping<i32> {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let payload = s
            .trim_start_matches("Ping(")
            .trim_end_matches(")")
            .parse()?;
        Ok(Ping::new(payload))
    }
}

/// A pong message
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Pong<T: std::fmt::Debug + Clone + Eq + PartialEq>(PayloadMessage<T>);

impl<T: std::fmt::Debug + Clone + Eq + PartialEq> Pong<T> {
    pub fn new(payload: T) -> Self {
        Pong(PayloadMessage::new(payload))
    }

    pub fn payload(&self) -> &T {
        self.0.payload()
    }

    pub fn to_string(&self) -> String {
        format!("Pong({:?})", self.payload())
    }
}

impl FromStr for Pong<i32> {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let payload = s
            .trim_start_matches("Pong(")
            .trim_end_matches(")")
            .parse()?;
        Ok(Pong::new(payload))
    }
}

/// A node that sends pings
pub struct PingNode {
    base: PassiveNode,
}

impl PingNode {
    pub fn new() -> Self {
        PingNode {
            base: PassiveNode::new(),
        }
    }
}

impl std::fmt::Debug for PingNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PingNode({})", self.ident())
    }
}

#[async_trait]
impl Node for PingNode {
    fn initialize(&mut self, ident: i32, network: Arc<Mutex<Network>>) {
        self.base.initialize(ident, network);
    }

    fn ident(&self) -> i32 {
        self.base.ident()
    }

    fn network(&self) -> Arc<Mutex<Network>> {
        self.base.network()
    }

    async fn run(&self) -> ProcessEffect {
        for i in 0..self.network().lock().await.num_nodes() {
            let message = MessageString::new(Ping::new(i).to_string());
            self.send(i as i32, message.clone(), None).await;
            sleep(Duration::from_secs(1)).await;
            self.send(i as i32, message, None).await;
            sleep(Duration::from_secs(2)).await;
        }
        skip().await
    }

    async fn handle(&self, sender: i32, message: MessageString) -> ProcessEffect {
        self.base.handle(sender, message).await
    }
}

/// A node that responds to pings sequentially
pub struct PongNode {
    base: SequentialNode,
}

impl PongNode {
    pub fn new() -> Self {
        PongNode {
            base: SequentialNode::new(),
        }
    }
}

impl std::fmt::Debug for PongNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PongNode({})", self.ident())
    }
}

#[async_trait]
impl Node for PongNode {
    fn initialize(&mut self, ident: i32, network: Arc<Mutex<Network>>) {
        self.base.initialize(ident, network);
    }

    fn ident(&self) -> i32 {
        self.base.ident()
    }

    fn network(&self) -> Arc<Mutex<Network>> {
        self.base.network()
    }

    async fn handle(&self, sender: i32, message: MessageString) -> ProcessEffect {
        if let Ok(ping) = Ping::from_str(&message.message) {
            sleep(Duration::from_secs(5)).await;
            let pong_message = MessageString::new(Pong::new(ping.payload()).to_string());
            self.send(sender, pong_message, None).await;
        } else {
            self.base.handle(sender, message).await;
        }
        skip().await
    }

    async fn run(&self) -> ProcessEffect {
        self.base.run().await
    }
}

/// Runs the demo
#[tokio::main]
pub async fn main() {
    env_logger::init();
    let network: Arc<tokio::sync::Mutex<Network>> = Network::new(None, 4, Box::new(PrintLogger::default()));
    // Add 10 PongNodes
    for _ in 0..10 {
        network.lock().await.add_node(Arc::new(PongNode::new()));
    }
    println!("Num nodes in network is {}", network.lock().await.num_nodes());

    // Add PingNode
    network.lock().await.add_node(Arc::new(PingNode::new()));
    // Start all nodes
    network.lock().await.start_all_nodes().await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_demo() {
        main()
    }
}
