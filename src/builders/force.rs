use ndarray::{s, stack, Array, Array2, Axis, Dim, Array1};
use ndarray_rand::rand::rngs::StdRng;
use ndarray_rand::rand::SeedableRng;
use ndarray_rand::rand_distr::Uniform;
use ndarray_rand::RandomExt;
use ndarray_stats::MaybeNanExt;

use crate::{BuildLayout, Graph, Observer};

/// Implements force directed placement by Fruchterman and Reingold.
///
/// Original paper: https://onlinelibrary.wiley.com/doi/epdf/10.1002/spe.4380211102
/// Mostly ignore performance and try to closely follow the pseudo code from the original paper (quote fig.1.).
/// ```text
///   area := W * L; { W and L are the width and length of the frame }
///   k := sqrt(area/|V|)
///   G := (V, E); { the vertices are assigned random initial positions }
///   function f_a(x) := begin return x^2/k end;
///   function f_r(x) := begin return k^2/x end;
///   for i := 1 to iterations do begin
///        { calculate repulsive forces }
///        for v in V do begin
///            { each vertex has two vectors: .pos and .disp }
///            v.disp := 0;
///            for u in V do
///                if (u != v) then begin
///                    { Δ is short hand for the difference}
///                    { vector between the positions of the two vertices }
///                    Δ := v.pos - u.pos
///                    v.disp := v.disp + (Δ/|Δ|) * f_r(|Δ|)
///            end
///        end
///
///        { calculate attractive forces }
///        for e in E do begin
///            { each edge is an ordered pair of vertices .v and .u }
///            Δ := e.v.pos - e.u.pos;
///            e.v.disp := e.v.disp - (Δ/|Δ|) * f_a(|Δ|)
///            e.u.disp := e.u.disp + (Δ/|Δ|) * f_a(|Δ|)
///         end
///
///        { limit the maximum displacement to the temperature t }
///        { and then prevent from being displaced outside frame }
///        for v in V do begin
///            v.pos := v.pos + (v.disp / |v.disp| ) * min(v.disp, t)
///            v.pos.x := min(W/2, max(-W/2, v.pos.x));
///            v.pos.y := min(L/2, max(- L/2, v.pos.y))
///        end
///        { reduce the temperature as the layout approaches a better configuration }
///        t := cool(t)
///   end
/// ```
pub struct FruchtermanReingold {
    width: f32,
    height: f32,
    rng: StdRng,
}

impl FruchtermanReingold {
    pub fn new(width: f32, length: f32, seed: u64) -> Self {
        Self {
            width,
            height: length,
            rng: StdRng::seed_from_u64(seed),
        }
    }

    /// Calculate the repulsive displacements for each node from their current positions.
    fn repulsive_force(positions: &Array2<f32>, k: f32) -> Array2<f32> {
        let f_r = |x: &f32| -> f32 { k * k / x };
        let nodes = positions.shape()[0];
        // V x 2 shaped displacements for all nodes
        let mut disp = Array2::<f32>::zeros((nodes, 2));

        // repulsive displacements for each node
        for j in 0..nodes {
            // V x D shaped matrix of delta vectors from node j to all other nodes.
            let delta: Array<f32, Dim<[usize; 2]>> = &positions.slice(s![j, ..]) - positions;
            // V x 1 shaped matrix holding the absolute distance between v and each other vertex
            let abs_delta: Array<f32, Dim<[usize; 2]>> = (&delta * &delta)
                .sum_axis(Axis(1))
                .map(|x: &f32| f32::sqrt(*x))
                .insert_axis(Axis(1));

            disp.slice_mut(s![j, ..]).assign(
                // V x 2 shaped displacements for node j caused by all other nodes.
                &((&delta / &abs_delta) * abs_delta.map(f_r)).fold_axis_skipnan(
                    Axis(0),
                    0.,
                    |agr, val| agr + val.const_raw(),
                ),
            );
        }

        disp
    }

