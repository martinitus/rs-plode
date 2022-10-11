pub mod builders;

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



pub trait BuildLayout {
    type Layout;

    fn build(self, graph: &impl Graph) -> Self::Layout
        where Self: Sized
    {
        self.observe(graph, |_| {})
    }

    fn observe(self, graph: &impl Graph, observer: impl FnMut(Self::Layout)) -> Self::Layout;
}


#[cfg(test)]
mod test {
    use ::petgraph::graph::UnGraph;
    use super::*;
    use crate::builders::force::ForceLayout;

    #[test]
    fn public_api() {
        // Create an undirected graph with `i32` nodes and edges with `()` associated data.
        let graph = UnGraph::<i32, ()>::from_edges(&[
            (1, 2), (2, 3), (3, 4), (1, 4)
        ]);
        let _layout: <ForceLayout as BuildLayout>::Layout = ForceLayout::new(10., 10., 32).build(&graph);
    }
}