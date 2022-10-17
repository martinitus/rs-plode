use crate::layout::scatter::ScatterLayout;
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
        document = document.set("viewBox", (-500, -500, 1800, 1800));
        for (u, v) in self.graph().edges() {
            let data = Data::new()
                .move_to(self.coord(u))
                .line_to(self.coord(v))
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
        fn node_group(n: usize, pos: (f32, f32)) -> Group {
            Group::new()
                .set("transform", format!("translate({}, {})", pos.0, pos.1))
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

        fn edge_line(_u: (f32, f32), _v: (f32, f32)) -> Line {
            Line::new()
                .set("fill", "none")
                .set("stroke", "black")
                .set("stroke-width", 1)
        }

        let layouts: Vec<ScatterLayout<'a, G>> = self.collect();

        if layouts.len() == 0 {
            return Err("Need at least one step".to_string());
        }

        document = document.set("viewBox", (-700, -700, 1400, 1400));
        for (u, v) in layouts[0].graph().edges() {
            let mut line = edge_line(layouts[0].coord(u), layouts[0].coord(v));

            let ux: String = layouts
                .iter()
                .map(|s| s.coord(u).0.to_string())
                .collect::<Vec<String>>()
                .join(";");
            let uy: String = layouts
                .iter()
                .map(|s| s.coord(u).1.to_string())
                .collect::<Vec<String>>()
                .join(";");
            let vx: String = layouts
                .iter()
                .map(|s| s.coord(v).0.to_string())
                .collect::<Vec<String>>()
                .join(";");
            let vy: String = layouts
                .iter()
                .map(|s| s.coord(v).1.to_string())
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
            let mut master = node_group(n, (0., 0.));

            if layouts.len() > 1 {
                let trajectory: String = layouts
                    .iter()
                    .map(|s| format!("{} {}", s.coord(n).0, s.coord(n).1))
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
