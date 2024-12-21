use tokio::sync::oneshot;
use std::sync::Arc;

use message::Message;
use utils::{ProcessEffect, skip};
use logging::Logger;

/// Base trait for node properties
pub trait Node: Send + Sync + std::fmt::Debug {
    /// Initializes a Node with the given ident, environment, and network
    fn initialize(&mut self, ident: i32, network: Arc<Network>);

    /// Returns the node's identifier
    fn ident(&self) -> i32;

    /// Returns a reference to the network
    fn network(&self) -> Arc<Network>;

    /// Logs an event for this node
    fn log(&self, event: &str, detail: &str) {
        self.network().log(self.ident(), event, detail);
    }
}

/// Trait for node message handling
pub trait NodeMessaging: Node {
    type Message: Message;  // Associated type for messages

    /// Sends a message to a target node
    async fn send(&self, target: i32, message: Self::Message, delay: Option<u32>) -> ProcessEffect {
        self.network().send(self.ident(), target, message, delay).await
    }

    /// Broadcasts a message to all nodes
    async fn broadcast(&self, message: Self::Message, delay: Option<u32>) -> ProcessEffect {
        self.network().broadcast(self.ident(), message, delay).await
    }

    /// Receives a message from a sender
    async fn receive(&self, sender: i32, message: Self::Message) -> ProcessEffect {
        self.handle(sender, message).await
    }

    /// Handles a received message
    async fn handle(&self, sender: i32, message: Self::Message) -> ProcessEffect;

    /// Runs the node
    async fn run(&self) -> ProcessEffect;
}

/// Network simulation layer
pub struct Network {
    self_ref: Option<Arc<Network>>,  // Reference to self
    nodes: Vec<Box<dyn Node>>,  // Only needs basic Node functionality
    delay: i64,
    logger: Box<dyn Logger>,
}

impl Network {
    /// Creates a new Network with optional initial nodes and delay
    pub fn new(
        nodes: Option<Vec<Box<dyn Node>>>, 
        delay: i64,
        logger: Box<dyn Logger>
    ) -> Arc<Self> {
        logger.header();
        let network = Network {
            self_ref: None,
            nodes: nodes.unwrap_or_default(),
            delay,
            logger,
        };
        let arc = Arc::new(network);
        arc

    }

    /// Logs an event for a node
    pub fn log(&self, ident: i32, event: &str, detail: &str) {
        self.logger.log(
            ident,
            event,
            detail,
        );
    }

    /// Returns the number of nodes
    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }

    /// Returns a reference to a node by ident
    pub fn node(&self, ident: i32) -> Option<&Box<dyn Node>> {
        self.nodes.get(ident as usize)
    }

    /// Adds a new node to the network
    pub fn add_node(&mut self, mut node: Box<dyn Node>) {
        let ident = self.num_nodes() as i32;
        if let Some(network_ref) = &self.self_ref {
            node.initialize(ident, network_ref.clone());
            self.nodes.push(node);
        }
    }

    /// Starts a specific node
    pub async fn start_node(&self, ident: i32) {
        if let Some(node) = self.node(ident) {
            self.log(ident, "start", &format!("{:?}", node));
            tokio::spawn(node.run());
        }
    }

    /// Starts all nodes
    pub async fn start_all_nodes(&self) {
        for i in 0..self.num_nodes() {
            self.start_node(i as i32).await;
        }
    }

    /// Sends a message from one node to another
    pub async fn send(
        &self,
        sender: i32,
        target: i32,
        message: impl Message,
        delay: Option<u32>,
    ) -> ProcessEffect {
        let delay = delay.unwrap_or(self.delay as u32) as i64;
        
        self.log(
            sender,
            "send",
            &format!("to {:2} with delay {:2}: {:?}", target, delay, message),
        );

        // Spawn convey process
        let network = self.self_ref.clone().unwrap();
        tokio::spawn(async move {
            network.convey(delay, sender, target, message).await
        });

        skip().await
    }

    /// Broadcasts a message to all nodes
    pub async fn broadcast(
        &self,
        sender: i32,
        message: impl Message + Clone,
        delay: Option<u32>,
    ) -> ProcessEffect {
        let delay = delay.unwrap_or(self.delay as u32) as i64;
        
        self.log(
            sender,
            "broadcast",
            &format!("to * with delay {:2}: {:?}", delay, message),
        );

        // Spawn convey process for each node
        for target in 0..self.num_nodes() as i32 {
            if target != sender {
                let network = self.self_ref.clone().unwrap();
                let message = message.clone();
                tokio::spawn(async move {
                    network.convey(delay, sender, target, message).await
                });
            }
        }

        skip().await
    }

    /// Conveys a message from sender to target after delay
    async fn convey(
        &self,
        delay: i64,
        sender: i32,
        target: i32,
        message: impl Message,
    ) -> ProcessEffect {
        tokio::time::sleep(tokio::time::Duration::from_secs(delay as u64)).await;
        
        self.log(
            target,
            "receive",
            &format!("from {:2} with delay {:2}: {:?}", sender, delay, message),
        );

        if let Some(node) = self.node(target) {
            node.receive(sender, message).await
        } else {
            skip().await
        }
    }
}