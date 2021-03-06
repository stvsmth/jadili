use crate::{
    block_edit_page, event_edit_page, events_page,
    header::header,
    login_page,
    router::{previous_route, router, Route},
};
use shared::{BlockId, EventId, Word};
use zoon::*;

// ------ ------
//     Types
// ------ ------

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum PageId {
    Event,
    EventList,
    BlockEdit {
        event_id: EventId,
        block_id: BlockId,
    },
    Home,
    Login,
    Unknown,
}

// Cover several fields with interior mutability (and/or pub(crate))
#[derive(Debug)]
pub struct RenderBlock {
    pub id: BlockId,
    pub speaker: String,
    pub raw_words: MutableVec<Word>,
    pub full_text: Mutable<String>,
    pub is_visible: Mutable<bool>,
}

// ------ ------
//    States
// ------ ------

#[static_ref]
pub fn logged_user() -> &'static Mutable<Option<String>> {
    Mutable::new(None)
}

#[static_ref]
fn page_id() -> &'static Mutable<PageId> {
    Mutable::new(PageId::Unknown)
}

// ------ ------
//    Helpers
// ------ ------

pub fn is_user_logged() -> bool {
    logged_user().map(Option::is_some)
}

// ------ ------
//   Commands
// ------ ------

pub fn set_page_id(new_page_id: PageId) {
    page_id().set_neq(new_page_id);
}

pub fn log_in(name: String) {
    logged_user().set(Some(name));
    router().go(previous_route().unwrap_or(Route::Root));
}

pub fn log_out() {
    logged_user().take();
    router().go(Route::Root);
}

// ------ ------
//     View
// ------ ------

pub fn root() -> impl Element {
    Column::new()
        .s(Padding::all(20))
        .s(Spacing::new(20))
        .item(header())
        .item(page())
}

fn page() -> impl Element {
    El::new().child_signal(page_id().signal().map(|page_id| match page_id {
        PageId::BlockEdit { event_id, block_id } => block_edit_page::page(event_id, block_id).into_raw_element(),
        PageId::Event => event_edit_page::page().into_raw_element(),
        PageId::EventList => events_page::page().into_raw_element(),
        PageId::Home => El::new().child("Welcome Home!").into_raw_element(),
        PageId::Login => login_page::page().into_raw_element(),
        PageId::Unknown => El::new().child("404").into_raw_element(),
    }))
}
