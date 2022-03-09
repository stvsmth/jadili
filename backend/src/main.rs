use std::sync::atomic::{AtomicUsize, Ordering};

use lipsum::lipsum_words;
use moon::tokio::time::{sleep, Duration};
use moon::*;
use rand::prelude::*;
use shared::{BlockMessage, DownMsg, EventStreamMessage, UpMsg};

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
            let stream = EventStreamMessage {
                id: event.id,
                data: lipsum_words(5),
            };
            sessions::broadcast_down_msg(&DownMsg::EventSelected(stream), cor_id).await;

            static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

            tokio::spawn(async move {
                loop {
                    // let next_id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
                    let range = rand::thread_rng().gen_range(7..50);
                    let speaker = ['A', 'B', 'C', 'D', 'E', 'F', 'G']
                        .choose(&mut rand::thread_rng())
                        .unwrap()
                        .to_string();
                    let block = BlockMessage {
                        id: NEXT_ID.fetch_add(1, Ordering::SeqCst),
                        // id: next_id,
                        text: lipsum_words(range),
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
