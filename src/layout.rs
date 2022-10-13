use ndarray::{s, Array, Array2, Ix2};
use ndarray_stats::QuantileExt;

/// A layout where nodes can have a real valued position in 2D space.
pub struct ScatterLayout {
    positions: Array2<f32>,
    bbox: ((f32, f32), (f32, f32)),
}

impl ScatterLayout {
    pub fn new(positions: Array2<f32>) -> Self {
        let bbox = (
            (
                *positions.slice(s![.., 0]).min().unwrap(),
                *positions.slice(s![.., 1]).min().unwrap(),
            ),
            (
                *positions.slice(s![.., 0]).max().unwrap(),
                *positions.slice(s![.., 1]).max().unwrap(),
            ),
        );
        Self { positions, bbox }
    }

    /// The bounding box that encompasses all nodes.
    /// Returns lower left and upper right corner.
    pub fn bbox(&self) -> ((f32, f32), (f32, f32)) {
        return self.bbox;
    }
}
