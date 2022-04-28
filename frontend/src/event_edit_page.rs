use crate::app::RenderBlock;
use crate::router::{router, Route};
use shared::{BlockId, BlockMessage, EventChoiceMessage, EventId, Word};
use shared::{DownMsg, UpMsg};
use std::ops::Not;
use std::sync::Arc;
use zoon::futures_signals::signal_vec::MutableVecLockRef;
use zoon::{
    eprintln, println, static_ref, Connection, Mutable, MutableVec, RawHtmlEl, Signal, Task, *,
};

// ------ ------
// Reference reading around Mutable and signals
// https://docs.rs/futures-signals/0.3.24/futures_signals/tutorial/index.html

// ------ ------
//    States
// ------ ------

#[static_ref]
fn selected_block() -> &'static Mutable<Option<BlockId>> {
    Mutable::new(None)
}

#[static_ref]
pub fn blocks() -> &'static MutableVec<Arc<RenderBlock>> {
    MutableVec::new()
}

#[static_ref]
fn event_id() -> &'static Mutable<Option<EventId>> {
    Mutable::new(None)
}

#[static_ref]
pub fn connection() -> &'static Connection<UpMsg, DownMsg> {
    Connection::new(|down_msg, cor_id| match down_msg {
        DownMsg::EventSelected(msg) => {
            println!("Chose event {:?}, cor_id: {}", msg.id, cor_id);
        }
        DownMsg::BlockCreated(msg) => {
            let mut blocks = blocks().lock_mut();
            match blocks.iter().find(|block| block.id == msg.id) {
                Some(block) => {
                    println!("... block {} already exists", block.id);
                }
                None => {
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
                        is_visible: Mutable::new(true),
                    };
                    blocks.push_cloned(Arc::new(block));
                    load_audio();
                }
            }
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
                            let mut prev_idx = idx - 1;
                            // walk back blocks until we find the first visible block above us
                            while !*blocks[prev_idx].is_visible.lock_ref() {
                                if prev_idx != 0 {
                                    prev_idx -= 1;
                                }
                            }
                            blocks[idx].is_visible.set(false);
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
                            }
                        }
                    }
                }
                None => println!("No current block {} found, cannot merge above", msg.id),
            };
        }

        DownMsg::BlockDeleted(msg) => do_block_delete(msg.id),
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
pub fn set_event_id(id: EventId) {
    event_id().set(Some(id));
}

pub fn edit_block(id: BlockId) {
    Task::start(async move {
        let blocks = blocks().lock_ref();
        match blocks.iter().find(|block| block.id == id) {
            None => println!("... no block #{} to edit", id),
            Some(block) => {
                router().go(Route::BlockEdit {
                    event_id: event_id().get().unwrap(), // FIXME: ... hmm, should this really be optional
                    block_id: block.id,
                });
            }
        }
    });
}

pub fn choose_event(event_id: Option<EventId>) {
    if let Some(id) = event_id {
        println!("choose_event: inside let");
        Task::start(async move {
            println!("choose_event: task started");
            let result = connection()
                .send_up_msg(UpMsg::ChooseEvent(EventChoiceMessage { id }))
                .await;
            if let Err(error) = result {
                eprintln!("Failed to send choose event message: {:?}", error);
            }
            println!("choose_event sent");
        });
    }
}

fn select_block(id: BlockId) {
    // TODO: This assigns the `current` class to the selected block, but we're not styling on that class yet
    selected_block().set(Some(id));
}

