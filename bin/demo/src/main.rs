use std::{str::FromStr, sync::Arc};
use tokio::time::{sleep, Duration};
use tokio::sync::Mutex;

use async_trait::async_trait;
use logging::{DebugLogger, PrintLogger};
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
        self.log("RUN", "ping node").await;
        for i in 0..self.network().lock().await.num_nodes() {
            let ping_i = Box::new(Ping::new(i));
            self.send(i as i32, ping_i.box_clone(), None).await;
            sleep(Duration::from_secs(1)).await;
            self.send(i as i32, ping_i, None).await;
            sleep(Duration::from_secs(2)).await;
        }
        skip().await
    }

    async fn handle(&self, sender: i32, message: Box<dyn Message>) -> ProcessEffect {
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

    async fn handle(&self, sender: i32, message: Box<dyn Message>) -> ProcessEffect {
        if let Some(ping) = message.as_any_ref().downcast_ref::<Ping<i32>>() {
            sleep(Duration::from_secs(5)).await;
            let pong_i = Box::new(Pong::new(*ping.payload()));
            self.send(sender, pong_i, None).await;
        } else {
            self.base.handle(sender, message).await;
        }
        skip().await
    }

    async fn run(&self) -> ProcessEffect {
        self.log("RUN", "pong node").await;
        self.base.run().await
    }
}

