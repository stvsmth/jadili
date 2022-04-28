use zoon::*;

mod app;
mod block_edit_page;
mod event_edit_page;
mod events_page;
mod header;
mod login_page;
mod router;

// ------ ------
//     Start
// ------ ------

#[wasm_bindgen(start)]
pub fn start() {
    router::router();
    start_app("main", app::root);
    event_edit_page::connection();
}
