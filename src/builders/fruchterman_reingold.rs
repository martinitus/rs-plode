use ndarray::{s, stack, Array, Array1, Array2, Axis, Dim};
use ndarray_rand::rand::rngs::StdRng;
use ndarray_rand::rand::SeedableRng;
use ndarray_rand::rand_distr::Uniform;
use ndarray_rand::RandomExt;
use ndarray_stats::{MaybeNanExt, QuantileExt};

use crate::{layout::scatter::ScatterLayout, BuildLayout, Graph, Observe};

/// Implements force directed placement by Fruchterman and Reingold.
///
/// Original paper: https://onlinelibrary.wiley.com/doi/epdf/10.1002/spe.4380211102
///
/// This implementation mostly ignores performance considerations and tries to closely follow the
//  pseudo code from the original paper (quote fig.1.).
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
    fn repulsive_force(&self, positions: &Array2<f32>, k: f32) -> Array2<f32> {
        // see page 1136 for details. This is actually pretty important, as otherwise
        // nodes keep getting pushed to the edge of the boundingbox forever.
        let f_r = |r: f32| -> f32 {
            if r < 2. * k {
                k * k / r
            } else {
                0.
            }
        };

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
                &((&delta / &abs_delta) * abs_delta.mapv(f_r)).fold_axis_skipnan(
                    Axis(0),
                    0.,
                    |agr, val| agr + val.const_raw(),
                ),
            );
        }

        disp
    }

    /// Calculate the attractive displacement for each node from their current positions and graph connectivity.
    fn attractive_force(&self, graph: &impl Graph, positions: &Array2<f32>, k: f32) -> Array2<f32> {
        let nodes = graph.nodes();
        let f_a = |r: f32| -> f32 { r * r / k };
        // note: for sparse connections we have a lot of zero terms in the attractive displacements
        //       however, for small graphs (~100 nodes, ~500 edge) performance is still no issue...
        let mut disp = Array2::<f32>::zeros((nodes, 2));
        for (v, u) in graph.edges() {
            let delta = &positions.slice(s![v, ..]) - &positions.slice(s![u, ..]);
            let abs_delta = (&delta * &delta).sum_axis(Axis(0)).into_scalar().sqrt();
            {
                let mut slice = disp.slice_mut(s![v, ..]);
                slice += &(((-1. / f32::max(abs_delta, 1.)) * &delta) * f_a(abs_delta));
            }
            {
                let mut slice = disp.slice_mut(s![u, ..]);
                slice += &(((1. / f32::max(abs_delta, 1.)) * &delta) * f_a(abs_delta));
            }
        }

        disp
    }
}

impl BuildLayout for FruchtermanReingold {
    type Layout = ScatterLayout;

    fn observe<G: Graph>(
        mut self,
        graph: &G,
        observer: &mut impl Observe<G, Self::Layout>,
    ) -> Self::Layout {
        let area = self.width * self.height;
        let k = f32::sqrt(area / graph.nodes() as f32);
        let mut t = self.width / 20.;

        // the positions of the nodes. initialized randomly in 2 dimensions
        let mut pos = stack![
            Axis(1),
            Array1::<f32>::random_using(
                (graph.nodes(),),
                Uniform::new(-self.width / 2., self.width / 2.),
                &mut self.rng,
            ),
            Array1::<f32>::random_using(
                (graph.nodes(),),
                Uniform::new(-self.height / 2., self.height / 2.),
                &mut self.rng,
            )
        ];

        observer.observe(graph, &ScatterLayout::new(&pos));

        for _ in 0..200 {
            // V x D shaped
            let force = self.repulsive_force(&pos, k) + self.attractive_force(graph, &pos, k);
            let force_norm = (&force * &force)
                .sum_axis(Axis(1))
                .map(|x: &f32| f32::sqrt(*x));
            let force_scale = force_norm.mapv(|x: f32| f32::min(t, x));
            let displacement =
                (&force / &force_norm.insert_axis(Axis(1))) * &force_scale.insert_axis(Axis(1));
            pos += &displacement;

            // one could add a little noise to help escape local minima
            //            let mean: f32 = f32::max(k / 20., displacement.mean().unwrap().abs());
            //            pos += &Array2::<f32>::random_using(
            //                (graph.node_count(), 2),
            //                Uniform::new(-mean, mean),
            //                &mut self.rng,
            //            );

            // respect the bounds and guarantee that nodes stay within the configured viewport
            // instead of simply clamping it to the bounding box, we here shift and scale everything
            // to fit back into the bounding box. This may eventually help avoiding a node stalling /
            // getting trapped in a corner.
            let bbox: ((f32, f32), (f32, f32)) = (
                //lower left
                (
                    *pos.slice(s![.., 0]).min().unwrap_or(&-1000.),
                    *pos.slice(s![.., 1]).min().unwrap_or(&-1000.),
                ),
                // upper right
                (
                    *pos.slice(s![.., 0]).max().unwrap_or(&1000.),
                    *pos.slice(s![.., 1]).max().unwrap_or(&1000.),
                ),
            );
            let dims: (f32, f32) = (bbox.1 .0 - bbox.0 .0, bbox.1 .1 - bbox.0 .1);
            // translate to center of coordinate system
            pos = stack![
                Axis(1),
                &pos.slice(s![.., 0]) - bbox.0 .0 - dims.0 / 2.,
                //                    .map(|x| x.clamp(-self.width / 2., self.width / 2.)),
                &pos.slice(s![.., 1]) - bbox.0 .1 - dims.1 / 2.
            ];

            // original clamping method
            //            pos = stack![
            //                Axis(1),
            //                pos.slice(s![.., 0])
            //                    .map(|x| x.clamp(-self.width / 2., self.width / 2.)),
            //                pos.slice(s![.., 1])
            //                    .map(|x| x.clamp(-self.height / 2., self.height / 2.))
            //            ];
            t = t * 0.995;
            observer.observe(graph, &ScatterLayout::new(&pos));
        }

        ScatterLayout::new(&pos)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::builders::fruchterman_reingold::FruchtermanReingold;
    use crate::render::svg::AnimationObserver;
    use crate::test::{defined_graphs, random_graph};
    use svg::Document;

    #[test]
    fn fruchterman_reingold() {
        fn create_animation(graph: &impl Graph, name: &str) {
            println!("Creating animation for {}", name);

            let mut observer = AnimationObserver::new(graph);
            {
                let layout =
                    FruchtermanReingold::new(500., 500., 424).observe(graph, &mut observer);
                svg::save(
                    format!("examples/{}-final.svg", name),
                    &crate::render::svg::render(graph, &layout),
                )
                .unwrap();
            }
            let doc: Document = observer.try_into().unwrap();
            svg::save(format!("examples/{}.svg", name), &doc).unwrap();
        }

        for (name, graph) in defined_graphs() {
            create_animation(&graph, name)
        }

        for n in (10..25).step_by(5) {
            for e in (20..50).step_by(5) {
                create_animation(&random_graph(n, e, 31), &format!("random-{}-{}", n, e));
            }
        }
    }
}
