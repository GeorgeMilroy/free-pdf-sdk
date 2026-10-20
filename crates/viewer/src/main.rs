//! Free PDF SDK viewer: pdfcraft's `PdfCraftApp` in a browser canvas, driven by the host page
//! over the postMessage RPC protocol in `fps_core::protocol`. One viewer per iframe.

#[cfg(target_arch = "wasm32")]
mod branding;
#[cfg(target_arch = "wasm32")]
mod bridge;
#[cfg(target_arch = "wasm32")]
mod host;

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast;
    use pdfcraft_ui_egui::PdfCraftApp;

    let bridge = match bridge::Bridge::new() {
        Ok(b) => b,
        Err(e) => {
            web_sys::console::error_1(&format!("Free PDF SDK: {e}").into());
            return;
        }
    };
    let storage_key = branding::install();
    let options = eframe::WebOptions { renderer: eframe::Renderer::Glow, ..Default::default() };
    wasm_bindgen_futures::spawn_local(async move {
        let Some(canvas) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("fps-canvas"))
            .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
        else {
            bridge::report_start_failure("the viewer page has no canvas");
            return;
        };
        let started = eframe::WebRunner::new()
            .start(
                canvas,
                options,
                Box::new(move |cc| {
                    let mut app = PdfCraftApp::new();
                    if let Some(json) = cc.storage.and_then(|s| s.get_string(&storage_key)) {
                        app.restore(&json);
                    }
                    let inbox: bridge::Inbox = std::rc::Rc::default();
                    bridge.listen(std::rc::Rc::clone(&inbox), cc.egui_ctx.clone());
                    let viewer = host::Viewer::new(app, bridge.clone(), inbox);
                    bridge.hello();
                    Ok(Box::new(viewer))
                }),
            )
            .await;
        if let Err(e) = started {
            web_sys::console::error_1(&e);
            bridge::report_start_failure(&e.as_string().unwrap_or_else(|| format!("the viewer could not start: {e:?}")));
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("fps-viewer {} targets wasm32: build it with scripts/build.ps1", fps_core::SDK_VERSION);
}
