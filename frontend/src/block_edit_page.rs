use shared::{BlockId, EventId};
use std::cmp::max;
use zoon::{named_color::*, println, *};

use crate::event_edit_page::{blocks, build_full_text, original_text_as_p};

// ------ ------
//    States
// ------ ------

// ------ ------
//   Commands
// ------ ------

// ------ ------
//     View
// ------ ------

pub fn page(event_id: EventId, block_id: BlockId) -> impl Element {
    println!("event_id {:?}", event_id);
    Column::new()
        .s(Spacing::new(15))
        .item(RawHtmlEl::new("h2").child("WARNING! Edits are currently discarded!!!"))
        .item(corrected_text(block_id))
        .item(original_text(block_id))
        .item(back_button())
}

fn corrected_text(id: BlockId) -> impl Element {
    let blocks = blocks().lock_ref();
    let found = blocks.iter().find(|b| b.id == id);
    match found {
        Some(block) => {
            let raw_words = block.raw_words.lock_ref();
            let num_rows = max(raw_words.len() * 5 / 75, 6);
            RawHtmlEl::new("div").child(
                RawHtmlEl::new("textarea")
                    .attr("rows", num_rows.to_string().as_str())
                    .attr("class", "col-md-8")
                    .child(build_full_text(raw_words)),
            )
        }
        None => {
            println!(" No block #{} found to display???", id);
            RawHtmlEl::new("div")
                .attr("disabled", "disabled")
                .child(RawHtmlEl::new("p"))
        }
    }

    // .on_change(set_name)

    // FIXME: Taken from the old edit_block command
    // let result = connection()
    //     .send_up_msg(UpMsg::EditBlock(BlockMessage {
    //         id,
    //         speaker: block.speaker.to_string(),
    //         words: new_text,
    //     }))
    //     .await;
    // if let Err(error) = result {
    //     println!("Failed to send edit message: {:?}", error);
    // }
}

fn original_text(id: BlockId) -> impl Element {
    let blocks = blocks().lock_ref();
    let found = blocks.iter().find(|b| b.id == id);
    match found {
        Some(block) => RawHtmlEl::new("div")
            .child(RawHtmlEl::new("p"))
            .child(original_text_as_p(block, "col-md-8")),
        None => {
            println!(" No block #{} found to display???", id);
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
