extern crate core;

pub mod engines;
pub mod layout;
#[cfg(feature = "petgraph")]
pub mod petgraph;
pub mod render;

pub trait Layout<G: Graph>: Clone {
    fn graph(&self) -> &G;
}

/// The algorithm that defines and computes the layout.
pub trait Engine: Sized {
    type Layout<'a, G>: Layout<G>
    where
        G: Graph,
        G: 'a;
    type LayoutSequence<'a, G>: Iterator<Item = Self::Layout<'a, G>>
    where
        G: Graph,
        G: 'a;

    fn compute<'a, G: Graph>(self, graph: &'a G) -> Self::Layout<'a, G> {
        return self.animate(graph).last().unwrap();
    }
    fn animate<'a, G: Graph>(self, graph: &'a G) -> Self::LayoutSequence<'a, G>;
}

/// Trait that needs to be implemented for graphs to support layouting.
pub trait Graph: Sized + Clone {
    /// The type of the used edge iterator.
    type Edges: Iterator<Item = (usize, usize)>;

    /// The number of nodes of the graph.
    fn nodes(&self) -> usize;

    /// Get the pairs of (source, target) nodes.
    fn edges(&self) -> Self::Edges;

    fn layout<'a, E: Engine>(&'a self, engine: E) -> E::Layout<'a, Self> {
        engine.compute(self)
    }

    fn animate<'a, E: Engine>(&'a self, engine: E) -> E::LayoutSequence<'a, Self> {
        engine.animate(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ::petgraph::graph::UnGraph;
    use ndarray::{Array2, Axis};
    use ndarray_rand::{
        rand::{prelude::StdRng, SeedableRng},
        rand_distr::Uniform,
        RandomExt,
    };

    /// Create a random graph with given amout of edges and up to given amout of nodes.
    pub fn random_graph(nodes: usize, edges: usize, seed: u64) -> impl Graph {
        let mut rng = StdRng::seed_from_u64(seed);
        UnGraph::<(), ()>::from_edges(
            Array2::<u32>::random_using((edges, 2), Uniform::new(0, nodes as u32), &mut rng)
                .axis_iter(Axis(0))
                .map(|a| (a[0], a[1])),
        )
    }

    /// Some predefined regular graphs helpful for testing and demonstration.
    #[rustfmt::skip]
    pub fn defined_graphs() -> Vec<(&'static str, impl Graph)> {
        let graphs: Vec<(&'static str, &'static [(u32, u32)])> = vec![
            ("triangle", &[(0, 1), (1, 2), (2, 0)]),
            ("square", &[(0, 1), (1, 2), (2, 3), (3, 0)]),
            ("pentagon", &[(0, 1), (1, 2), (2, 3), (3, 4), (4, 0)]),
            ("tetrahedron", &[(0, 1), (1, 2), (2, 0), (0, 3), (1, 3), (2, 3)]),
            ("custom", &[(0, 1), (1, 2), (2, 3), (3, 0), (0, 2), (1, 2), (2, 4), (2, 5), (4, 5)]),
            ("cube", &[
                    // plane 1
                    (0, 1), (1, 2), (2, 3), (3, 0),
                    // plane 1
                    (4, 5), (5, 6), (6, 7), (7, 4),
                    // plane connections
                    (0, 4), (1, 5), (2, 6), (3, 7),
                ],
            ),
            (
                "tree",
                &[
                    // root->level1
                    (0, 1), (0, 2),
                    // level1->level2
                    (1, 3), (1, 4), (1, 5), (2, 6), (2, 7),
                    // level2->level3
                    (3, 8), (4, 9), (4, 10), (6, 11), (6, 12), (6, 13), (7, 14),
                    // level3->level4
                    (14, 15), (14, 16), (14, 17), (14, 18), (14, 19)
                ],
            ),
            (
                "prism",
                &[
                    // plane 1
                    (0, 1), (1, 2), (2, 0),
                    // plane 2
                    (3, 4), (4, 5), (5, 3),
                    // connections
                    (0, 3), (1, 4), (2, 5),
                ],
            ),
            (
                "pentagram",
                &[
                    // pentagon
                    (0, 1), (1, 2), (2, 3), (3, 4), (4, 0),
                    // diagonals
                    (0, 2), (1, 3), (2, 4), (3, 0), (4, 1),
                ],
            ),
            (
                "disconnected-components",
                &[
                    // triangle 1
                    (0, 1), (1, 2), (2, 0),
                    // triangle 2
                    (3, 4), (4, 5), (5, 3),
                ],
            ),
            (
                "triangulated-triangle",
                &[
                    // outer edge 1
                    (0, 1), (1, 2), (2, 3),
                    // outer edge 2
                    (3, 4), (4, 5), (5, 6),
                    // outer edge 3
                    (6, 7), (7, 8), (8, 0),
                    // cut edges
                    (1, 8), (2, 4), (5, 7),
                    // connections to center
                    (1, 9), (2, 9), (4, 9), (5, 9), (7, 9), (8, 9),
                ],
            ),
        ];
        let v = graphs.iter().map(|&tpl|{(tpl.0, UnGraph::<(), ()>::from_edges(tpl.1))}).collect();
        v
   }
}