/// Runs the demo
#[tokio::main]
pub async fn main() {
    env_logger::init();
    let network: Arc<tokio::sync::Mutex<Network>> = Network::new(None, 4, Box::new(DebugLogger::default()));
    // Add 10 PongNodes
    for _ in 0..10 {
        network.lock().await.add_node(Arc::new(PongNode::new()));
    }

    // Add PingNode
    network.lock().await.add_node(Arc::new(PingNode::new()));
    // Start all nodes
    network.lock().await.start_all_nodes().await;

    // Run for 10 seconds
    // TODO: Make shutdown more graceful
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    // Stop all nodes on command
    println!("Press Enter to stop network");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use logging::PrintLogger;
    use std::any::Any;
    // Objectives:
    // Abstract - It simulates node behavior in response to messages based on the description of the protocol design. (It doesn’t need to make network connections, use persistent storage, or define message formats.)
    // Deterministic - A given simulator run should always produce identical results on any platform.
    // Network-wide - A simulator run simulates all nodes on the network.
    // Faster-than-real-time - It simulates message transmission times, delays, or message order interleaving directly without using the real clock so that each simulation can run as fast as the host system allows. 
    // Full-network Per-message Causal Ordering - It is possible to simulate the absolute arrival time of every message in the network. For example if node A sends node B message T1, and node C sends node D message T2, it can simulate either T1 arriving and being processed first, or T2 arriving and being processed.
    
    /// A node that responds to pings with strings
    
    pub struct PingNodeType {
        base: PassiveNode,
    }

    impl PingNodeType {
        pub fn new() -> Self {
            PingNodeType {
                base: PassiveNode::new(),
            }
        }
    }

    impl std::fmt::Debug for PingNodeType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "PingNodeType({})", self.ident())
        }
    }

    #[async_trait]
    impl Node for PingNodeType {
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
            self.log("RUN", "ping node").await;
            for i in 0..self.network().lock().await.num_nodes() {
                match i % 3 {
                    0 => {
                        let ping_i = Box::new(Ping::new(i.to_string()));
                        self.send(i as i32, ping_i.box_clone(), None).await;
                        sleep(Duration::from_secs(1)).await;
                        self.send(i as i32, ping_i, None).await;
                        sleep(Duration::from_secs(2)).await;
                    },
                    1 => {
                        let ping_i = Box::new(Ping::new(i as i32));
                        self.send(i as i32, ping_i.box_clone(), None).await;
                        sleep(Duration::from_secs(1)).await;
                        self.send(i as i32, ping_i, None).await;
                        sleep(Duration::from_secs(2)).await;
                    },
                    _ => {
                        let ping_i = Box::new(Ping::new(i));
                        self.send(i as i32, ping_i.box_clone(), None).await;
                        sleep(Duration::from_secs(1)).await;
                        self.send(i as i32, ping_i, None).await;
                        sleep(Duration::from_secs(2)).await;
                    },
                }
            }
            skip().await
        }

        async fn handle(&self, sender: i32, message: Box<dyn Message>) -> ProcessEffect {
            self.base.handle(sender, message).await
        }
    }

    pub struct PongNodeType {
        base: SequentialNode,
    }

    impl PongNodeType {
        pub fn new() -> Self {
            PongNodeType {
                base: SequentialNode::new(),
            }
        }
    }

    impl std::fmt::Debug for PongNodeType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "PongNodeType({})", self.ident())
        }
    }

    #[async_trait]
    impl Node for PongNodeType{
        fn initialize(&mut self, ident: i32, network: Arc<Mutex<Network>>) {
            self.base.initialize(ident, network);
        }

        fn ident(&self) -> i32 {
            self.base.ident()
        }

        fn network(&self) -> Arc<Mutex<Network>> {
            self.base.network()
        }

        async fn handle(&self, sender: i32, message: Box<dyn Message>) -> ProcessEffect {
            if let Some(ping) = message.as_any_ref().downcast_ref::<Ping<String>>() {
                sleep(Duration::from_secs(5)).await;
                let pong = String::default().handle_ping(ping.payload().as_any_ref());
                self.send(sender, pong, None).await;
            } else if let Some(ping) = message.as_any_ref().downcast_ref::<Ping<i32>>() {
                sleep(Duration::from_secs(5)).await;
                let pong = 0i32.handle_ping(ping.payload().as_any_ref());
                self.send(sender, pong, None).await;
            } else {
                self.base.handle(sender, message).await;
            }
            skip().await
        }

        async fn run(&self) -> ProcessEffect {
            self.log("RUN", "pong node").await;
            self.base.run().await
        }
    }

    trait PingHandler {
        fn handle_ping(&self, payload: &dyn Any) -> Box<dyn Message>;
    }
    
    // Implement for different types
    impl PingHandler for String {
        fn handle_ping(&self, payload: &dyn Any) -> Box<dyn Message> {
            if let Some(p) = payload.downcast_ref::<String>() {
                Box::new(Pong::new(p.clone()))
            } else {
                panic!("Invalid payload type")
            }
        }
    }
    
    impl PingHandler for i32 {
        fn handle_ping(&self, payload: &dyn Any) -> Box<dyn Message> {
            if let Some(p) = payload.downcast_ref::<i32>() {
                Box::new(Pong::new(*p))
            } else {
                panic!("Invalid payload type")
            }
        }
    }
    
    #[tokio::test]
    async fn test_abstract_behavior() {
        env_logger::init();
        let network: Arc<tokio::sync::Mutex<Network>> = Network::new(None, 4, Box::new(PrintLogger::default()));
        // Add 10 PongNodes
        for _ in 0..10 {
            network.lock().await.add_node(Arc::new(PongNodeType::new()));
        }

        // Add PingNode
        network.lock().await.add_node(Arc::new(PingNodeType::new()));
        // Start all nodes
        network.lock().await.start_all_nodes().await;

        // Run for 10 seconds
        // TODO: Make shutdown more graceful
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }

    // Test that the simulation is deterministic is done in the scripts directory

    #[tokio::test] 
    async fn test_network_wide_simulation() {
        // Verify simulation runs across all nodes
        let network = Network::new(None, 4, Box::new(DebugLogger::default()));
        
        // Add multiple nodes
        for _ in 0..5 {
            network.lock().await.add_node(Arc::new(PongNode::new()));
        }

        assert_eq!(network.lock().await.num_nodes(), 5);
        network.lock().await.start_all_nodes().await;
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }

    #[tokio::test]
    async fn test_faster_than_realtime() {
        // Verify simulated delays don't use real clock time
        let start = std::time::Instant::now();
        
        let network = Network::new(None, 4, Box::new(DebugLogger::default()));
        let pong_node = Arc::new(PongNode::new());
        network.lock().await.add_node(pong_node.clone());

        // Handle message with 5 second simulated delay
        let ping = Box::new(Ping::new(42));
        pong_node.handle(0, ping).await;

        // Should complete much faster than 5 seconds
        assert!(start.elapsed() < Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_message_ordering() {
        // Test different message arrival orderings
        let network = Network::new(None, 4, Box::new(DebugLogger::default()));
        
        let node1 = Arc::new(PongNode::new());
        let node2 = Arc::new(PongNode::new());
        network.lock().await.add_node(node1.clone());
        network.lock().await.add_node(node2.clone());

        // Send messages with different delays
        let ping1 = Box::new(Ping::new(1));
        let ping2 = Box::new(Ping::new(2));

        // Messages can be processed in either order
        tokio::join!(
            node1.handle(0, ping1),
            node2.handle(0, ping2)
        );
    }
    
}
