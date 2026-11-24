#![forbid(unsafe_code)]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starter_core::{Application, Definition, Error, Kind};
use starter_dataset::Data;
use std::sync::Arc;

pub const MAX_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_ROWS: usize = 25_000;
#[derive(Clone)]
pub struct Domain {
    pub table: Arc<Data>,
    pub view: Arc<View>,
}
#[derive(Clone)]
pub struct View {
    pub indices: Vec<usize>,
    pub column: usize,
    pub filter: String,
    pub sort: Option<usize>,
}
#[derive(Deserialize, JsonSchema)]
struct Empty {}
#[derive(Deserialize, JsonSchema)]
struct Import {
    csv: String,
}
#[derive(Deserialize, JsonSchema)]
struct Filter {
    query: String,
    column: usize,
    sort: Option<usize>,
}
#[derive(Serialize, JsonSchema)]
pub struct Summary {
    pub rows: usize,
    pub numeric: usize,
    pub mean: Option<f64>,
}
pub fn metrics(data: &Data, view: &View) -> Summary {
    let (mut mean, mut n) = (0.0, 0);
    for (_, value) in data.values(&view.indices, view.column) {
        n += 1;
        let weight = 1.0 / n as f64;
        mean = mean * (1.0 - weight) + value * weight;
    }
    Summary {
        rows: view.indices.len(),
        numeric: n,
        mean: (n != 0 && mean.is_finite()).then_some(mean),
    }
}
fn initial_view(data: &Data) -> Arc<View> {
    Arc::new(View {
        indices: (0..data.rows.len()).collect(),
        column: data.headers.len() - 1,
        filter: String::new(),
        sort: None,
    })
}
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
fn fail(e: impl ToString) -> Error {
    Error::new("data", e)
}
pub fn application() -> Application<Domain> {
    let sample = Arc::new(
        Data::parse(
            &b"trial,voltage\n1,1.2\n2,1.8\n3,2.3\n4,2.7\n5,3.1\n"[..],
            MAX_BYTES,
            MAX_ROWS,
        )
        .unwrap(),
    );
    let mut app = Application::new(
        Domain {
            view: initial_view(&sample),
            table: sample,
        },
        32,
    );
    app.command(
        definition("data.import", "Import bounded CSV text", "write"),
        Kind::Command,
        |state, input: Import, _| {
            let data = Data::parse(input.csv.as_bytes(), MAX_BYTES, MAX_ROWS).map_err(fail)?;
            state.view = initial_view(&data);
            state.table = Arc::new(data);
            Ok(state.table.rows.len())
        },
    )
    .unwrap();
    app.command(
        definition("data.filter", "Filter and sort rows by column", "write"),
        Kind::Presentation,
        |state, input: Filter, _| {
            if input.query.len() > 256
                || input.column >= state.table.headers.len()
                || input.sort.is_some_and(|s| s >= state.table.headers.len())
            {
                return Err(fail("invalid filter or column"));
            }
            let indices = state.table.filter(&input.query, input.sort);
            let count = indices.len();
            state.view = Arc::new(View {
                indices,
                column: input.column,
                filter: input.query,
                sort: input.sort,
            });
            Ok(count)
        },
    )
    .unwrap();
    app.query(
        definition("data.summary", "Summarize finite numeric values", "read"),
        |state, _: Empty, _| Ok(metrics(&state.table, &state.view)),
    )
    .unwrap();
    app.query(
        definition("data.csv", "Export filtered rows as CSV text", "read"),
        |state, _: Empty, _| {
            let mut csv = Vec::new();
            state
                .table
                .export(&mut csv, &state.view.indices)
                .map_err(fail)?;
            String::from_utf8(csv).map_err(fail)
        },
    )
    .unwrap();
    app
}
