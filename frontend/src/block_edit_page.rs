use crate::event_edit_page::{blocks, connection, original_text_as_p, play_block, player_element};
use shared::{BlockEdited, BlockId, EventId, UpMsg};
use std::cmp::max;
use zoon::{eprintln, named_color::*, println, *};

// ------ ------
//    States
// ------ ------

#[static_ref]
fn content() -> &'static Mutable<String> {
    Mutable::new(String::new())
}

#[static_ref]
fn this_block_id() -> &'static Mutable<Option<BlockId>> {
    Mutable::new(None)
}

// ------ ------
//   Commands
// ------ ------

fn text_change_handler(text: String) {
    content().set(text);
}

fn text_blur_handler() {
    if let Some(block_id) = this_block_id().get() {
        let blocks = blocks().lock_ref();
        let found = blocks.iter().find(|b| b.id == block_id);
        match found {
            Some(block) => {
                let text = content().lock_ref().clone();
                block_edit_message(block_id, block.speaker.clone(), text);
            }
            None => eprintln!("Block {} not found!", block_id),
        }
    }
}

// ------ ------
//     View
// ------ ------

pub fn page(_event_id: EventId, block_id: BlockId) -> impl Element {
    this_block_id().set(Some(block_id));
    Column::new()
        .s(Spacing::new(15))
        .item(player_element())
        .item(corrected_text(block_id))
        .item(original_text(block_id))
        .item(back_button())
}

fn corrected_text(id: BlockId) -> impl Element {
    let mut num_rows: u32 = 6;
    let blocks = blocks().lock_ref();
    let found = blocks.iter().find(|b| b.id == id);
    let text = match found {
        Some(block) => {
            let raw_words = block.raw_words.lock_ref();
            num_rows = max(raw_words.len() * 5 / 75, 6) as u32; // TODO! Finalize math and raise to constant
            block.full_text.lock_ref().clone()
        }
        None => {
            eprintln!("Block {} not found to display!", id);
            "".to_string()
        }
    };

    RawHtmlEl::new("div").attr("class", "col-md-8").child(
        TextArea::new()
            .s(Width::fill())
            .s(Height::new(num_rows * 12)) //
            .s(Padding::all(4))
            .text(text)
            .on_change(text_change_handler)
            .on_blur(text_blur_handler)
            .label_hidden("Corrected text"),
    )
}

fn block_edit_message(block_id: BlockId, speaker: String, text: String) {
    println!("Send block edited message for block {}", block_id);
    Task::start(async move {
        let result = connection()
            .send_up_msg(UpMsg::EditBlock(BlockEdited {
                id: block_id,
                speaker: speaker.to_string(),
                corrected_text: text,
            }))
            .await;
        if let Err(error) = result {
            eprintln!("Failed to send block edit message: {:?}", error);
        }
    });
}

fn original_text(id: BlockId) -> impl Element {
    let blocks = blocks().lock_ref();
    let found = blocks.iter().find(|b| b.id == id);
    match found {
        Some(block) => RawHtmlEl::new("div")
            .child(RawHtmlEl::new("p"))
            .child(
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
            .child(original_text_as_p(block, "col-md-8")),
        None => {
            println!("Block {} not found to display!", id);
            RawHtmlEl::new("div")
                .attr("disabled", "disabled")
                .child(RawHtmlEl::new("p").child("Error"))
        }
    }
}

fn back_button() -> impl Element {
    let (hovered, hovered_signal) = Mutable::new_and_signal(false);
    Button::new()
        .s(Width::new(120))
        .s(Background::new().color_signal(hovered_signal.map_bool(|| BLUE_2, || BLUE_4)))
        .s(Padding::new().x(7).y(4))
        .s(Font::new().color(hsluv!(0, 0, 100)))
        .s(RoundedCorners::all(5))
        .on_hovered_change(move |is_hovered| hovered.set(is_hovered))
        .label("Back to event")
        .on_press(routing::back)
}
