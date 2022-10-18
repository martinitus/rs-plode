use ndarray::{s, stack, Array2, Axis};

use ndarray_stats::QuantileExt;

use crate::{Graph, Layout};

use super::{BoundingBox, Point};

/// A layout where nodes can have a real valued position in 2D space.
#[derive(Clone, Debug)]
pub struct ScatterLayout<'a, G: Graph> {
    positions: Array2<f32>,
    graph: &'a G,
    bbox: BoundingBox,
}

impl<'a, G: Graph> ScatterLayout<'a, G> {
    pub fn new(graph: &'a G, positions: Array2<f32>) -> Result<Self, String> {
        if positions.shape()[0] != graph.nodes() {
            return Err(format!(
                "Node count {} does not match position shape {}",
                graph.nodes(),
                positions.shape()[0]
            )
            .to_string());
        }
        let bbox = BoundingBox(
            Point(
                *positions
                    .slice(s![.., 0])
                    .min()
                    .map_err(|_| "Found NaN in positions".to_string())?,
                *positions
                    .slice(s![.., 1])
                    .min()
                    .map_err(|_| "Found NaN in positions".to_string())?,
            ),
            Point(
                *positions
                    .slice(s![.., 0])
                    .max()
                    .map_err(|_| "Found NaN in positions".to_string())?,
                *positions
                    .slice(s![.., 1])
                    .max()
                    .map_err(|_| "Found NaN in positions".to_string())?,
            ),
        );

        if [
            bbox.lower_left().x(),
            bbox.lower_left().y(),
            bbox.upper_right().x(),
            bbox.upper_right().y(),
        ]
        .into_iter()
        .any(f32::is_infinite)
        {
            return Err("Infinite size bounding box.".to_string());
        }

        Ok(Self {
            positions,
            graph,
            bbox,
        })
    }

    /// The bounding box that encompasses all nodes.
    /// Returns lower left and upper right corner.
    pub fn bbox(&self) -> &BoundingBox {
        return &self.bbox;
    }

    /// Get the location of a node.
    pub fn coord(&self, node: usize) -> Point {
        return Point(self.positions[[node, 0]], self.positions[[node, 1]]);
    }

    //    /// Translate by given vector. If no vector provided, translate such that the result is centered around the origin.
    //    pub fn translate(self, t: Option<(f32, f32)>) -> Self {
    //        let (tx, ty) = t.unwrap_or((
    //            -self.bbox().0 .0 - self.bbox().width() / 2.,
    //            -self.bbox().0 .1 - self.bbox().height() / 2.,
    //        ));
    //        return Self {
    //            graph: self.graph,
    //            positions: stack![
    //                Axis(1),
    //                &self.positions.slice(s![.., 0]) + tx,
    //                &self.positions.slice(s![.., 1]) + ty
    //            ],
    //        };
    //    }

    /// Translate and scale to match given target bounding box
    pub fn transform(mut self, bbox: &BoundingBox) -> Self {
        self.positions = stack![
            Axis(1),
            &(&self.positions.slice(s![.., 0]) - self.bbox().lower_left().x()) * bbox.width()
                / self.bbox().width()
                + bbox.lower_left().x(),
            &(&self.positions.slice(s![.., 1]) - self.bbox().lower_left().y()) * bbox.height()
                / self.bbox().height()
                + bbox.lower_left().y()
        ];
        self
    }
}

impl<'a, G: Graph> Layout<G> for ScatterLayout<'a, G> {
    fn graph(&self) -> &'a G {
        self.graph
    }
}

#[cfg(test)]
mod test {
    use ndarray::arr2;

    use crate::test::random_graph;

    use super::ScatterLayout;

    #[test]
    fn fail_on_nan() {
        assert!(
            ScatterLayout::new(&random_graph(2, 2, 2), arr2(&[[0., 0.], [0., 0.]]) / 0.).is_err()
        );
    }

    #[test]
    fn fail_on_inf() {
        assert!(
            ScatterLayout::new(&random_graph(2, 2, 2), arr2(&[[1., 1.], [1., 1.]]) / 0.).is_err()
        );
    }

    #[test]
    fn fail_on_count_mismatch() {
        assert!(ScatterLayout::new(
            &random_graph(2, 2, 2),
            arr2(&[[1., 1.], [1., 1.], [1., 1.]])
        )
        .is_err());
    }

    #[test]
    fn success() {
        assert!(ScatterLayout::new(&random_graph(2, 2, 2), arr2(&[[0., 0.], [1., 1.]])).is_ok());
    }
}