fn remove_block(id: BlockId) {
    println!("Remove block {}", id);
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

fn play_block(id: BlockId) {
    let blocks = blocks().lock_ref();
    let found = blocks.iter().find(|b| b.id == id);

    match found {
        None => println!(" No block #{} found to play???", id),
        Some(block) => {
            let mut start_time =
                (block.raw_words.lock_ref().first().unwrap().start as f32 / 1000.0) - 1.0;
            if start_time < 0.0 {
                start_time = 0.0
            }
            let end_time = block.raw_words.lock_ref().last().unwrap().start as f32 / 1000.0;
            let duration = end_time - start_time + 1.0;
            println!("Play block starting at {} for {}", start_time, duration);
            play_from(start_time, duration);
        }
    }
}

fn merge_above(id: BlockId) {
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

pub fn page() -> impl Element {
    RawHtmlEl::new("div")
        .attr("class", "container")
        .child(jumbotron())
        .child(table())
}

fn jumbotron() -> impl Element {
    let event_name = match event_id().get() {
        Some(id) => format!("event_{:04}", id),
        None => "test_event".to_string(), // TODO! What should we do if event id isn't set???
    };

    RawHtmlEl::new("div").attr("class", "jumbotron").child(
        RawHtmlEl::new("div").attr("class", "row").children([
            RawHtmlEl::new("div")
                .attr("class", "col-md-6")
                .child(RawHtmlEl::new("h1").child("Jadili")),
            RawHtmlEl::new("div")
                .attr("class", "col-md-6")
                .child(action_buttons()),
            RawHtmlEl::new("div").child(
                RawHtmlEl::new("audio")
                    .attr("id", "audio-player")
                    .attr("class", "player col-md-5")
                    .attr("controls", "")
                    .attr("async", "")
                    .attr(
                        "src",
                        // FIXME: Set this to backblaze/jadili/events/<id>/__event_audio.wav
                        format!(
                            "http://localhost:8080/_api/public/assets/{}/__event_audio.wav",
                            event_name
                        )
                        .as_str(),
                    ),
            ),
        ]),
    )
}

fn action_buttons() -> impl Element {
    RawHtmlEl::new("div")
        .attr("class", "row")
        .children([action_button("select-event", "Select Event", || {
            choose_event(event_id().get())
        })])
}

fn action_button(id: &'static str, title: &'static str, on_click: fn()) -> impl Element {
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

fn table() -> impl Element {
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

fn block(block: Arc<RenderBlock>) -> impl Element {
    let id = block.id;
    RawHtmlEl::new("tr")
        .attr_signal(
            "class",
            selected_block()
                .signal_ref(move |selected_id| ((*selected_id)? == id).then(|| "current")),
        )
        .attr_signal(
            "class",
            block
                .is_visible
                .signal_ref(move |is_visible| (!*is_visible).then(|| "hide")),
        )
        .attr("class", block.speaker.as_str())
        .child(block_id(id))
        .child(block_speaker(id, block.speaker.clone()))
        .child(block_text(block))
        .child(block_edit_button(id))
        .child(block_merge_above(id))
        .child(block_remove_button(id))
        .child(block_play_button(id))
}

fn block_id(id: BlockId) -> impl Element {
    RawHtmlEl::new("td").attr("class", "col-md-1").child(id)
}

fn block_speaker(id: BlockId, speaker: String) -> impl Element {
    RawHtmlEl::new("td").attr("class", "col-md-1").child(
        RawHtmlEl::new("a")
            .event_handler(move |_: events::Click| select_block(id))
            .child(speaker),
    )
}

fn block_text(block: Arc<RenderBlock>) -> impl Element {
    let id = block.id;
    RawHtmlEl::new("td")
        .event_handler(move |_: events::Click| select_block(id))
        .child(original_text_as_p(&block, "col-md-8"))
}

pub fn original_text_as_p(block: &Arc<RenderBlock>, width_class: &str) -> impl Element {
    let words = &block.raw_words;
    RawHtmlEl::new("p")
        .attr("class", width_class)
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
        }))
}

fn block_edit_button(id: BlockId) -> impl Element {
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
fn block_merge_above(id: BlockId) -> impl Element {
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

fn block_remove_button(id: BlockId) -> impl Element {
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

fn block_play_button(id: BlockId) -> impl Element {
    RawHtmlEl::new("td").attr("class", "col-1").child(
        RawHtmlEl::new("a")
            .event_handler(move |_: events::Click| play_block(id))
            .child(
                RawHtmlEl::new("span")
                    .attr("class", "glyphicon glyphicon-play play")
                    .attr("aria-hidden", "true")
                    .attr("data-toggle", "tooltip")
                    .attr("data-placement", "bottom")
                    .attr("title", "Play audio for this block"),
            ),
    )
}

// ------ ------
//     Utils
// ------ ------

// TODO: This should be a method on the RenderBlock struct, but I have some things to figure out
pub fn build_full_text(raw_words: MutableVecLockRef<Word>) -> String {
    // Use the raw word structs to build up the space-delimited full text for validation by humans
    raw_words
        .iter()
        .map(|w| w.text.clone())
        .collect::<Vec<String>>()
        .join(" ")
}

#[wasm_bindgen(module = "/js/audio-player.js")]
extern "C" {
    #[wasm_bindgen(js_name = loadAudio)]
    fn load_audio();

    #[wasm_bindgen(js_name = playFrom)]
    fn play_from(position: f32, duration: f32);
}

fn do_block_delete(msg_id: BlockId) {
    // Utility function called by Delete (and, formerly,  MergeAbove before we moved to hiding;
    // isolated here because calling remove_block from MergeAbove will trigger cascading delete messages
    println!("... looking for block {} to delete", msg_id);
    let mut blocks = blocks().lock_mut();
    // println!("... blocks are mut"); // FIXME: <== why isn't this line reached ? we must be invoking lock_mut in some bad way.
    // as it works in a raw delete, then it must be something we're doing int he merge codes
    if let Some(index) = blocks.iter().position(|block| block.id == msg_id) {
        println!("Found block {}, deleting", msg_id);
        blocks.remove(index);
    } else {
        print!("No block found for {}", msg_id);
    }
}
