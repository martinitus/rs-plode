use crate::layout::scatter::ScatterLayout;
use crate::layout::{BoundingBox, Point};
use crate::{Graph, Layout};
use svg::node::element::path::Data;
use svg::node::element::{Animate, AnimateTransform, Circle, Group, Line, Path, Text};
use svg::{Document, Node};

pub trait RenderSVG {
    type Canvas;

    /// Render self onto canvas returning Ok in case of success or a string indicating the failure.
    fn render(self, canvas: Self::Canvas) -> Result<Self::Canvas, String>;
}

impl<'a, G: Graph> RenderSVG for ScatterLayout<'a, G> {
    type Canvas = Document;

    fn render(self, mut document: Document) -> Result<Self::Canvas, String> {
        document = document
            .set("viewBox", view_box(&self.bbox(), 10))
            .set("preserveAspectRatio", "xMidYMid meet");
        for (u, v) in self.graph().edges() {
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

        for n in 0..self.graph().nodes() {
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

impl<'a, T, G: Graph + 'a> RenderSVG for T
where
    T: Iterator<Item = ScatterLayout<'a, G>>,
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

        let layouts: Vec<ScatterLayout<'a, G>> = self.collect();

        if layouts.len() == 0 {
            return Err("Need at least one step".to_string());
        }

        // translate/transform all layouts to match the last layouts bounding box.
        let bbox = *layouts.last().unwrap().bbox();
        let layouts: Vec<ScatterLayout<_>> =
            layouts.into_iter().map(|l| l.transform(&bbox)).collect();

        document = document
            .set("viewBox", view_box(&bbox, 10))
            .set("preserveAspectRatio", "xMidYMid meet");

        for (u, v) in layouts[0].graph().edges() {
            let mut line = edge_line(layouts[0].coord(u), layouts[0].coord(v));

            let ux: String = layouts
                .iter()
                .map(|s| s.coord(u).x().to_string())
                .collect::<Vec<String>>()
                .join(";");
            let uy: String = layouts
                .iter()
                .map(|s| s.coord(u).y().to_string())
                .collect::<Vec<String>>()
                .join(";");
            let vx: String = layouts
                .iter()
                .map(|s| s.coord(v).x().to_string())
                .collect::<Vec<String>>()
                .join(";");
            let vy: String = layouts
                .iter()
                .map(|s| s.coord(v).y().to_string())
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

        for n in 0..layouts[0].graph().nodes() {
            let mut master = node_group(n, Point(0., 0.));

            if layouts.len() > 1 {
                let trajectory: String = layouts
                    .iter()
                    .map(|s| format!("{} {}", s.coord(n).x(), s.coord(n).y()))
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
