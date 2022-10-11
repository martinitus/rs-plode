use petgraph::csr::IndexType;
use petgraph::EdgeType;
use petgraph::prelude::EdgeRef;
use crate::Graph;

impl<N, E, Ty, Ix> Graph for petgraph::Graph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType
{
    fn node_count(&self) -> usize {
        petgraph::Graph::node_count(self)
    }
    fn edges(&self) -> Vec<(usize, usize)> {
        self.edge_references()
            .map(
                |edge| { (edge.source().index(), edge.target().index()) })
            .collect()
    }
}