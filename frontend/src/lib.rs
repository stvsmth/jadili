use fake::faker::company::en::*;
use fake::Fake;
use shared::{BlockMessage, EventChoiceMessage, Word};
use shared::{DownMsg, UpMsg};
use std::ops::Not;
use std::sync::Arc;
use zoon::futures_signals::signal_vec::MutableVecLockRef;
use zoon::{
    eprintln, println, static_ref, Connection, Mutable, MutableVec, RawHtmlEl, Signal, Task, *,
};

// ------ ------
// Reference reading around Mutable and signals
// https://docs.rs/futures-signals/0.3.22/futures_signals/tutorial/index.html

// ------ ------
//    States
// ------ ------

#[static_ref]
fn selected_block() -> &'static Mutable<Option<Id>> {
    Mutable::new(None)
}

// TODO: Research exactly why we're using Arc
#[static_ref]
fn blocks() -> &'static MutableVec<Arc<RenderBlock>> {
    MutableVec::new()
}

type Id = usize;

#[derive(Debug)]
struct RenderBlock {
    id: usize,
    speaker: String,
    raw_words: MutableVec<Word>,
    full_text: Mutable<String>,
}

#[static_ref]
pub fn connection() -> &'static Connection<UpMsg, DownMsg> {
    Connection::new(|down_msg, cor_id| match down_msg {
        DownMsg::EventSelected(msg) => {
            println!("Chose event {:?}, cor_id: {}", msg.id, cor_id);
        }
        DownMsg::BlockCreated(msg) => {
            println!("Create block {}", msg.id);
            let raw_words = MutableVec::new();
            for word in msg.words {
                raw_words.lock_mut().push_cloned(word)
            }
            let full_text = build_full_text(raw_words.lock_ref());
            let block = RenderBlock {
                id: msg.id,
                speaker: msg.speaker,
                raw_words,
                full_text: Mutable::new(full_text),
            };
            blocks().lock_mut().push_cloned(Arc::new(block));
        }
        DownMsg::BlockEdited(msg) => {
            println!("Edit block {}", msg.id);
            let blocks = blocks().lock_ref();
            match blocks.iter().find(|block| block.id == msg.id) {
                Some(block) => {
                    for word in msg.words {
                        block.raw_words.lock_mut().push_cloned(word)
                    }
                }
                None => println!("No block {:?} found to update", msg.id),
            }
        }
        DownMsg::BlockMergedWithAbove(msg) => {
            println!("Merge block {} with the block above", msg.id);

            // Find our position in the blocks (if we exist)
            let pos = blocks()
                .lock_ref()
                .iter()
                .position(|block| block.id == msg.id);

            // Given the position of the block we want to merge above, extract its text, append above, then delete
            // ... only merge blocks if the speakers in both blocks are the same
            match pos {
                Some(0) => println!("Cannot merge first element, there's nothing above us"),
                Some(idx) => {
                    let blocks = blocks().lock_ref();
                    match blocks.iter().find(|block| block.id == msg.id) {
                        None => println!(" ... no block #{} found to merge above", msg.id),
                        Some(_) => {
                            let prev_idx = idx - 1;
                            if blocks[prev_idx].speaker != blocks[idx].speaker {
                                eprintln!("Cannot merge different speakers");
                            } else {
                                for word in msg.words {
                                    blocks[prev_idx].raw_words.lock_mut().push_cloned(word);
                                }
                                let full_text =
                                    build_full_text(blocks[prev_idx].raw_words.lock_ref());
                                blocks[prev_idx]
                                    .full_text
                                    .lock_mut()
                                    .replace_range(.., full_text.as_str());
                                println!(
                                    "Full text: {}",
                                    blocks[prev_idx].full_text.lock_ref().to_string()
                                );
                                remove_block(msg.id);
                            }
                        }
                    }
                }
                None => println!("No current block {} found, cannot merge above", msg.id),
            };
        }

        DownMsg::BlockDeleted(msg) => {
            println!("... looking for block {} to delete", msg.id);
            let pos = blocks()
                .lock_ref()
                .iter()
                .position(|block| block.id == msg.id);
            match pos {
                Some(index) => {
                    println!("Found block {}, deleting", msg.id);
                    blocks().lock_mut().remove(index);
                }
                None => print!("No block found for {}", msg.id),
            }
        }
    })
}
// ------ ------
//    Signals
// ------ ------

