use async_trait::async_trait;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

use logging::{Logger, PrintLogger};
#[allow(unused_imports)]
use message::{Message, MessageString};
use network::{Network, Node};
use utils::{skip, ProcessEffect};

#[allow(dead_code)]
pub struct PassiveNode {
    id: i32,
    network: Option<Arc<Mutex<Network>>>,
    logger: Arc<dyn Logger>,
}

impl PassiveNode {
    pub fn new() -> Self {
        PassiveNode {
            id: 0,
            network: None,
            logger: Arc::new(PrintLogger {}),
        }
    }
}

impl std::fmt::Debug for PassiveNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PassiveNode({})", self.id)
    }
}

#[async_trait]
impl Node for PassiveNode {
    fn initialize(&mut self, ident: i32, network: Arc<Mutex<Network>>) {
        self.id = ident;
        self.network = Some(network);
    }

    fn ident(&self) -> i32 {
        self.id
    }

    fn network(&self) -> Arc<Mutex<Network>> {
        self.network.clone().expect("Node not initialized")
    }

    async fn handle(&self, sender: i32, message: MessageString) -> ProcessEffect {
        self.log("RECEIVE", &format!("from {}: {:?}", sender, message)).await;
        skip().await
    }

    async fn run(&self) -> ProcessEffect {
        self.log("RUN", "passive node").await;
        skip().await
    }
}

/// A node that processes messages sequentially
pub struct SequentialNode {
    ident: i32,
    network: Option<Arc<Mutex<Network>>>,
    mailbox: Arc<Mutex<VecDeque<(i32, Box<MessageString>)>>>,
}

impl SequentialNode {
    pub fn new() -> Self {
        SequentialNode {
            ident: 0,
            network: None,
            mailbox: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
}

impl Default for SequentialNode {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for SequentialNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SequentialNode({})", self.ident)
    }
}

#[async_trait]
impl Node for SequentialNode {
    fn initialize(&mut self, ident: i32, network: Arc<Mutex<Network>>) {
        self.ident = ident;
        self.network = Some(network);
    }

    fn ident(&self) -> i32 {
        self.ident
    }

    fn network(&self) -> Arc<Mutex<Network>> {
        self.network.as_ref().expect("Node not initialized").clone()
    }

    async fn handle(&self, _sender: i32, _message: MessageString) -> ProcessEffect {
        skip().await
    }

    async fn receive(&self, sender: i32, message: MessageString) -> ProcessEffect {
        let mut mailbox = self.mailbox.lock().await;
        mailbox.push_back((sender, Box::new(message)));
        skip().await
    }

    async fn run(&self) -> ProcessEffect {
        loop {
            let maybe_message = {
                let mut mailbox = self.mailbox.lock().await;
                mailbox.pop_front()
            };

            if let Some((sender, message)) = maybe_message {
                self.log("handle", &format!("from {:2}: {:?}", sender, message)).await;
                self.handle(sender, *message).await;
            } else {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    }
}
