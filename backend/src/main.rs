use fake::faker::lorem::en::*;
use fake::Fake;
use moon::tokio::time::{sleep, Duration};
use moon::*;
use rand::prelude::*;
use shared::{BlockMessage, DownMsg, EventStreamMessage, UpMsg};
use std::sync::atomic::{AtomicUsize, Ordering};

async fn frontend() -> Frontend {
    Frontend::new()
        .title("Jadili")
        .default_styles(false)
        .append_to_head(r#"<link href="/_api/public/css/currentStyle.css" rel="stylesheet"/>"#)
        .body_content(r#"<div id="main"></div>"#)
}

async fn up_msg_handler(req: UpMsgRequest<UpMsg>) {
    println!("request: {:?}", req);
    let UpMsgRequest { up_msg, cor_id, .. } = req;

    match up_msg {
        UpMsg::DeleteBlock(block) => {
            sessions::broadcast_down_msg(&DownMsg::BlockDeleted(block), cor_id).await;
        }
        UpMsg::EditBlock(block) => {
            sessions::broadcast_down_msg(&DownMsg::BlockEdited(block), cor_id).await;
        }
        UpMsg::ChooseEvent(event) => {
            let lorem: Vec<String> = Words(3..5).fake();
            let stream = EventStreamMessage {
                id: event.id,
                data: lorem.join(" "),
            };
            sessions::broadcast_down_msg(&DownMsg::EventSelected(stream), cor_id).await;

            static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

            tokio::spawn(async move {
                loop {
                    // let next_id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
                    let lorem: Vec<String> = Sentences(2..14).fake();
                    let speaker = ['A', 'B', 'C', 'D', 'E', 'F', 'G']
                        .choose(&mut rand::thread_rng())
                        .unwrap()
                        .to_string();
                    let block = BlockMessage {
                        id: NEXT_ID.fetch_add(1, Ordering::SeqCst),
                        // id: next_id,
                        text: lorem.join(" "),
                        speaker,
                    };

                    sessions::broadcast_down_msg(&DownMsg::BlockCreated(block), cor_id).await;
                    sleep(Duration::from_millis(2000)).await;

                    // if next_id > 15 {
                    //     break;
                    // }
                }
            });
        }
    }
}

#[moon::main]
async fn main() -> std::io::Result<()> {
    start(frontend, up_msg_handler, |_| {}).await
}