fn blocks_exist() -> impl Signal<Item = bool> {
    blocks().signal_vec_cloned().is_empty().map(Not::not)
}

// ------ ------
//   Commands
// ------ ------

fn edit_block(id: Id) {
    Task::start(async move {
        let blocks = blocks().lock_ref();
        match blocks.iter().find(|block| block.id == id) {
            None => println!("... no block #{} to edit", id),
            Some(block) => {
                let new_text: Vec<Word> = vec![
                    Word {
                        confidence: 99.0,
                        start: 0,
                        end: 0,
                        speaker: Some(block.speaker.clone()),
                        text: Buzzword().fake(),
                    },
                    Word {
                        confidence: 99.0,
                        start: 0,
                        end: 0,
                        speaker: Some(block.speaker.clone()),
                        text: BuzzwordMiddle().fake(),
                    },
                    Word {
                        confidence: 99.0,
                        start: 0,
                        end: 0,
                        speaker: Some(block.speaker.clone()),
                        text: format!("{}.", BuzzwordTail().fake::<String>()),
                    },
                ];
                let result = connection()
                    .send_up_msg(UpMsg::EditBlock(BlockMessage {
                        id,
                        speaker: block.speaker.to_string(),
                        words: new_text,
                    }))
                    .await;
                if let Err(error) = result {
                    println!("Failed to send edit message: {:?}", error);
                }
            }
        }
    });
}

fn choose_event(event_id: usize) {
    Task::start(async move {
        let result = connection()
            .send_up_msg(UpMsg::ChooseEvent(EventChoiceMessage { id: event_id }))
            .await;
        if let Err(error) = result {
            eprintln!("Failed to send choose event message: {:?}", error);
        }
    });
}

fn select_block(id: Id) {
    // TODO: This assigns the `current` class to the selected block, but we're not styling on that class yet
    selected_block().set(Some(id))
}

fn remove_block(id: Id) {
    Task::start(async move {
        let result = connection()
            .send_up_msg(UpMsg::DeleteBlock(BlockMessage {
                id,
                speaker: "n/a".to_string(), // TODO: Create a BlockIdOnlyMessage (but w/ better name)
                words: vec![],
            }))
            .await;
        if let Err(error) = result {
            eprintln!("Failed to send delete block message: {:?}", error);
        }
    });
}

fn merge_above(id: Id) {
    println!("Merge above {}", id);
    Task::start(async move {
        let blocks = blocks().lock_ref();
        let found = blocks.iter().find(|b| b.id == id);

        match found {
            None => println!("Merge block #{} not found", id),
            Some(block) => {
                let mut words_to_merge: Vec<Word> = Vec::new();
                for word in block.raw_words.lock_ref().iter() {
                    words_to_merge.push(word.clone());
                }
                let result = connection()
                    .send_up_msg(UpMsg::MergeBlockAbove(BlockMessage {
                        id,
                        speaker: block.speaker.to_string(),
                        words: words_to_merge,
                    }))
                    .await;
                if let Err(error) = result {
                    eprintln!("Failed to send merge above block message: {:?}", error);
                }
            }
        }
    });
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
                    .child(RawHtmlEl::new("h1").child("Jadili")),
                RawHtmlEl::new("div")
                    .attr("class", "col-md-6")
                    .child(action_buttons()),
            ])),
    )
}

fn action_buttons() -> RawHtmlEl {
    RawHtmlEl::new("div")
        .attr("class", "row")
        .children([action_button("select-event", "Select Event", || {
            choose_event(1)
        })])
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
        .child_signal(blocks_exist().map(|blocks_exist| {
            blocks_exist.then(|| {
                RawHtmlEl::new("tbody")
                    .attr("id", "tbody")
                    .children_signal_vec(blocks().signal_vec_cloned().map(block))
            })
        }))
}

