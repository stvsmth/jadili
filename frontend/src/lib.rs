use lipsum::lipsum_words;
use rand::prelude::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::{iter::repeat_with, ops::Not};

use zoon::*;

// ------ ------
// TODO
// Read Mutable / future
// https://docs.rs/futures-signals/0.3.22/futures_signals/tutorial/index.html

// ------ ------
//    States
// ------ ------

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

#[static_ref]
fn selected_row() -> &'static Mutable<Option<ID>> {
    Mutable::new(None)
}

#[static_ref]
fn rows() -> &'static MutableVec<Arc<Row>> {
    MutableVec::new()
}

type ID = usize;

struct Row {
    id: ID,
    speaker: Mutable<String>,
    label: Mutable<String>,
}

// ------ ------
//    Signals
// ------ ------

fn rows_exist() -> impl Signal<Item = bool> {
    rows().signal_vec_cloned().is_empty().map(Not::not)
}

// ------ ------
//   Commands
// ------ ------

fn create_row() -> Arc<Row> {
    let range = rand::thread_rng().gen_range(7..150);
    let speaker = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K']
        .choose(&mut rand::thread_rng())
        .unwrap()
        .to_string();
    let label = lipsum_words(range);
    Arc::new(Row {
        id: NEXT_ID.fetch_add(1, Ordering::SeqCst),
        speaker: Mutable::new(speaker),
        label: Mutable::new(label),
    })
}

fn append_rows(count: usize) {
    rows()
        .lock_mut()
        .extend(repeat_with(create_row).take(count));
}

fn select_row(id: ID) {
    selected_row().set(Some(id))
}

fn remove_row(id: ID) {
    rows().lock_mut().retain(|row| row.id != id);
}

fn edit_row(id: ID) {
    let rows = rows().lock_ref();
    let elem = rows.into_iter().filter(|row| row.id == id).take(1).next();
    match elem {
        Some(row) => {
            let mut content = row.label.lock_mut();
            let range = rand::thread_rng().gen_range(5..80);
            content.replace_range(.., lipsum_words(range).as_str());
        }
        None => panic!("Hmmm, no row with that id, that shouldn't happen."),
    }
}

// ------ ------
//     View
// ------ ------

fn root() -> RawHtmlEl {
    RawHtmlEl::new("div")
        .attr("class", "container")
        .children(IntoIterator::into_iter([
            jumbotron(),
            table(),
            RawHtmlEl::new("span")
                .attr("class", "preloadicon glyphicon glyphicon-remove")
                .attr("aria-hidden", ""),
        ]))
}

fn jumbotron() -> RawHtmlEl {
    RawHtmlEl::new("div").attr("class", "jumbotron").child(
        RawHtmlEl::new("div")
            .attr("class", "row")
            .children(IntoIterator::into_iter([
                RawHtmlEl::new("div")
                    .attr("class", "col-md-6")
                    .child(RawHtmlEl::new("h1").child("Andika")),
                RawHtmlEl::new("div")
                    .attr("class", "col-md-6")
                    .child(action_buttons()),
            ])),
    )
}

fn action_buttons() -> RawHtmlEl {
    RawHtmlEl::new("div")
        .attr("class", "row")
        .children(IntoIterator::into_iter([action_button(
            "add",
            "Append 5 rows",
            || append_rows(5),
        )]))
}

fn action_button(id: &'static str, title: &'static str, on_click: fn()) -> RawHtmlEl {
    RawHtmlEl::new("div")
        .attr("class", "col-sm-6 smallpad")
        .child(
            RawHtmlEl::new("button")
                .attr("id", id)
                .attr("class", "btn btn-primary btn-block")
                .attr("type", "button")
                .event_handler(move |_: events::Click| on_click())
                .child(title),
        )
}

fn table() -> RawHtmlEl {
    RawHtmlEl::new("table")
        .attr("class", "table table-hover table-striped test-data")
        .child_signal(rows_exist().map(|rows_exist| {
            rows_exist.then(|| {
                RawHtmlEl::new("tbody")
                    .attr("id", "tbody")
                    .children_signal_vec(rows().signal_vec_cloned().map(row))
            })
        }))
}

fn row(row: Arc<Row>) -> RawHtmlEl {
    let id = row.id;
    RawHtmlEl::new("tr")
        .attr_signal(
            "class",
            selected_row().signal_ref(move |selected_id| ((*selected_id)? == id).then(|| "danger")),
        )
        .children(IntoIterator::into_iter([
            row_id(id),
            row_speaker(id, row.speaker.signal_cloned()),
            row_text(id, row.label.signal_cloned()),
            row_edit_button(id),
            row_remove_button(id),
            RawHtmlEl::new("td").attr("class", "col-md-6"),
        ]))
}

fn row_id(id: ID) -> RawHtmlEl {
    RawHtmlEl::new("td").attr("class", "col-md-1").child(id)
}

fn row_speaker(id: ID, speaker: impl Signal<Item = String> + Unpin + 'static) -> RawHtmlEl {
    RawHtmlEl::new("td").attr("class", "col-md-1").child(
        RawHtmlEl::new("a")
            .event_handler(move |_: events::Click| select_row(id))
            .child(Text::with_signal(speaker)),
    )
}

fn row_text(id: ID, label: impl Signal<Item = String> + Unpin + 'static) -> RawHtmlEl {
    RawHtmlEl::new("td").attr("class", "col-md-4").child(
        RawHtmlEl::new("div")
            .event_handler(move |_: events::Click| select_row(id))
            .child(Text::with_signal(label)),
    )
}

fn row_edit_button(id: ID) -> RawHtmlEl {
    RawHtmlEl::new("td").attr("class", "col-md-1").child(
        RawHtmlEl::new("a")
            .event_handler(move |_: events::Click| edit_row(id))
            .child(
                RawHtmlEl::new("span")
                    .attr("class", "glyphicon glyphicon-edit edit")
                    .attr("aria-hidden", "true"),
            ),
    )
}

fn row_remove_button(id: ID) -> RawHtmlEl {
    RawHtmlEl::new("td").attr("class", "col-md-1").child(
        RawHtmlEl::new("a")
            .event_handler(move |_: events::Click| remove_row(id))
            .child(
                RawHtmlEl::new("span")
                    .attr("class", "glyphicon glyphicon-remove remove")
                    .attr("aria-hidden", "true"),
            ),
    )
}

// ------ ------
//     Start
// ------ ------

#[wasm_bindgen(start)]
pub fn start() {
    start_app("main", root);
    append_rows(2);
}
