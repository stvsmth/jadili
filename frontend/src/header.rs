use crate::{app, router::Route};
use zoon::{named_color::*, *};

// ------ ------
//     View
// ------ ------

pub fn header() -> impl Element {
    Row::new()
        .s(Spacing::new(20))
        .item(link("Home", Route::Root))
        .item(link("Events", Route::EventRoot))
        .item_signal(app::logged_user().signal_ref(|name| {
            if let Some(name) = name {
                log_out_button(name).left_either()
            } else {
                link("Log in", Route::Login).right_either()
            }
        }))
}

fn link(label: &str, route: Route) -> impl Element {
    Link::new()
        .s(Font::new().color(BLUE_4).line(FontLine::new().underline()))
        .label(label)
        .to(route)
}

fn log_out_button(name: &str) -> impl Element {
    let (hovered, hovered_signal) = Mutable::new_and_signal(false);
    Button::new()
        .s(Background::new().color_signal(hovered_signal.map_bool(|| BLUE_2, || BLUE_4)))
        .s(Padding::new().x(7).y(4))
        .s(Font::new().color(hsluv!(0, 0, 100)))
        .s(RoundedCorners::all(5))
        .on_hovered_change(move |is_hovered| hovered.set(is_hovered))
        .label(format!("Log out {}", name))
        .on_press(app::log_out)
}
