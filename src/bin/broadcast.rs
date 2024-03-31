use std::{
    collections::{HashMap, HashSet},
    io::{stdout, Write},
    sync::Arc,
    thread,
};

use duststorm::*;
use serde::{Deserialize, Serialize};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Server.run(&mut BroadcastNode::new())?;
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct Topology {}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Broadcast {
    Broadcast {
        message: i32,
    },
    Gossip {
        message: i32,
        recipients: Vec<String>,
    },
    GossipOk {
        node_id: String,
        message: i32,
    },
    Read,
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
enum BroadcastOk {
    BroadcastOk,
    GossipOk,
    ReadOk { messages: Vec<i32> },
    TopologyOk,
}

struct BroadcastNode {
    node_id: Option<String>,
    neighbors: Vec<String>,
    messages: Vec<i32>,
    statuses: HashMap<i32, HashSet<String>>,
}

impl Node<Broadcast> for BroadcastNode {
    fn handle_init(
        &mut self,
        message: &Message<Init>,
        sender: &mut Sender,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.node_id = Some(message.body.custom.node_id.clone());
        let reply = message.reply(InitOk::InitOk);
        sender.send(&reply)?;

        Ok(())
    }

    fn handle(
        &mut self,
        message: &Message<Broadcast>,
        sender: &mut Sender,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match &message.body.custom {
            Broadcast::Broadcast { message: msg } => {
                self.messages.push(msg.clone());
                for neighbor in &self.neighbors {
                    let nodes = self.statuses.entry(*msg).or_insert_with(HashSet::new);
                    if nodes.contains(neighbor) {
                        let gossip = self.make_gossip(neighbor, msg);
                        sender.send(&gossip)?;
                    }
                }
                let reply = message.reply(BroadcastOk::BroadcastOk);
                sender.send(&reply)?;
            }
            Broadcast::Read => {
                let reply = message.reply(self.read_ok());
                sender.send(&reply)?;
            }
            Broadcast::Topology { topology } => {
                self.neighbors = topology
                    .get(self.node_id.as_ref().unwrap())
                    .unwrap()
                    .clone();
                let reply = message.reply(BroadcastOk::TopologyOk);
                sender.send(&reply)?;
            }
            Broadcast::Gossip {
                message: msg,
                recipients,
            } => {
                if !self.messages.contains(msg) {
                    self.messages.push(*msg);
                    let mut new_recipients = recipients.clone();
                    new_recipients.extend(self.neighbors.clone());
                    let sender = Arc::new(*sender);
                    for neighbor in &self.neighbors {
                        let sender = sender.clone();
                        thread::spawn(move || {
                            let nodes = self.statuses.entry(*msg).or_insert_with(HashSet::new);
                            if nodes.contains(neighbor)
                                && neighbor != &message.meta.src
                                && !recipients.contains(neighbor)
                            {
                                nodes.insert(neighbor.clone());
                                let gossip = self.make_gossip_to(&new_recipients, neighbor, msg);
                                sender.send(&gossip);
                            }
                        });
                    }
                    sender.send(&message.reply(BroadcastOk::GossipOk))?;
                }
            }
            Broadcast::GossipOk {
                node_id,
                message: msg,
            } => {
                if let Some(nodes) = self.statuses.get_mut(msg) {
                    nodes.remove(node_id);
                }
            }
        };
        Ok(())
    }
}

impl BroadcastNode {
    fn new() -> Self {
        Self {
            node_id: None,
            neighbors: Vec::new(),
            messages: Vec::new(),
            statuses: HashMap::new(),
        }
    }

    fn read_ok(&self) -> BroadcastOk {
        BroadcastOk::ReadOk {
            messages: self.messages.clone(),
        }
    }

    fn make_gossip_to(
        &self,
        neighbors: &Vec<String>,
        neighbor: &str,
        msg: &i32,
    ) -> Message<Broadcast> {
        Message {
            meta: Meta {
                src: self.node_id.as_ref().unwrap().clone(),
                dest: neighbor.to_string(),
            },
            body: Body::new(Broadcast::Gossip {
                message: *msg,
                recipients: neighbors.clone(),
            }),
        }
    }

    fn make_gossip(&self, neighbor: &str, msg: &i32) -> Message<Broadcast> {
        self.make_gossip_to(&self.neighbors, neighbor, msg)
    }
}
