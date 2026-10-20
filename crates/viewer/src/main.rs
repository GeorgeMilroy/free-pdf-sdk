//! Free PDF SDK viewer: pdfcraft's `PdfCraftApp` in a browser canvas.

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast;
    use pdfcraft_ui_egui::PdfCraftApp;

    let options = eframe::WebOptions { renderer: eframe::Renderer::Glow, ..Default::default() };
    wasm_bindgen_futures::spawn_local(async move {
        let Some(canvas) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("fps-canvas"))
            .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
        else {
            return;
        };
        let started = eframe::WebRunner::new().start(canvas, options, Box::new(|_cc| Ok(Box::new(PdfCraftApp::new())))).await;
        if let Err(e) = started {
            web_sys::console::error_1(&e);
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("fps-viewer {} targets wasm32: build it with scripts/build.ps1", fps_core::SDK_VERSION);
}
