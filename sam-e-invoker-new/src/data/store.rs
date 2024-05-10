use std::{collections::HashMap, sync::Arc};

use parking_lot::RwLock;
use sam_e_types::{
    config::lambda::Lambda,
    invocation::Invocation,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InvocationQueue {
    invocations: Vec<Invocation>,
}

impl InvocationQueue {
    pub fn new() -> Self {
        Self {
            invocations: Vec::new(),
        }
    }

    pub fn get_invocations(&self) -> &Vec<Invocation> {
        &self.invocations
    }
    pub fn get_invocations_mut(&mut self) -> &mut Vec<Invocation> {
        &mut self.invocations
    }
}

pub type InvocationQueues = HashMap<String, InvocationQueue>;

#[derive(Clone, Debug)]
pub struct Store {
    pub queues: Arc<RwLock<InvocationQueues>>,
}

impl Store {
    pub async fn new(lambdas: &Vec<Lambda>) -> Self {
        debug!("Creating new store");
        let mut invocation_queues = HashMap::new();

        debug!("Setting up invocation queues for each lambda. {} lambdas found", lambdas.len());
        for l in lambdas {
            invocation_queues.insert(l.get_name().to_string(), InvocationQueue::new());
            trace!("Invocation queue set up for lambda: {}", l.get_name());
        }
        debug!("Invocation queues set up for each lambda");

        Store {
            queues: Arc::new(RwLock::new(invocation_queues)),
        }
    }
}
