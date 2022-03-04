use lipsum::lipsum_words;
use rand::prelude::*;
use shared::{BlockMessage, EventChoiceMessage};
use shared::{DownMsg, UpMsg};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::{iter::repeat_with, ops::Not};
use zoon::{
    println, eprintln, static_ref, Connection, Mutable, MutableVec, RawHtmlEl, Signal, Task, Text, *,
};

// ------ ------
// TODO
// Read Mutable / future
// https://docs.rs/futures-signals/0.3.22/futures_signals/tutorial/index.html

// ------ ------
//    States
// ------ ------

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

#[static_ref]
fn selected_row() -> &'static Mutable<Option<Id>> {
    Mutable::new(None)
}

#[static_ref]
fn rows() -> &'static MutableVec<Arc<Block>> {
    MutableVec::new()
}

type Id = usize;

struct Block {
    id: Id,
    speaker: Mutable<String>, // TODO: Remove mutable
    text: Mutable<String>,
}


#[static_ref]
pub fn connection() -> &'static Connection<UpMsg, DownMsg> {
    Connection::new(|down_msg, cor_id| {
        println!("DownMsg received: {:?}", down_msg);

        match down_msg {
            DownMsg::EventSelected(msg) => {
               println!("Chose event {:?}", msg.data); 
               println!("cor_Id {}", cor_id); 
            },
            DownMsg::BlockReceived(msg) => {
                let rows = rows().lock_ref();
                let elem = rows.into_iter().filter(|row| row.id == msg.id).take(1).next();
                match elem {
                    Some(row) => {
                        let mut content = row.text.lock_mut();
                        content.replace_range(.., msg.text.as_str());
                    }
                    None => eprintln!("Hmmm, no row with that id, that shouldn't happen."),   
                }
            },
        }
    })
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

fn pull_block_data(id: Id) {
    Task::start(async move {
        let result = connection()
            .send_up_msg(UpMsg::SendBlock(BlockMessage {
                id,
                speaker: "Z".to_string(),
                text: "Todo".to_string(),
            }))
            .await;
        if let Err(error) = result {
            eprintln!("Failed to send poll data message: {:?}", error);
        }
    });
}

fn choose_event(event_id: usize ) {
    Task::start(async move {
        let result = connection()
            .send_up_msg(UpMsg::ChooseEvent(EventChoiceMessage {
                id: event_id
            }))
            .await;
        if let Err(error) = result {
            eprintln!("Failed to choose event message: {:?}", error);
        }
    });
}

fn create_row() -> Arc<Block> {
    let range = rand::thread_rng().gen_range(7..150);
    let speaker = ['A', 'B', 'C', 'D', 'E', 'F', 'G']
        .choose(&mut rand::thread_rng())
        .unwrap()
        .to_string();
    let text = lipsum_words(range);
    Arc::new(Block {
        id: NEXT_ID.fetch_add(1, Ordering::SeqCst),
        speaker: Mutable::new(speaker),
        text: Mutable::new(text),
    })
}

fn fetch_rows(count: usize) {
    rows()
        .lock_mut()
        .extend(repeat_with(create_row).take(count));
}

fn select_row(id: Id) {
    selected_row().set(Some(id))
}

fn remove_row(id: Id) {
    rows().lock_mut().retain(|row| row.id != id);
}

// fn edit_row(id: Id) {
//     let rows = rows().lock_ref();
//     let elem = rows.into_iter().filter(|row| row.id == id).take(1).next();
//     match elem {
//         Some(row) => {
//             let mut content = row.text.lock_mut();
//             let range = rand::thread_rng().gen_range(5..80);
//             content.replace_range(.., lipsum_words(range).as_str());
//         }
//         None => panic!("Hmmm, no row with that id, that shouldn't happen."),
//     }
// }

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
                    .child(RawHtmlEl::new("h1").child("Jadili")),
                RawHtmlEl::new("div")
                    .attr("class", "col-md-6")
                    .child(action_buttons()),
            ])),
    )
}

fn action_buttons() -> RawHtmlEl {
    RawHtmlEl::new("div").attr("class", "row").children([
        action_button("select-event", "Select Event", || choose_event(1)),
        action_button("add", "Fetch 5 rows", || fetch_rows(5)),
    ])
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
        .attr("class", "table test-data")
        .child_signal(rows_exist().map(|rows_exist| {
            rows_exist.then(|| {
                RawHtmlEl::new("tbody")
                    .attr("id", "tbody")
                    .children_signal_vec(rows().signal_vec_cloned().map(row))
            })
        }))
}

fn row(row: Arc<Block>) -> RawHtmlEl {
    let id = row.id;
    let speaker = row.speaker.get_cloned();
    let speaker_id = speaker.as_str();
    RawHtmlEl::new("tr")
        .attr_signal(
            "class",
            selected_row()
                .signal_ref(move |selected_id| ((*selected_id)? == id).then(|| "current")),
        )
        .attr("class", speaker_id)
        .children(IntoIterator::into_iter([
            row_id(id),
            row_speaker(id, row.speaker.signal_cloned()),
            row_text(id, row.text.signal_cloned()),
            row_edit_button(id),
            row_remove_button(id),
            RawHtmlEl::new("td").attr("class", "col-md-6"),
        ]))
}

fn row_id(id: Id) -> RawHtmlEl {
    RawHtmlEl::new("td").attr("class", "col-md-1").child(id)
}

fn row_speaker(id: Id, speaker: impl Signal<Item = String> + Unpin + 'static) -> RawHtmlEl {
    RawHtmlEl::new("td").attr("class", "col-md-1").child(
        RawHtmlEl::new("a")
            .event_handler(move |_: events::Click| select_row(id))
            .child(Text::with_signal(speaker)),
    )
}

fn row_text(_id: Id, text: impl Signal<Item = String> + Unpin + 'static) -> RawHtmlEl {
    RawHtmlEl::new("td").attr("class", "col-md-6").child(
        RawHtmlEl::new("div")
            // .event_handler(move |_: events::Click| select_row(id))
            .child(Text::with_signal(text)),
    )
}

fn row_edit_button(id: Id) -> RawHtmlEl {
    RawHtmlEl::new("td").attr("class", "col-md-1").child(
        RawHtmlEl::new("a")
            // .event_handler(move |_: events::Click| edit_row(id))
            .event_handler(move |_: events::Click| pull_block_data(id))
            .child(
                RawHtmlEl::new("span")
                    .attr("class", "glyphicon glyphicon-edit edit")
                    .attr("aria-hidden", "true"),
            ),
    )
}

fn row_remove_button(id: Id) -> RawHtmlEl {
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
}
