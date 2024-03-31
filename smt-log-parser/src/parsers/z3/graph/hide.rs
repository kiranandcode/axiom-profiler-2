use petgraph::{graph::{DiGraph, EdgeReference, NodeIndex}, visit::{Bfs, EdgeFiltered, EdgeRef, Reversed, ReversedEdgeReference, Walker}};

use super::{raw::{EdgeKind, Node, NodeState, RawInstGraph}, InstGraph};

impl RawInstGraph {
    pub fn reset_visibility_to(&mut self, hidden: bool) {
        let state = if hidden { NodeState::Hidden } else { NodeState::Visible };
        for node in self.graph.node_weights_mut().filter(|n| !n.disabled()) {
            self.stats.set_state(node, state);
        }
    }
    pub fn set_visibility(&mut self, hidden: bool, node: NodeIndex) {
        let node = &mut self.graph[node];
        if node.disabled() {
            return;
        }
        self.stats.set_state(node, if hidden { NodeState::Hidden } else { NodeState::Visible });
    }

    pub fn keep_first_n(&mut self, nodes: impl Iterator<Item = NodeIndex>, mut n: usize) {
        for node in nodes {
            if n == 0 {
                self.set_visibility(true, node);
            } else if self.graph[node].visible() {
                n -= 1;
            }
        }
    }

    /// When predicate `p` evaluates to true the visibility of the corresponding
    /// node is set to `hidden`.
    pub fn set_visibility_when(&mut self, hidden: bool, mut p: impl FnMut(NodeIndex, &Node) -> bool) {
        for node in self.graph.node_indices() {
            let n = &self.graph[node];
            if n.disabled() {
                continue;
            }
            if p(node, n) {
                self.set_visibility(hidden, node);
            }
        }
    }
    pub fn set_visibility_many(&mut self, hidden: bool, nodes: impl Iterator<Item = NodeIndex>) {
        for node in nodes {
            self.set_visibility(hidden, node);
        }
    }

    fn filter_path(&self, edge: impl EdgeRef<NodeId = NodeIndex>, f: impl Fn(&Node) -> u32) -> bool {
        let from = &self.graph[edge.source()];
        let to = &self.graph[edge.target()];
        if from.disabled() {
            f(from) == f(to)
        } else {
            f(from) == f(to) + 1
        }
    }
    /// A graph with edges that aren't part of any `longest/shortest` path to a
    /// root filtered out. The edges are also reversed, so the graph can be
    /// walked from any node to find the longest/shortest path to a root.
    pub fn path_to_root_graph<'a>(&'a self, longest: bool) -> EdgeFiltered<Reversed<&'a DiGraph<Node, EdgeKind>>, impl Fn(ReversedEdgeReference<EdgeReference<EdgeKind>>) -> bool + 'a> {
        let f = move |depth: &Node| if longest { depth.fwd_depth.max } else { depth.fwd_depth.min };
        let filter = move |edge: ReversedEdgeReference<EdgeReference<EdgeKind>>| self.filter_path(edge, f);
        EdgeFiltered::from_fn(self.rev(), filter)
    }
    /// A graph with edges that aren't part of any `longest/shortest` path to a
    /// leaf filtered out. The graph can be walked from any node to find the
    /// longest/shortest path to a leaf.
    pub fn path_to_leaf_graph<'a>(&'a self, longest: bool) -> EdgeFiltered<&'a DiGraph<Node, EdgeKind>, impl Fn(EdgeReference<EdgeKind>) -> bool + 'a> {
        let f = move |depth: &Node| if longest { depth.bwd_depth.max } else { depth.bwd_depth.min };
        let filter = move |edge: EdgeReference<EdgeKind>| self.filter_path(edge, f);
        EdgeFiltered::from_fn(&self.graph, filter)
    }

    pub fn show_longest_path_through(&mut self, node: NodeIndex) -> Vec<NodeIndex> {
        let mut path: Vec<_> = {
            let to_root = self.path_to_root_graph(true);
            Bfs::new(&to_root, node).iter(&to_root).collect()
        };
        path.reverse();
        {
            let to_leaf = self.path_to_leaf_graph(true);
            let to_leaf = Bfs::new(&to_leaf, node).iter(&to_leaf).skip(1);
            path.extend(to_leaf);
        };
        path.retain(|&n| !self.graph[n].disabled());
        self.set_visibility_many(false, path.iter().copied());
        path
    }
}

impl InstGraph {
    pub fn keep_first_n_cost(&mut self, n: usize) {
        self.raw.keep_first_n(self.analysis.cost.iter().copied(), n)
    }
    pub fn keep_first_n_children(&mut self, n: usize) {
        self.raw.keep_first_n(self.analysis.children.iter().copied(), n)
    }
    pub fn keep_first_n_fwd_depth_min(&mut self, n: usize) {
        self.raw.keep_first_n(self.analysis.fwd_depth_min.iter().copied(), n)
    }
    // pub fn keep_first_n_max_depth(&mut self, n: usize) {
    //     self.raw.keep_first_n(self.analysis.max_depth.iter().copied(), n)
    // }
}