    /// Calculate the attractive displacement for each node from their current positions and graph connectivity.
    fn attractive_force(graph: &impl Graph, positions: &Array2<f32>, k: f32) -> Array2<f32> {
        let nodes = graph.node_count();
        let f_a = |x: &f32| -> f32 { x * x / k };
        // fixme: for sparse connections we have a lot of zero terms in the attractive displacements
        let mut disp = Array2::<f32>::zeros((nodes, 2));
        for (v, u) in graph.edges() {
            let delta = &positions.slice(s![v, ..]) - &positions.slice(s![u, ..]);
            let abs_delta = (&delta * &delta).sum_axis(Axis(0)).into_scalar().sqrt();
            disp.slice_mut(s![v, ..])
                .assign(&(((-1. / abs_delta) * &delta) * f_a(&abs_delta)));
            disp.slice_mut(s![u, ..])
                .assign(&(((1. / abs_delta) * &delta) * f_a(&abs_delta)));
        }
        disp
    }
}

impl BuildLayout for FruchtermanReingold {
    type Layout = Array2<f32>;

    fn observe<G: Graph>(
        mut self,
        graph: &G,
        observer: &mut impl Observer<G, Self::Layout>,
    ) -> Self::Layout {
        let area = self.width * self.height;
        let k = f32::sqrt(area / graph.node_count() as f32);
        let mut t = self.width / 10.;

        // the positions of the nodes. initialized randomly in 2 dimensions
        let mut pos = stack![
            Axis(1),
            Array1::<f32>::random_using(
                (graph.node_count(),),
                Uniform::new(0., self.width),
                &mut self.rng,
            ),
            Array1::<f32>::random_using(
                (graph.node_count(),),
                Uniform::new(0., self.height),
                &mut self.rng,
            )
        ];

        observer.observe(graph, &pos);

        for n in 0..50 {
            // V x D shaped
            let force = FruchtermanReingold::repulsive_force(&pos, k)
                + FruchtermanReingold::attractive_force(graph, &pos, k);
            let force_norm = (&force * &force)
                .sum_axis(Axis(1))
                .map(|x: &f32| f32::sqrt(*x));
            let force_scale = force_norm.map(|x: &f32| f32::min(t, *x));
            let displacement =
                (&force / &force_norm.insert_axis(Axis(1))) * &force_scale.insert_axis(Axis(1));
            pos += &displacement;

            // respect the bounds and guarantee that nodes stay within the configured viewport
            pos = stack![
                Axis(1),
                pos.slice(s![.., 0]).map(|x| x.clamp(0., self.width)),
                pos.slice(s![.., 1]).map(|x| x.clamp(0., self.height))
            ];
            t = self.width / (n as f32 + 10.);
            observer.observe(graph, &pos);
        }

        pos
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::builders::force::FruchtermanReingold;
    use crate::render::svg::AnimationObserver;
    use ::petgraph::graph::UnGraph;
    use svg::Document;

    /// Create a random graph with given amout of edges and up to given amout of nodes.
    fn random_graph(nodes: usize, edges: usize) -> UnGraph<u32, ()> {
        UnGraph::<u32, ()>::from_edges(
            Array2::<u32>::random((edges, 2), Uniform::new(0, nodes as u32))
                .axis_iter(Axis(0))
                .map(|a| (a[0], a[1])),
        )
    }

    #[test]
    fn spring_force_layout() {
        // Create an undirected graph with `i32` nodes and edges with `()` associated data.
        let graph = UnGraph::<u32, ()>::from_edges(&[
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 0),
            (0, 2),
            (1, 2),
            (2, 4),
            (2, 5),
            (4, 5),
        ]);
        let graph = random_graph(20, 100);

        let _layout = FruchtermanReingold::new(800., 800., 420).build(&graph);

        let mut c = 0;
        let mut observer = |g: &UnGraph<u32, ()>, l: &Array2<f32>| {
            let document = crate::render::svg::render(g, l);
            svg::save(format!("image-{}.svg", c), &document).unwrap();
            c += 1;
        };
        let _layout = FruchtermanReingold::new(200., 200., 420).observe(&graph, &mut observer);

        let mut animationobs = AnimationObserver::new(&graph);
        {
            let _layout = FruchtermanReingold::new(200., 200., 420).observe(&graph, &mut animationobs);
        }
        let doc: Document = animationobs.into();
        svg::save("animation.svg", &doc).unwrap();
        // let mut doc = document();
        // {
        //     let mut animation_observer = animation(&mut doc);
        //     ForceLayout::new(800., 800., 420).observe(&graph, &mut animation_observer);
        // }
        // svg::save("images.svg", &doc).unwrap();
    }
}
