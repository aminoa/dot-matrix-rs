// Browser file picker. rfd's wasm backend draws its own overlay dialog on
// top of the native browser picker, so we drive a bare <input type="file">
// ourselves instead — the only UI the user sees is the browser's.

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

/// Opens the browser's file picker and calls `on_pick(name, bytes)` once a
/// file is chosen. Must be called in response to a user gesture (click),
/// or the browser will refuse to show the dialog.
pub fn pick_rom_file(on_pick: impl FnOnce(String, Vec<u8>) + 'static) {
    let document = web_sys::window()
        .expect("no window")
        .document()
        .expect("no document");
    let input: web_sys::HtmlInputElement = document
        .create_element("input")
        .expect("failed to create input")
        .dyn_into()
        .expect("input is not an HtmlInputElement");
    input.set_type("file");
    input.set_accept(".gb,.gbc,.bin,.rom");

    let input_for_change = input.clone();
    let on_change = Closure::once_into_js(move |_event: web_sys::Event| {
        let Some(file) = input_for_change.files().and_then(|files| files.get(0)) else {
            return;
        };
        let name = file.name();
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(buffer) = wasm_bindgen_futures::JsFuture::from(file.array_buffer()).await {
                let bytes = js_sys::Uint8Array::new(&buffer).to_vec();
                on_pick(name, bytes);
            }
        });
    });
    input.set_onchange(Some(on_change.unchecked_ref()));

    // the input never enters the DOM; a detached input's click() still
    // opens the picker in all current browsers
    input.click();
}
