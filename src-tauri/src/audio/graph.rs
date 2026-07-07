use fundsp::hacker::*;
use serde::Serialize;

/// Whether the node goes in the pre-fx or post-fx chain.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum NodeType {
    PreFx,
    PostFx,
}

/// A node in the processing chain with metadata.
#[derive(Clone, Debug)]
pub struct NodeHandle {
    pub id: usize,
    pub name: String,
    pub node_type: NodeType,
    pub enabled: bool,
}

/// The audio processing graph built with fundsp.
///
/// Maintains two chains:
/// - **Pre-fx**: applied before the mix engine (per-track processing).
/// - **Post-fx**: applied after the mix engine (master processing).
pub struct AudioGraph {
    /// Sample rate the graph operates at.
    sample_rate: f64,
    /// Registered nodes metadata.
    nodes: Vec<NodeHandle>,
    /// The compiled fundsp graph, rebuilt when topology changes.
    compiled: Option<An<Pass>>,
}

impl AudioGraph {
    /// Create a new audio graph at the given sample rate (Hz).
    pub fn new(sample_rate: f64) -> Self {
        Self {
            sample_rate,
            nodes: Vec::new(),
            compiled: None,
        }
    }

    /// Update the sample rate (invalidates the compiled graph).
    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
        self.compiled = None;
    }

    /// Register a new processing node in the graph.
    pub fn add_node(&mut self, name: String, node_type: NodeType) -> NodeHandle {
        let id = self.nodes.len();
        let handle = NodeHandle {
            id,
            name,
            node_type,
            enabled: true,
        };
        self.nodes.push(handle.clone());
        self.compiled = None;
        handle
    }

    /// Remove a node by ID.
    pub fn remove_node(&mut self, id: usize) {
        self.nodes.retain(|n| n.id != id);
        self.compiled = None;
    }

    /// Enable or disable a node (bypassed when disabled).
    pub fn set_node_enabled(&mut self, id: usize, enabled: bool) {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == id) {
            node.enabled = enabled;
        }
        self.compiled = None;
    }

    /// Reorder nodes by providing a new ID sequence.
    pub fn reorder_nodes(&mut self, order: &[usize]) {
        let mut reordered: Vec<NodeHandle> = Vec::with_capacity(order.len());
        for &id in order {
            if let Some(pos) = self.nodes.iter().position(|n| n.id == id) {
                reordered.push(self.nodes.remove(pos));
            }
        }
        reordered.extend(self.nodes.drain(..));
        self.nodes = reordered;
        self.compiled = None;
    }

    /// List all registered nodes.
    pub fn nodes(&self) -> &[NodeHandle] {
        &self.nodes
    }

    /// Build (or return cached) compiled fundsp graph.
    ///
    /// The default graph is a simple pass-through.
    /// As plugin nodes are registered, they will be wired into the chain here.
    pub fn get_graph(&mut self) -> An<Pass> {
        if self.compiled.is_none() {
            self.compiled = Some(pass());
        }
        self.compiled.as_ref().unwrap().clone()
    }

    /// Force a rebuild of the graph on the next `get_graph` call.
    pub fn invalidate(&mut self) {
        self.compiled = None;
    }
}
