#![forbid(unsafe_code)]
use std::io::{Read, Write};
#[derive(Clone, Default)]
pub struct Data {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}
impl Data {
    pub fn parse(reader: impl Read, max: usize, rows: usize) -> Result<Self, String> {
        let mut input = Vec::new();
        reader
            .take((max as u64).saturating_add(1))
            .read_to_end(&mut input)
            .map_err(|e| e.to_string())?;
        if input.len() > max {
            return Err("CSV exceeds byte limit".into());
        }
        let mut reader = csv::Reader::from_reader(&input[..]);
        let header = reader.headers().map_err(|e| e.to_string())?;
        if header.is_empty() || header.len() > 64 {
            return Err("CSV requires 1–64 columns".into());
        }
        let headers = header.iter().map(str::to_owned).collect();
        let mut result = Vec::new();
        for record in reader.records() {
            if result.len() >= rows {
                return Err("CSV exceeds row limit".into());
            }
            result.push(
                record
                    .map_err(|e| e.to_string())?
                    .iter()
                    .map(str::to_owned)
                    .collect(),
            );
        }
        Ok(Self {
            headers,
            rows: result,
        })
    }
    pub fn filter(&self, needle: &str, sort: Option<usize>) -> Vec<usize> {
        let needle = needle.to_lowercase();
        let mut result: Vec<_> = self
            .rows
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                needle.is_empty() || r.iter().any(|cell| cell.to_lowercase().contains(&needle))
            })
            .map(|(i, _)| i)
            .collect();
        if let Some(col) = sort {
            result.sort_by(|a, b| self.rows[*a].get(col).cmp(&self.rows[*b].get(col)));
        }
        result
    }
    pub fn values<'a>(
        &'a self,
        indices: &'a [usize],
        column: usize,
    ) -> impl Iterator<Item = (usize, f64)> + 'a {
        indices.iter().filter_map(move |&i| {
            let value = self.rows.get(i)?.get(column)?.parse::<f64>().ok()?;
            value.is_finite().then_some((i, value))
        })
    }
    pub fn export(&self, out: impl Write, indices: &[usize]) -> Result<(), String> {
        let mut writer = csv::Writer::from_writer(out);
        writer
            .write_record(&self.headers)
            .map_err(|e| e.to_string())?;
        for &id in indices {
            writer
                .write_record(self.rows.get(id).ok_or("unknown row")?)
                .map_err(|e| e.to_string())?;
        }
        writer.flush().map_err(|e| e.to_string())
    }
}
