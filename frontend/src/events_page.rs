use crate::router::Route;
use zoon::{named_color::*, *};

// ------ ------
//     View
// ------ ------

pub fn page() -> impl Element {
    Column::new()
        .s(Spacing::new(20))
        .item(link("Gettysburg Address", Route::Event { event_id: 1 }))
    // .item(link("TBD", Route::Event{event_id: 2}))
}

// TODO! duplicated in header page, move somewhere more useful (app?)
fn link(label: &str, route: Route) -> impl Element {
    Link::new()
        .s(Font::new().color(BLUE_4).line(FontLine::new().underline()))
        .label(label)
        .to(route)
}
