use egui::{Context, Event, KeyboardShortcut, Ui};
use std::collections::BTreeSet;

/// Presentation and shortcut for an operation registered by the application.
#[derive(Clone, Copy, Debug)]
pub struct ActionSpec {
    pub command: &'static str,
    pub label: &'static str,
    pub menu: Option<&'static str>,
    pub toolbar: bool,
    pub shortcut: Option<KeyboardShortcut>,
}

/// A specification and its current availability, evaluated by the application host.
#[derive(Clone, Copy, Debug)]
pub struct ActionState {
    pub spec: ActionSpec,
    pub enabled: bool,
}

pub(crate) fn menu_bar(ui: &mut Ui, actions: &[ActionState], selected: &mut Option<usize>) {
    if !actions.iter().any(|action| action.spec.menu.is_some()) {
        return;
    }
    egui::Panel::top("starter-action-menus").show(ui, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            let mut seen = BTreeSet::new();
            for menu in actions.iter().filter_map(|action| action.spec.menu) {
                if !seen.insert(menu) {
                    continue;
                }
                ui.menu_button(menu, |ui| {
                    for (index, action) in actions.iter().enumerate() {
                        if action.spec.menu == Some(menu) && button(ui, action) {
                            *selected = Some(index);
                            ui.close();
                        }
                    }
                });
            }
        });
    });
}

pub(crate) fn toolbar(ui: &mut Ui, actions: &[ActionState], selected: &mut Option<usize>) {
    for (index, action) in actions.iter().enumerate() {
        if action.spec.toolbar && button(ui, action) {
            *selected = Some(index);
        }
    }
}

fn button(ui: &mut Ui, action: &ActionState) -> bool {
    let mut button = egui::Button::new(action.spec.label);
    if let Some(shortcut) = &action.spec.shortcut {
        button = button.shortcut_text(ui.ctx().format_shortcut(shortcut));
    }
    ui.add_enabled(action.enabled, button).clicked()
}

pub(crate) fn shortcut(ctx: &Context, actions: &[ActionState]) -> Option<usize> {
    ctx.input_mut(|input| {
        actions.iter().enumerate().find_map(|(index, action)| {
            let chord = action.spec.shortcut.filter(|_| action.enabled)?;
            let press = input.events.iter().position(|event| {
                matches!(
                    event,
                    Event::Key {
                        key,
                        pressed: true,
                        repeat: false,
                        modifiers,
                        ..
                    } if *key == chord.logical_key && modifiers.matches_exact(chord.modifiers)
                )
            })?;
            input.events.remove(press);
            Some(index)
        })
    })
}
