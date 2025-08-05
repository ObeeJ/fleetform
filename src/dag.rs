use petgraph::Graph;
use petgraph::graph::NodeIndex;
use std::collections::HashMap;

pub struct ResourceGraph {
    graph: Graph<String, ()>,
    nodes: HashMap<String, NodeIndex>,
}

impl ResourceGraph {
    pub fn new() -> Self {
        ResourceGraph {
            graph: Graph::new(),
            nodes: HashMap::new(),
        }
    }

    pub fn add_resource(&mut self, name: &str) -> NodeIndex {
        let idx = self.graph.add_node(name.to_string());
        self.nodes.insert(name.to_string(), idx);
        idx
    }

    pub fn add_dependency(&mut self, from: &str, to: &str) {
        let from_idx = *self.nodes.get(from).expect("Source resource not found");
        let to_idx = *self.nodes.get(to).expect("Target resource not found");
        self.graph.add_edge(from_idx, to_idx, ());
    }

    pub fn get_ordered_resources(&self) -> Vec<String> {
        use petgraph::algo::toposort;
        let sorted = toposort(&self.graph, None).expect("Cycle detected in resource graph");
        sorted.into_iter().map(|idx| self.graph[idx].clone()).collect()
    }

    pub fn resource_count(&self) -> usize {
        self.graph.node_count()
    }
}
