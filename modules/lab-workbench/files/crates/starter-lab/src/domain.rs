use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starter_core::{Application, Definition, Error, Kind};
use starter_documents::{Format, storage::Document};
use starter_evaluation::Graph;
use starter_science::{length::meter, quantities::Length, volume::cubic_meter};
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Tank {
    pub width_m: f64,
    pub height_m: f64,
    pub depth_m: f64,
    pub fill: f64,
}

impl Default for Tank {
    fn default() -> Self {
        Self {
            width_m: 2.0,
            height_m: 1.0,
            depth_m: 1.0,
            fill: 0.5,
        }
    }
}

impl Tank {
    fn validate(&self) -> Result<(), starter_documents::Error> {
        if [self.width_m, self.height_m, self.depth_m]
            .iter()
            .all(|side| (0.01..=10.0).contains(side))
            && (0.0..=1.0).contains(&self.fill)
        {
            Ok(())
        } else {
            Err(starter_documents::Error::Invalid(
                "Use dimensions between 0.01 and 10 m and a fill fraction between 0 and 1".into(),
            ))
        }
    }

    pub fn litres(self) -> f64 {
        let [width, height, depth] =
            [self.width_m, self.height_m, self.depth_m].map(Length::new::<meter>);
        (width * height * depth).get::<cubic_meter>() * self.fill * 1000.0
    }
}

pub const FORMAT: Format<Tank> = Format {
    id: "tank-laboratory",
    version: 1,
    max_bytes: 65536,
    migrate: |_, _| Err(starter_documents::Error::Format),
    validate: Tank::validate,
};

#[derive(Clone)]
pub struct Lab {
    pub document: Document<Tank>,
    pub result: Option<f64>,
}

#[derive(Deserialize, JsonSchema)]
struct Empty {}

#[derive(Deserialize, JsonSchema)]
struct FilePath {
    path: String,
}

#[derive(Deserialize, JsonSchema)]
struct Measurement {
    litres: f64,
}

fn operation(
    id: &'static str,
    description: &'static str,
    permission: &'static str,
    public: bool,
) -> Definition<Lab> {
    Definition {
        id,
        description,
        permission,
        public,
        available: |_, _| true,
    }
}

pub fn application() -> Application<Lab> {
    let mut app = Application::new(
        Lab {
            document: Document::new(FORMAT, Tank::default(), 64).expect("valid initial tank"),
            result: None,
        },
        64,
    );
    app.query(
        operation(
            "tank.parameters",
            "Read the tank dimensions and fill",
            "read",
            true,
        ),
        |lab, _: Empty, _| Ok(*lab.document.history().value()),
    )
    .unwrap();
    app.query(
        operation(
            "tank.litres",
            "Calculate liquid volume in litres",
            "read",
            true,
        ),
        |lab, _: Empty, _| Ok(lab.document.history().value().litres()),
    )
    .unwrap();
    app.command(
        operation(
            "tank.resize",
            "Commit tank dimensions and fill",
            "write",
            true,
        ),
        Kind::Document,
        |lab, tank: Tank, _| {
            lab.document
                .replace(tank)
                .map_err(|e| Error::new("parameters", e))?;
            lab.result = None;
            Ok(tank)
        },
    )
    .unwrap();
    app.command(
        operation("tank.undo", "Undo the latest tank edit", "write", true),
        Kind::Document,
        |lab, _: Empty, _| {
            lab.document.undo().map_err(|e| Error::new("history", e))?;
            lab.result = None;
            Ok(*lab.document.history().value())
        },
    )
    .unwrap();
    app.command(
        operation(
            "tank.publish",
            "Commit a result from the current revision",
            "write",
            false,
        ),
        Kind::Session,
        |lab, value: Measurement, _| {
            if !value.litres.is_finite() {
                return Err(Error::new("measurement", "The result must be finite"));
            }
            lab.result = Some(value.litres);
            Ok(value.litres)
        },
    )
    .unwrap();
    app.command(
        operation(
            "tank.save",
            "Save the current document to a local path",
            "write",
            false,
        ),
        Kind::Session,
        |lab, file: FilePath, _| {
            lab.document
                .save(Path::new(&file.path))
                .map_err(|e| Error::new("save", e))?;
            Ok(file.path)
        },
    )
    .unwrap();
    app.command(
        operation("tank.open", "Open a verified tank document", "write", false),
        Kind::Session,
        |lab, file: FilePath, _| {
            if lab.document.history().dirty() {
                return Err(Error::new(
                    "dirty",
                    "Save the current document before opening another",
                ));
            }
            lab.document = Document::open(FORMAT, Path::new(&file.path), 64)
                .map_err(|e| Error::new("open", e))?;
            lab.result = None;
            Ok(*lab.document.history().value())
        },
    )
    .unwrap();
    app
}

pub fn evaluate(tank: Tank, cancelled: impl Fn() -> bool) -> Result<f64, String> {
    let mut graph = Graph::default();
    let capacity = graph.add(false);
    let fill = graph.add(true);
    graph.depend(capacity, fill).map_err(|e| format!("{e:?}"))?;
    graph
        .evaluate(cancelled, |is_fill, inputs| {
            if *is_fill {
                Ok::<_, String>(inputs[0].1 * tank.fill)
            } else {
                Ok(Tank { fill: 1.0, ..tank }.litres())
            }
        })
        .map_err(|e| format!("{e:?}"))?;
    graph
        .get(fill)
        .copied()
        .ok_or_else(|| "Missing fill result".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_liquid_volume_and_cancellation() {
        let tank = Tank {
            fill: 0.25,
            ..Tank::default()
        };
        assert_eq!(evaluate(tank, || false), Ok(500.0));
        assert!(evaluate(tank, || true).is_err());
    }
}
