#[cfg(feature = "svg")]
pub mod svg {
    use crate::{Graph, Observer};
    use ndarray::Array2;
    use svg::node::element::path::Data;
    use svg::node::element::{
        Animate, AnimateMotion, AnimateTransform, Circle, Group, Line, Path, Text,
    };
    use svg::{Document, Node};

    /// Render given graph with nodes defined by the layout arrays.
    pub fn render(graph: &impl Graph, layout: &Array2<f32>) -> Document {
        let mut document = Document::new()
            .set("viewBox", (-500, -500, 1800, 1800))
            .set("width", "800px")
            .set("height", "800px")
            .set("preserveAspectRatio", "none");
        println!("layout: {:3.2?}", layout);
        _render(&mut document, graph, layout);
        document
    }

    // helper rendering in a predefined document
    fn _render(document: &mut Document, graph: &impl Graph, layout: &Array2<f32>) {
        for (u, v) in graph.edges() {
            let data = Data::new()
                .move_to((layout[[u, 0]], layout[[u, 1]]))
                .line_to((layout[[v, 0]], layout[[v, 1]]))
                .close();
            let path = Path::new()
                .set("fill", "none")
                .set("stroke", "black")
                .set("stroke-width", 1)
                .set("d", data);

            document.append(path);
        }

        for n in 0..graph.node_count() {
            let group = Group::new()
                .set(
                    "transform",
                    format!("translate({}, {})", layout[[n, 0]], layout[[n, 1]]),
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
    }

    pub struct AnimationObserver<'a, G: Graph> {
        step: usize,
        layouts: Vec<Array2<f32>>,
        graph: &'a G,
    }

    impl<'a, G: Graph> AnimationObserver<'a, G> {
        pub fn new(graph: &'a G) -> Self {
            Self {
                step: 0,
                layouts: Vec::new(),
                graph,
            }
        }

        fn node_group(&self, n: usize, pos: (f32, f32)) -> Group {
            Group::new()
                .set("transform", format!("translate({}, {})", pos.0, pos.1))
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
                )
        }

        fn edge_path(&self, u: (f32, f32), v: (f32, f32)) -> Path {
            Path::new()
                .set("fill", "none")
                .set("stroke", "black")
                .set("stroke-width", 1)
        }

        fn edge_line(&self, u: (f32, f32), v: (f32, f32)) -> Line {
            Line::new()
                .set("fill", "none")
                .set("stroke", "black")
                .set("stroke-width", 1)
        }

        fn coordinates(&self, node: usize, step: usize) -> (f32, f32) {
            (
                self.layouts[step][[node, 0]] as f32,
                self.layouts[step][[node, 1]] as f32,
            )
        }
    }

    impl<'a, G: Graph> Observer<G, Array2<f32>> for AnimationObserver<'a, G> {
        fn observe(&mut self, graph: &G, layout: &Array2<f32>) {
            self.step += 1;
            self.layouts.push(layout.clone());
        }
    }

    impl<'a, G: Graph> Into<Document> for AnimationObserver<'a, G> {
        fn into(self) -> Document {
            let mut document = Document::new()
                .set("viewBox", (-50, -50, 250, 250))
                .set("width", "1000px")
                .set("height", "1000px");
//                .set("preserveAspectRatio", "none");

            if self.step == 0 {
                panic!("Need at least one step");
            }

            for (u, v) in self.graph.edges() {
                let mut line = self.edge_line(self.coordinates(u, 0), self.coordinates(v, 0));

                let ux: String = (0..self.step)
                    .map(|s| self.coordinates(u, s).0.to_string())
                    .collect::<Vec<String>>()
                    .join(";");
                let uy: String = (0..self.step)
                    .map(|s| self.coordinates(u, s).1.to_string())
                    .collect::<Vec<String>>()
                    .join(";");
                let vx: String = (0..self.step)
                    .map(|s| self.coordinates(v, s).0.to_string())
                    .collect::<Vec<String>>()
                    .join(";");
                let vy: String = (0..self.step)
                    .map(|s| self.coordinates(v, s).1.to_string())
                    .collect::<Vec<String>>()
                    .join(";");
                line.append(
                    Animate::new()
                        .set("attributeType", "XML")
                        .set("fill", "freeze")
                        .set("dur", "10s")
                        .set("repeatCount", "indefinite")
                        .set("attributeName", "x1")
                        .set("values", ux),
                );
                line.append(
                    Animate::new()
                        .set("attributeType", "XML")
                        .set("fill", "freeze")
                        .set("dur", "10s")
                        .set("repeatCount", "indefinite")
                        .set("attributeName", "y1")
                        .set("values", uy),
                );
                line.append(
                    Animate::new()
                        .set("attributeType", "XML")
                        .set("fill", "freeze")
                        .set("dur", "10s")
                        .set("repeatCount", "indefinite")
                        .set("attributeName", "x2")
                        .set("values", vx),
                );
                line.append(
                    Animate::new()
                        .set("attributeType", "XML")
                        .set("fill", "freeze")
                        .set("dur", "10s")
                        .set("repeatCount", "indefinite")
                        .set("attributeName", "y2")
                        .set("values", vy),
                );
                document.append(line);
            }

            for n in 0..self.graph.node_count() {
                let mut master = self.node_group(n, (0., 0.));

                if self.step > 1 {
                    let trajectory: String = (0..self.step)
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
                            .set("repeatCount", "indefinite")
                            .set("values", trajectory),
                    );
                }

                document.append(master);
            }

            document
        }
    }
}
