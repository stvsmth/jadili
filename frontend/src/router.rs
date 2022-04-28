use crate::{
    app::{self, PageId},
    event_edit_page,
};
use shared::{BlockId, EventId};
use std::collections::VecDeque;
use zoon::{println, *};

// ------ route_history ------

#[static_ref]
fn route_history() -> &'static Mutable<VecDeque<Route>> {
    Mutable::new(VecDeque::new())
}

fn push_to_route_history(route: Route) {
    let mut history = route_history().lock_mut();
    if history.len() == 2 {
        history.pop_back();
    }
    history.push_front(route);
}

pub fn previous_route() -> Option<Route> {
    route_history().lock_ref().get(1).cloned()
}

// ------ router ------

#[static_ref]
pub fn router() -> &'static Router<Route> {
    Router::new(|route: Option<Route>| {
        println!("{}", routing::url());

        let route = match route {
            Some(route) => {
                push_to_route_history(route.clone());
                route
            }
            None => {
                return app::set_page_id(PageId::Unknown);
            }
        };

        match route {
            Route::EventRoot => {
                println!("Event Root route");
                if not(app::is_user_logged()) {
                    return router().replace(Route::Login);
                }
                app::set_page_id(PageId::EventList);
            }
            Route::Event { event_id } => {
                println!("Event route");
                if not(app::is_user_logged()) {
                    return router().replace(Route::Login);
                }
                app::set_page_id(PageId::Event);
                event_edit_page::set_event_id(event_id);
            }
            Route::BlockEdit { event_id, block_id } => {
                println!("Block edit route");
                // FIXME: Un-comment this when we're done testing
                // if not(app::is_user_logged()) {
                //     return router().replace(Route::Login);
                // }
                app::set_page_id(PageId::BlockEdit);
                println!("Routing to block_edit/{}/{}", event_id, block_id);
            }
            Route::Login => {
                println!("Login route");
                if app::is_user_logged() {
                    return router().replace(Route::Root);
                }
                app::set_page_id(PageId::Login);
            }
            Route::Root => {
                print!("Root route");
                app::set_page_id(PageId::Home);
            }
        }
    })
}

// ------ Route ------

#[route]
#[derive(Clone)]
pub enum Route {
    #[route("event")]
    EventRoot,

    #[route("event", event_id)]
    Event { event_id: EventId },

    #[route("block_edit", event_id, block_id)]
    BlockEdit {
        event_id: EventId,
        block_id: BlockId,
    },

    #[route("login")]
    Login,

    #[route()]
    Root,
}
