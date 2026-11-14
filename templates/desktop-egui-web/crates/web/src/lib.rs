#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    wasm_bindgen_futures::spawn_local(async {
        use wasm_bindgen::JsCast;

        let document = web_sys::window()
            .expect("window")
            .document()
            .expect("document");
        let canvas = document
            .get_element_by_id("egui_canvas")
            .expect("canvas")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("HtmlCanvasElement");
        eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|cc| Ok(Box::new(ui_app::TemplateApp::new(cc)))),
            )
            .await
            .expect("web runner");
    });
}
