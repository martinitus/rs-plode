use crate::Graph;
use petgraph::csr::IndexType;
use petgraph::prelude::EdgeRef;
use petgraph::EdgeType;

impl<N, E, Ty, Ix> Graph for petgraph::Graph<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    type Edges = std::vec::IntoIter<(usize, usize)>;

    fn nodes(&self) -> usize {
        petgraph::Graph::node_count(self)
    }

    fn edges(&self) -> Self::Edges {
        let v: Vec<(usize, usize)> = self
            .edge_references()
            .map(|edge| (edge.source().index(), edge.target().index()))
            .collect();
        v.into_iter()
    }
}
