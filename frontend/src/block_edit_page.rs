use zoon::{named_color::*, *};

// ------ ------
//    States
// ------ ------

// ------ ------
//   Commands
// ------ ------

// ------ ------
//     View
// ------ ------

pub fn page() -> impl Element {
    Column::new()
        .s(Spacing::new(15))
        .item(corrected_text())
        .item(original_text())
        .item(back_button())
}

fn corrected_text() -> impl Element {
    TextArea::new()
        .s(Padding::all(7))
        .label_hidden("Corrected Text")
        .text("Foo bar baz")
    // .on_change(set_name)
}

// TODO: Use same rendering as event page (hover displays data etc)
fn original_text() -> impl Element {
    RawHtmlEl::new("div")
        .attr("disabled", "disabled")
        .child(RawHtmlEl::new("p"))
        .child("Foo Bar Baz. Original")
}

fn back_button() -> impl Element {
    let (hovered, hovered_signal) = Mutable::new_and_signal(false);
    Button::new()
        .s(Background::new().color_signal(hovered_signal.map_bool(|| BLUE_2, || BLUE_4)))
        .s(Padding::new().x(7).y(4))
        .s(Font::new().color(hsluv!(0, 0, 100)))
        .s(RoundedCorners::all(5))
        .on_hovered_change(move |is_hovered| hovered.set(is_hovered))
        .label("Back to event")
        .on_press(routing::back)
}
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
