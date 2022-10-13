extern crate core;

pub mod builders;

pub mod layout;
#[cfg(feature = "petgraph")]
pub mod petgraph;
pub mod render;

pub trait Graph {
    fn node_count(&self) -> usize;

    /// Get the pairs of (source, target) nodes.
    /// For undirected graphs edges should be represented twice (u -> v) and (v -> u),
    /// otherwise, e.g. force layouts of a directed vs undirected graphs would behave differently.
    fn edges(&self) -> Vec<(usize, usize)>;
}

pub trait Observer<G, L>
where
    G: Graph,
{
    fn observe(&mut self, graph: &G, layout: &L);
}

impl<G, L, F> Observer<G, L> for F
where
    F: FnMut(&G, &L),
    G: Graph,
{
    fn observe(&mut self, graph: &G, layout: &L) {
        self(graph, layout)
    }
}

pub trait BuildLayout {
    type Layout;

    fn build<G: Graph>(self, graph: &G) -> Self::Layout
    where
        Self: Sized,
    {
        self.observe(graph, &mut |_: &G, _: &Self::Layout| {})
    }

    fn observe<G: Graph>(
        self,
        graph: &G,
        observer: &mut impl Observer<G, Self::Layout>,
    ) -> Self::Layout;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::builders::force::FruchtermanReingold;
    use ::petgraph::graph::UnGraph;

    #[test]
    fn public_api() {
        // Create an undirected graph with `i32` nodes and edges with `()` associated data.
        let graph = UnGraph::<i32, ()>::from_edges(&[(1, 2), (2, 3), (3, 4), (1, 4)]);
        let _layout: <FruchtermanReingold as BuildLayout>::Layout =
            FruchtermanReingold::new(10., 10., 32).build(&graph);
    }
}
