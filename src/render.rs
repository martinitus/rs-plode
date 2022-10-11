#[cfg(feature = "svg")]
pub mod svg {
    use ndarray::{Array, Array2};
    use svg::{Document};
    use svg::node::element::path::Data;
    use svg::node::element::{Circle, Group, Path, Text};
    use svg::node::element::tag::Group;
    use crate::Graph;

    /// Render given graph with nodes defined by the layout arrays.
    pub fn render(graph: &impl Graph, layout: &Array2<f32>) -> Document {
        let mut document = Document::new()
            .set("viewBox", (-500, -500, 1800, 1800))
            .set("width", "800px")
            .set("height", "800px")
            .set("preserveAspectRatio", "none");
        println!("layout: {:3.2?}", layout);
        _render(document, graph, layout)
    }

    // helper rendering in a predefined document
    fn _render(mut document: Document, graph: &impl Graph, layout: &Array2<f32>) -> Document {
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


            document = document.add(path);
        }

        for n in 0..graph.node_count() {
            let group = Group::new()
                .set("transform", format!("translate({}, {})", layout[[n, 0]], layout[[n, 1]]))
                .add(
                    Circle::new()
                        .set("r", 30)
                        .set("stroke", "black")
                        .set("stroke-width", 1)
                        .set("fill", "white")
                ).add(
                Text::new()
                    .set("text-anchor", "middle")
                    .set("alignment-baseline", "central")
                    .add(svg::node::Text::new(format!("node {}", n)))
            );

            document = document.add(group);
        }

        document
    }

    /// Return an observer that includes intermediate layout steps as an svg animation.
    pub fn animation(graph: &impl Graph) -> impl FnMut(&Array2<f32>) {
        let mut document = Document::new()
            .set("viewBox", (-500, -500, 1800, 1800))
            .set("width", "800px")
            .set("height", "800px")
            .set("preserveAspectRatio", "none");

        let mut c = 0;
        // document = _render(document);

        struct S;

        impl FnOnce<Args> for S {
            type Output = ();

            extern "rust-call" fn call_once(self, args: Args) -> Self::Output {
                todo!()
            }
        }
        impl FnMut(&Array2<f32>) for S {
            extern "rust-call" fn call_mut(&mut self, args: Args) -> Self::Output {
                todo!()
            }
        };

        let observe = |layout: &Array2<f32>| {
            c = c + 1;
            document = _render(document, graph, layout);
        };
        return observe;
    }
}