fn block(block: Arc<RenderBlock>) -> RawHtmlEl {
    let id = block.id;
    RawHtmlEl::new("tr")
        .attr_signal(
            "class",
            selected_block()
                .signal_ref(move |selected_id| ((*selected_id)? == id).then(|| "current")),
        )
        .attr("class", block.speaker.as_str())
        .children(IntoIterator::into_iter([
            block_id(id),
            block_speaker(id, block.speaker.clone()),
            block_text(block),
            block_edit_button(id),
            block_merge_above(id),
            block_remove_button(id),
            RawHtmlEl::new("td").attr("class", "col-md-6"),
        ]))
}

fn block_id(id: Id) -> RawHtmlEl {
    RawHtmlEl::new("td").attr("class", "col-md-1").child(id)
}

fn block_speaker(id: Id, speaker: String) -> RawHtmlEl {
    RawHtmlEl::new("td").attr("class", "col-md-1").child(
        RawHtmlEl::new("a")
            .event_handler(move |_: events::Click| select_block(id))
            .child(speaker),
    )
}

fn block_text(block: Arc<RenderBlock>) -> RawHtmlEl {
    let id = block.id;
    let words = &block.raw_words;
    RawHtmlEl::new("td").attr("class", "col-md-6").child(
        RawHtmlEl::new("p")
            .event_handler(move |_: events::Click| select_block(id))
            .children_signal_vec(words.signal_vec_cloned().map(|word| {
                let conf_class = if word.confidence <= 0.50 {
                    "conf-low"
                } else {
                    ""
                };
                RawHtmlEl::new("span")
                    .attr("class", conf_class)
                    .child(format!("{} ", word.text))
                    .attr("data-toggle", "tooltip")
                    .attr("data-placement", "bottom")
                    .attr(
                        "title",
                        format!("{:02.1}%", word.confidence * 100.0).as_str(),
                    )
            })),
    )
}

fn block_edit_button(id: Id) -> RawHtmlEl {
    RawHtmlEl::new("td").attr("class", "col-1").child(
        RawHtmlEl::new("a")
            .event_handler(move |_: events::Click| edit_block(id))
            .child(
                // TODO: Investigate creating a custom SpanWithTooltip element, there's a lot of boiler plate below
                RawHtmlEl::new("span")
                    .attr("class", "glyphicon glyphicon-edit edit")
                    .attr("aria-hidden", "true")
                    .attr("data-toggle", "tooltip")
                    .attr("data-placement", "bottom")
                    .attr("title", "Edit block contents"),
            ),
    )
}
fn block_merge_above(id: Id) -> RawHtmlEl {
    RawHtmlEl::new("td").attr("class", "col-1").child(
        RawHtmlEl::new("a")
            .event_handler(move |_: events::Click| merge_above(id))
            .child(
                RawHtmlEl::new("span")
                    .attr("class", "glyphicon glyphicon-upload upload")
                    .attr("aria-hidden", "true")
                    .attr("data-toggle", "tooltip")
                    .attr("data-placement", "bottom")
                    .attr("title", "Merge with block above"),
            ),
    )
}

fn block_remove_button(id: Id) -> RawHtmlEl {
    RawHtmlEl::new("td").attr("class", "col-1").child(
        RawHtmlEl::new("a")
            .event_handler(move |_: events::Click| remove_block(id))
            .child(
                RawHtmlEl::new("span")
                    .attr("class", "glyphicon glyphicon-remove remove")
                    .attr("aria-hidden", "true")
                    .attr("data-toggle", "tooltip")
                    .attr("data-placement", "bottom")
                    .attr("title", "Remove this block"),
            ),
    )
}
// ------ ------
//     Utils
// ------ ------

// TODO: This should be a method on the RenderBlock struct, but I have some things to figure out
fn build_full_text(raw_words: MutableVecLockRef<Word>) -> String {
    // Use the raw word structs to build up the space-delimited full text for validation by humans
    raw_words
        .iter()
        .map(|w| w.text.clone())
        .collect::<Vec<String>>()
        .join(" ")
}

// ------ ------
//     Start
// ------ ------

#[wasm_bindgen(start)]
pub fn start() {
    start_app("main", root);
}
