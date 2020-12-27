use petgraph::prelude::{Dfs, Direction, Graph};
use petgraph::visit::{GraphBase, Visitable};

use crate::cause_and_effect::Label;
use petgraph::dot::Dot;

type CAEGraph = Graph<Label, ()>;
type CAENodeId = <CAEGraph as GraphBase>::NodeId;
type CAEDfs = Dfs<CAENodeId, <CAEGraph as Visitable>::Map>;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Link {
    pub index: CAENodeId,
    pub label: Label,
}

pub struct CauseVisitor {
    dfs: CAEDfs,
}

impl CauseVisitor {
    fn new(dfs: CAEDfs) -> CauseVisitor {
        CauseVisitor { dfs }
    }

    pub fn next(&mut self, cae: &CauseAndEffect) -> Option<Link> {
        self.dfs.next(&cae.graph).map(|idx| cae.get(idx))
    }
}

#[derive(Debug)]
pub struct CauseAndEffect {
    graph: CAEGraph,
    root: CAENodeId,
}

impl CauseAndEffect {
    #[must_use]
    pub fn new() -> CauseAndEffect {
        let mut graph = CAEGraph::new();
        let root = graph.add_node(Label::Root);
        CauseAndEffect { graph, root }
    }

    pub fn new_turn(&mut self) {
        self.graph.clear();
        self.root = self.graph.add_node(Label::Root);
    }

    fn add_link(&mut self, cause: CAENodeId, effect: Label) -> Link {
        let u = self.graph.add_node(effect);
        self.graph.add_edge(cause, u, ());
        Link {
            index: u,
            label: effect,
        }
    }

    pub fn get_root(&self) -> Link {
        self.get(self.root)
    }

    pub fn get_cause(&self, effect: &Link) -> Option<Link> {
        self.graph
            .neighbors_directed(effect.index, Direction::Incoming)
            .map(|id| self.get(id))
            .next()
    }

    pub fn scan(&self) -> CauseVisitor {
        CauseVisitor::new(Dfs::new(&self.graph, self.root))
    }

    pub fn add_effect(&mut self, cause: &Link, effect: Label) -> Link {
        self.add_link(cause.index, effect)
    }

    fn get(&self, u: CAENodeId) -> Link {
        Link {
            index: u,
            label: *self.graph.node_weight(u).unwrap(),
        }
    }

    pub fn find_first_link<F>(&self, filter: F) -> Option<Link>
    where
        F: Fn(Link) -> bool,
    {
        let mut s = self.scan();
        while let Some(n) = s.next(self) {
            if filter(n) {
                return Some(n);
            }
        }
        None
    }

    pub fn find_first_ancestor<F>(&self, effect: &Link, filter: F) -> Option<Link>
    where
        F: Fn(Link) -> bool,
    {
        let mut u = *effect;
        while let Some(v) = self.get_cause(&u) {
            if filter(v) {
                return Some(v);
            } else {
                u = v;
            }
        }
        None
    }

    pub fn dot(&self) -> Dot<&CAEGraph> {
        petgraph::dot::Dot::with_config(&self.graph, &[petgraph::dot::Config::EdgeNoLabel])
    }
}

impl Default for CauseAndEffect {
    fn default() -> Self {
        CauseAndEffect::new()
    }
}
