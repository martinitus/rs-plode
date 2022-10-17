use crate::layout::scatter::ScatterLayout;
use crate::{Graph, Observe};
use svg::node::element::path::Data;
use svg::node::element::{Animate, AnimateTransform, Circle, Group, Line, Path, Text};
use svg::{Document, Node};


/// Render given graph with nodes defined by the layout arrays.
pub fn render(graph: &impl Graph, layout: &ScatterLayout) -> Document {
    let mut document = Document::new()
        .set("viewBox", (-500, -500, 1800, 1800))
        .set("width", "800px")
        .set("height", "800px")
        .set("preserveAspectRatio", "none");
    for (u, v) in graph.edges() {
        let data = Data::new()
            .move_to(layout.coord(u))
            .line_to(layout.coord(v))
            .close();
        let path = Path::new()
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-width", 1)
            .set("d", data);

        document.append(path);
    }

    for n in 0..graph.nodes() {
        let group = Group::new()
            .set(
                "transform",
                format!("translate({}, {})", layout.coord(n).0, layout.coord(n).1),
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
    document
}

pub struct AnimationObserver<'a, G: Graph> {
    layouts: Vec<ScatterLayout>,
    graph: &'a G,
}

impl<'a, G: Graph> AnimationObserver<'a, G> {
    pub fn new(graph: &'a G) -> Self {
        Self {
            layouts: Vec::new(),
            graph,
        }
    }

    fn node_group(&self, n: usize, pos: (f32, f32)) -> Group {
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

    fn edge_line(&self, _u: (f32, f32), _v: (f32, f32)) -> Line {
        Line::new()
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-width", 1)
    }

    fn coordinates(&self, node: usize, step: usize) -> (f32, f32) {
        self.layouts[step].coord(node)
    }
}

impl<'a, G: Graph> Observe<G, ScatterLayout> for AnimationObserver<'a, G> {
    fn observe(&mut self, _graph: &G, layout: &ScatterLayout) {
        self.layouts.push(layout.clone());
    }
}

impl<'a, G: Graph> TryInto<Document> for AnimationObserver<'a, G> {
    type Error = String;

    fn try_into(self) -> Result<Document, Self::Error> {
        if self.layouts.len() == 0 {
            return Err("Need at least one step".to_string());
        }

        let mut document = Document::new()
            .set("viewBox", (-700, -700, 1400, 1400))
            .set("width", "1000px")
            .set("height", "1000px")
            .set("preserveAspectRatio", "none");

        for (u, v) in self.graph.edges() {
            let mut line = self.edge_line(self.coordinates(u, 0), self.coordinates(v, 0));

            let ux: String = (0..self.layouts.len())
                .map(|s| self.coordinates(u, s).0.to_string())
                .collect::<Vec<String>>()
                .join(";");
            let uy: String = (0..self.layouts.len())
                .map(|s| self.coordinates(u, s).1.to_string())
                .collect::<Vec<String>>()
                .join(";");
            let vx: String = (0..self.layouts.len())
                .map(|s| self.coordinates(v, s).0.to_string())
                .collect::<Vec<String>>()
                .join(";");
            let vy: String = (0..self.layouts.len())
                .map(|s| self.coordinates(v, s).1.to_string())
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
            let mut master = self.node_group(n, (0., 0.));

            if self.layouts.len() > 1 {
                let trajectory: String = (0..self.layouts.len())
                    .map(|s| {
                        let coords = self.coordinates(n, s);
                        format!("{} {}", coords.0, coords.1)
                    })
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
