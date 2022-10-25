use ndarray::{s, stack, Array2, Axis, Array3, ArrayView, ArrayView2};

use ndarray_stats::QuantileExt;

use crate::{Graph};

use super::{BoundingBox, Point};

/// A layout where nodes can have a real valued position in 2D space.
#[derive(Clone, Debug)]
pub struct ScatterLayout<G: Graph> {
    positions: Array2<f32>,
    pub(crate) graph: G,
    bbox: BoundingBox,
}

impl<G: Graph> ScatterLayout<G> {
    pub fn new(graph: G, positions: Array2<f32>) -> Result<Self, String> {
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


/// A sequence of scatter layouts that represent the progress during layouting.
pub struct ScatterLayoutSequence<G: Graph> {
    positions: Array3<f32>,
    pub(crate) graph: G,
    bbox: BoundingBox,
}


impl<G: Graph> ScatterLayoutSequence<G> {
    pub fn new(graph: G, positions: Vec<Array2<f32>>) -> Result<Self, String> {
        if positions.len() == 0 {
            return Err("Need at least one step".to_string());
        }

        if positions.iter().any(|frame| frame.shape()[0] != graph.nodes()) {
            return Err(
                format!("Node count {} does not match layout shape for all frames", graph.nodes()).to_string()
            );
        }

        let positions = ndarray::stack(
            Axis(0),
            positions
                .iter()
                .map(ArrayView::from)
                .collect::<Vec<_>>()
                .as_slice())
            .map_err(|_| "Shape mismatch between individual frames.".to_string())?;

        let bbox = BoundingBox(
            Point(
                *positions
                    .slice(s![..,.., 0])
                    .min()
                    .map_err(|_| "Found NaN in positions".to_string())?,
                *positions
                    .slice(s![..,.., 1])
                    .min()
                    .map_err(|_| "Found NaN in positions".to_string())?,
            ),
            Point(
                *positions
                    .slice(s![..,.., 0])
                    .max()
                    .map_err(|_| "Found NaN in positions".to_string())?,
                *positions
                    .slice(s![..,.., 1])
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

    /// The number of individual layout frames in the sequence.
    pub fn frames(&self) -> usize {
        return self.positions.shape()[0];
    }

    pub fn frame(&self, f: usize) -> ArrayView2<f32> {
        return self.positions.slice(s![f,..,..]);
    }

    /// The bounding box that encompasses all nodes.
    /// Returns lower left and upper right corner.
    pub fn bbox(&self) -> &BoundingBox {
        return &self.bbox;
    }

    /// Get the location of a node.
    pub fn coord(&self, frame: usize, node: usize) -> Point {
        return Point(self.positions[[frame, node, 0]], self.positions[[frame, node, 1]]);
    }

    /// Translate and scale to match given target bounding box
    pub fn transform(mut self, bbox: &BoundingBox) -> Self {
        self.positions = stack![
            Axis(2),
            &(&self.positions.slice(s![..,.., 0]) - self.bbox().lower_left().x()) * bbox.width()
                / self.bbox().width()
                + bbox.lower_left().x(),
            &(&self.positions.slice(s![..,.., 1]) - self.bbox().lower_left().y()) * bbox.height()
                / self.bbox().height()
                + bbox.lower_left().y()
        ];
        self
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
            ScatterLayout::new(random_graph(2, 2, 2), arr2(&[[0., 0.], [0., 0.]]) / 0.).is_err()
        );
    }

    #[test]
    fn fail_on_inf() {
        assert!(
            ScatterLayout::new(random_graph(2, 2, 2), arr2(&[[1., 1.], [1., 1.]]) / 0.).is_err()
        );
    }

    #[test]
    fn fail_on_count_mismatch() {
        assert!(ScatterLayout::new(
            random_graph(2, 2, 2),
            arr2(&[[1., 1.], [1., 1.], [1., 1.]]),
        )
            .is_err());
    }

    #[test]
    fn success() {
        ScatterLayout::new(random_graph(2, 2, 2), arr2(&[[0., 0.], [1., 1.]])).unwrap();
        assert!(ScatterLayout::new(random_graph(2, 2, 2), arr2(&[[0., 0.], [1., 1.]])).is_ok());
    }
}
