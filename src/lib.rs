extern crate core;

pub mod engines;
pub mod layout;
#[cfg(feature = "petgraph")]
pub mod petgraph;
pub mod render;

/// The algorithm that defines and computes the layout.
pub trait Engine: Sized {
    type Layout<G: Graph>: Sized;
    type LayoutSequence<G: Graph>: Sized;

    fn compute<G: Graph>(self, graph: G) -> Self::Layout<G>;
    fn animate<G: Graph>(self, graph: G) -> Self::LayoutSequence<G>;
}

/// Trait that needs to be implemented for graphs to support layouting.
pub trait Graph: Sized {
    /// The type of the used edge iterator.
    type Edges: Iterator<Item=(usize, usize)>;

    /// The number of nodes of the graph.
    fn nodes(&self) -> usize;

    /// Get the pairs of (source, target) nodes.
    fn edges(&self) -> Self::Edges;

    fn layout<E: Engine>(self, engine: E) -> E::Layout<Self> {
        engine.compute(self)
    }

    fn animate<E: Engine>(self, engine: E) -> E::LayoutSequence<Self> {
        engine.animate(self)
    }
}

impl<T> Graph for &T where T: Graph {
    type Edges = T::Edges;
    fn nodes(&self) -> usize { (*self).nodes() }
    fn edges(&self) -> T::Edges { (*self).edges() }
    fn layout<E: Engine>(self, engine: E) -> E::Layout<Self> { engine.compute(self) }
    fn animate<E: Engine>(self, engine: E) -> E::LayoutSequence<Self> { engine.animate(self) }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::{Rng, SeedableRng, rngs::StdRng};


    #[derive(Clone, Debug)]
    struct E;

    #[derive(Clone, Debug)]
    struct L<G: Graph>(G);

    impl Graph for Vec<(usize, usize)> {
        type Edges = std::vec::IntoIter<(usize, usize)>;

        fn nodes(&self) -> usize {
            // number of nodes is defined by the largest node id we know from the edge list.
            let mut n: usize = 0;
            for (s, t) in self {
                n = usize::max(n, *s);
                n = usize::max(n, *t);
            }
            n + 1
        }

        fn edges(&self) -> Self::Edges {
            return self.clone().into_iter();
        }
    }

    impl Graph for Vec<(u32, u32)> {
        type Edges = std::vec::IntoIter<(usize, usize)>;

        fn nodes(&self) -> usize {
            // number of nodes is defined by the largest node id we know from the edge list.
            let mut n: usize = 0;
            for (s, t) in self {
                n = usize::max(n, *s as usize);
                n = usize::max(n, *t as usize);
            }
            n + 1
        }

        fn edges(&self) -> Self::Edges {
            return self.iter().map(|(s, t)| (*s as usize, *t as usize)).collect::<Vec<(usize, usize)>>().into_iter();
        }
    }

    impl Engine for E {
        type Layout<G: Graph> = L<G>;
        type LayoutSequence<G: Graph> = (G, Vec<L<G>>);

        fn compute<G: Graph>(self, graph: G) -> Self::Layout<G> {
            return L(graph);
        }

        fn animate<G: Graph>(self, graph: G) -> Self::LayoutSequence<G> {
            return (graph, Vec::new());
        }
    }

    #[test]
    fn layout_by_ref_and_value() {
        let graph: Vec<(usize, usize)> = Vec::new();

        fn layout_by_reference<G: Graph>(graph: &G) -> L<&G> {
            graph.layout(E {})
        }

        fn layout_by_value<G: Graph>(graph: G) -> L<G> {
            graph.layout(E {})
        }

        layout_by_reference(&graph);
        layout_by_value(graph);
    }


    /// Create a random graph with given amout of edges and up to given amout of nodes.
    pub fn random_graph(nodes: usize, edges: usize, seed: u64) -> impl Graph {
        let mut rng = StdRng::seed_from_u64(seed);
        (0..edges).map(|_| (rng.gen_range(0..nodes), rng.gen_range(0..nodes))).collect::<Vec<(usize, usize)>>()
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
        let v = graphs.iter().map(|&tpl| { (tpl.0, Vec::from(tpl.1)) }).collect();
        v
    }
}
