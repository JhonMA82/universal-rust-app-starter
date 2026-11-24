#![forbid(unsafe_code)]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starter_core::{Application, Definition, Error, Kind};
use starter_documents::History;
use starter_evaluation::{Graph, NodeIndex};
use starter_science::{Tolerance, length::meter, quantities::Length, volume::cubic_meter};

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Dimensions {
    pub width_m: f64,
    pub height_m: f64,
    pub depth_m: f64,
}
impl Default for Dimensions {
    fn default() -> Self {
        Self {
            width_m: 2.0,
            height_m: 1.0,
            depth_m: 2.0,
        }
    }
}
impl Dimensions {
    fn sides(self) -> [f64; 3] {
        [self.width_m, self.height_m, self.depth_m]
    }
    fn validate(&self) -> Result<(), starter_documents::Error> {
        if self.sides().into_iter().all(|v| (0.01..=10.0).contains(&v)) {
            Ok(())
        } else {
            Err(starter_documents::Error::Invalid(
                "side outside 0.01–10 m".into(),
            ))
        }
    }
    pub fn volume(self) -> f64 {
        let [w, h, d] = self.sides().map(Length::new::<meter>);
        (w * h * d).get::<cubic_meter>()
    }
}
pub type Domain = History<Dimensions>;
#[derive(Deserialize, JsonSchema)]
struct Empty {}
fn definition(
    id: &'static str,
    description: &'static str,
    permission: &'static str,
) -> Definition<Domain> {
    Definition {
        id,
        description,
        permission,
        public: true,
        available: |_, _| true,
    }
}
pub fn application() -> Application<Domain> {
    let mut a = Application::new(History::new(Dimensions::default(), 64), 64);
    a.query(
        definition("box.dimensions", "Read dimensions in metres", "read"),
        |h, _: Empty, _| Ok(*h.value()),
    )
    .unwrap();
    a.query(
        definition("box.volume", "Compute volume in cubic metres", "read"),
        |h, _: Empty, _| Ok(h.value().volume()),
    )
    .unwrap();
    a.command(
        definition("box.resize", "Commit three dimensions in metres", "write"),
        Kind::Document,
        |h, d: Dimensions, _| {
            h.replace(d, Dimensions::validate)
                .map_err(|e| Error::new("dimension", e))?;
            Ok(*h.value())
        },
    )
    .unwrap();
    for (id, description, redo) in [
        ("box.undo", "Undo a resize", false),
        ("box.redo", "Redo a resize", true),
    ] {
        a.command(
            definition(id, description, "write"),
            Kind::Document,
            move |h, _: Empty, _| {
                if redo { h.redo() } else { h.undo() }.map_err(|e| Error::new("history", e))?;
                Ok(*h.value())
            },
        )
        .unwrap();
    }
    a
}
enum Compute {
    Side(usize),
    Volume,
}
pub struct Derived {
    graph: Graph<Compute, f64>,
    sides: [NodeIndex; 3],
    volume: NodeIndex,
    last: Option<[f64; 3]>,
}
impl Default for Derived {
    fn default() -> Self {
        let mut graph = Graph::default();
        let sides = [0, 1, 2].map(|i| graph.add(Compute::Side(i)));
        let volume = graph.add(Compute::Volume);
        for id in sides {
            graph.depend(id, volume).unwrap();
        }
        Self {
            graph,
            sides,
            volume,
            last: None,
        }
    }
}
impl Derived {
    pub fn update(&mut self, d: Dimensions) -> (usize, f64, bool) {
        let sides = d.sides();
        for (i, id) in self.sides.iter().enumerate() {
            if self.last.is_none_or(|earlier| earlier[i] != sides[i]) {
                self.graph.invalidate(*id).unwrap();
            }
        }
        self.last = Some(sides);
        let changed = self
            .graph
            .evaluate(
                || false,
                |c, deps| {
                    Ok::<_, ()>(match c {
                        Compute::Side(i) => sides[*i],
                        Compute::Volume => deps.iter().map(|(_, v)| **v).product(),
                    })
                },
            )
            .unwrap();
        let v = *self.graph.get(self.volume).unwrap();
        (
            changed,
            v,
            Tolerance::new(1e-12, 1e-12).unwrap().agrees(v, d.volume()),
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn one_changed_dimension_recomputes_two_nodes() {
        let mut graph = Derived::default();
        assert_eq!(graph.update(Dimensions::default()), (4, 4.0, true));
        assert_eq!(
            graph.update(Dimensions {
                height_m: 3.0,
                ..Default::default()
            }),
            (2, 12.0, true)
        );
        assert_eq!(
            graph.update(Dimensions {
                height_m: 3.0,
                ..Default::default()
            }),
            (0, 12.0, true)
        );
    }
}
