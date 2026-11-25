use eframe::egui::Ui;
use starter_tasks::{Control, Error, Job, Pool};

/// Example of submitting an immutable input snapshot from an egui view.
pub struct TaskPanel {
    pool: Pool,
    job: Option<Job<usize, ()>>,
    completed: Option<(u64, String, bool)>,
}

impl Default for TaskPanel {
    fn default() -> Self {
        Self {
            pool: Pool::new(1),
            job: None,
            completed: None,
        }
    }
}

impl TaskPanel {
    /// Render the example. Update `revision` whenever the input changes.
    pub fn show(&mut self, ui: &mut Ui, text: &str, revision: u64) {
        ui.heading("Background task");
        if let Some(job) = &mut self.job {
            if let Some(result) = job.poll(revision) {
                let (message, error) = match result {
                    Ok(words) => (format!("{words} words"), false),
                    Err(Error::Cancelled) => ("Task cancelled".to_owned(), false),
                    Err(Error::Stale) => ("Input changed; count discarded".to_owned(), false),
                    Err(Error::WorkerStopped | Error::Failed(())) => {
                        ("The word count could not finish".to_owned(), true)
                    }
                };
                self.completed = Some((revision, message, error));
                self.job = None;
            } else {
                starter_desktop::progress(ui, f32::from(job.progress()) / 10_000.0);
                if ui.button("Cancel word count").clicked() {
                    job.cancel();
                }
                ui.ctx()
                    .request_repaint_after(std::time::Duration::from_millis(50));
                return;
            }
        }
        if ui.button("Count words in background").clicked() {
            self.completed = None;
            let snapshot = text.to_owned();
            match self
                .pool
                .start(revision, move |control| Ok(count_words(&snapshot, control)))
            {
                Ok(job) => {
                    self.job = Some(job);
                    ui.ctx().request_repaint();
                }
                Err(error) => self.completed = Some((revision, error.to_string(), true)),
            }
        }
        if let Some((origin, message, error)) = &self.completed
            && *origin == revision
        {
            starter_desktop::status(ui, message, *error);
        }
    }
}

fn count_words(text: &str, control: &Control) -> usize {
    let mut words = 0;
    let mut in_word = false;
    let mut checkpoint = 0;
    for (offset, character) in text.char_indices() {
        if offset >= checkpoint {
            if control.cancelled() {
                return words;
            }
            let basis_points = (offset as u128 * 10_000 / text.len() as u128) as u16;
            control.report(basis_points);
            checkpoint = offset.saturating_add(4096);
        }
        let word_character = !character.is_whitespace();
        if word_character && !in_word {
            words += 1;
        }
        in_word = word_character;
    }
    control.report(10_000);
    words
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{Harness, kittest::Queryable};
    use std::{
        cell::Cell,
        time::{Duration, Instant},
    };

    #[test]
    fn task_panel_shows_result_for_current_input() {
        let mut panel = TaskPanel::default();
        let revision = Cell::new(1);
        let mut harness =
            Harness::new_ui(|ui| panel.show(ui, " café\u{2003}con\tpan ", revision.get()));
        harness.get_by_label("Count words in background").click();
        let deadline = Instant::now() + Duration::from_secs(5);
        while harness.query_by_label("3 words").is_none() {
            assert!(Instant::now() < deadline, "word count did not finish");
            harness.step();
            std::thread::yield_now();
        }
        revision.set(2);
        harness.step();
        assert!(harness.query_by_label("3 words").is_none());
    }
}
