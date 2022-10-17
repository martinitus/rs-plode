use ndarray::{s, stack, Array2, Axis};

use ndarray_stats::QuantileExt;

/// A layout where nodes can have a real valued position in 2D space.
#[derive(Clone, Debug)]
pub struct ScatterLayout {
    positions: Array2<f32>
}

impl ScatterLayout {
    pub fn new(positions: &Array2<f32>) -> Self {
        Self {
            positions: positions.clone(),
        }
    }

    /// The bounding box that encompasses all nodes.
    /// Returns lower left and upper right corner.
    pub fn bbox(&self) -> ((f32, f32), (f32, f32)) {
        return (
            (
                *self.positions.slice(s![.., 0]).min().unwrap(),
                *self.positions.slice(s![.., 1]).min().unwrap(),
            ),
            (
                *self.positions.slice(s![.., 0]).max().unwrap(),
                *self.positions.slice(s![.., 1]).max().unwrap(),
            ),
        );
    }

    pub fn width(&self) -> f32 {
        self.bbox().1 .0 - self.bbox().0 .0
    }

    pub fn height(&self) -> f32 {
        self.bbox().1 .1 - self.bbox().0 .1
    }

    /// Get the location of a node.
    pub fn coord(&self, node: usize) -> (f32, f32) {
        return (self.positions[[node, 0]], self.positions[[node, 1]]);
    }

    /// Translate by given vector. If no vector provided, translate such that the result is centered around the origin.
    pub fn translate(self, t: Option<(f32, f32)>) -> Self {
        let (tx, ty) = t.unwrap_or((
            -self.bbox().0 .0 - self.width() / 2.,
            -self.bbox().0 .1 - self.height() / 2.,
        ));
        return Self {
            positions: stack![
                Axis(1),
                &self.positions.slice(s![.., 0]) + tx,
                &self.positions.slice(s![.., 1]) + ty
            ],
        };
    }
}
