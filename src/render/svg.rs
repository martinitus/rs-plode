use crate::layout::scatter::{ScatterLayout, ScatterLayoutSequence};
use crate::layout::{BoundingBox, Point};
use crate::{Graph};
use svg::node::element::path::Data;
use svg::node::element::{Animate, AnimateTransform, Circle, Group, Line, Path, Text};
use svg::{Document, Node};

pub trait RenderSVG {
    type Canvas;

    /// Render self onto canvas returning Ok in case of success or a string indicating the failure.
    fn render(self, canvas: Self::Canvas) -> Result<Self::Canvas, String>;
}

impl<G: Graph> RenderSVG for ScatterLayout<G> {
    type Canvas = Document;

    fn render(self, mut document: Document) -> Result<Self::Canvas, String> {
        document = document
            .set("viewBox", view_box(&self.bbox(), 10))
            .set("preserveAspectRatio", "xMidYMid meet");
        for (u, v) in self.graph.edges() {
            let data = Data::new()
                .move_to((self.coord(u).x(), self.coord(u).y()))
                .line_to((self.coord(v).x(), self.coord(v).y()))
                .close();
            let path = Path::new()
                .set("fill", "none")
                .set("stroke", "black")
                .set("stroke-width", 1)
                .set("d", data);

            document.append(path);
        }

        for n in 0..self.graph.nodes() {
            let group = Group::new()
                .set(
                    "transform",
                    format!("translate({}, {})", self.coord(n).0, self.coord(n).1),
                )
                .add(
                    Circle::new()
                        .set("r", 30)
                        .set("stroke", "black")
                        .set("stroke-width", 1)
                        .set("fill", "white"),
                )
                .add(
                    Text::new()
                        .set("text-anchor", "middle")
                        .set("alignment-baseline", "central")
                        .add(svg::node::Text::new(format!("node {}", n))),
                );

            document.append(group);
        }
        Ok(document)
    }
}

impl<G: Graph> RenderSVG for ScatterLayoutSequence<G>
{
    type Canvas = Document;

    fn render(self, mut document: Document) -> Result<Self::Canvas, String> {
        fn node_group(n: usize, pos: Point) -> Group {
            Group::new()
                .set("transform", format!("translate({}, {})", pos.x(), pos.y()))
                .add(
                    Circle::new()
                        .set("r", "1cm")
                        .set("stroke", "black")
                        .set("stroke-width", 1)
                        .set("fill", "white"),
                )
                .add(
                    Text::new()
                        .set("text-anchor", "middle")
                        .set("alignment-baseline", "central")
                        .add(svg::node::Text::new(format!("node {}", n))),
                )
        }

        fn edge_line(_u: Point, _v: Point) -> Line {
            Line::new()
                .set("fill", "none")
                .set("stroke", "black")
                .set("stroke-width", 1)
        }

        // translate/transform all layouts to match the last layouts bounding box.
        let bbox = self.bbox();
        // let layouts: Vec<ScatterLayout<_>> =
        //     layouts.into_iter().map(|l| l.transform(&bbox)).collect();

        document = document
            .set("viewBox", view_box(&bbox, 10))
            .set("preserveAspectRatio", "xMidYMid meet");

        for (u, v) in self.graph.edges() {
            let mut line = edge_line(self.coord(0, u), self.coord(0, v));

            let ux: String = (0..self.frames())
                .map(|s| self.coord(s, u).x().to_string())
                .collect::<Vec<String>>()
                .join(";");
            let uy: String = (0..self.frames())
                .map(|s| self.coord(s, u).y().to_string())
                .collect::<Vec<String>>()
                .join(";");
            let vx: String = (0..self.frames())
                .map(|s| self.coord(s, v).x().to_string())
                .collect::<Vec<String>>()
                .join(";");
            let vy: String = (0..self.frames())
                .map(|s| self.coord(s, v).y().to_string())
                .collect::<Vec<String>>()
                .join(";");
            line.append(
                Animate::new()
                    .set("attributeType", "XML")
                    .set("fill", "freeze")
                    .set("dur", "10s")
                    //                        .set("repeatCount", "indefinite")
                    .set("attributeName", "x1")
                    .set("values", ux),
            );
            line.append(
                Animate::new()
                    .set("attributeType", "XML")
                    .set("fill", "freeze")
                    .set("dur", "10s")
                    //                        .set("repeatCount", "indefinite")
                    .set("attributeName", "y1")
                    .set("values", uy),
            );
            line.append(
                Animate::new()
                    .set("attributeType", "XML")
                    .set("fill", "freeze")
                    .set("dur", "10s")
                    //                        .set("repeatCount", "indefinite")
                    .set("attributeName", "x2")
                    .set("values", vx),
            );
            line.append(
                Animate::new()
                    .set("attributeType", "XML")
                    .set("fill", "freeze")
                    .set("dur", "10s")
                    //                        .set("repeatCount", "indefinite")
                    .set("attributeName", "y2")
                    .set("values", vy),
            );
            document.append(line);
        }

        for n in 0..self.graph.nodes() {
            let mut master = node_group(n, Point(0., 0.));

            if self.frames() > 1 {
                let trajectory: String = (0..self.frames())
                    .map(|s| format!("{} {}", self.coord(s, n).x(), self.coord(s, n).y()))
                    .collect::<Vec<String>>()
                    .join(";");
                master.append(
                    AnimateTransform::new()
                        .set("attributeName", "transform")
                        .set("type", "translate")
                        .set("dur", "10s")
                        .set("fill", "freeze")
                        //                            .set("repeatCount", "indefinite")
                        .set("values", trajectory),
                );
            }

            document.append(master);
        }

        Ok(document)
    }
}

/// Define a viewBox tuple from giving bounding box and padding percentage.
fn view_box(bbox: &BoundingBox, padding: usize) -> (f32, f32, f32, f32) {
    let frac = padding as f32 / 100.;

    let height = f32::max(bbox.height() * (1. + 2. * frac), 400.);
    let width = f32::max(bbox.width() * (1. + 2. * frac), 400.);

    let shiftx = f32::max(0., height - bbox.height() * (1. + frac)) / 2.;
    let shifty = f32::max(0., width - bbox.width() * (1. + frac)) / 2.;

    (
        bbox.lower_left().x() - shiftx,
        bbox.lower_left().y() - shifty,
        width,
        height,
    )
}
