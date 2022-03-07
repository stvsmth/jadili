use lipsum::lipsum_words;
use moon::*;
use moon::tokio::time::{sleep, Duration};

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
        UpMsg::SendBlock(mut block) => {
            block.text = lipsum_words(7);
            sessions::broadcast_down_msg(&DownMsg::BlockReceived(block), cor_id).await;
        }
        UpMsg::ChooseEvent(event) => {
            let stream = EventStreamMessage {
                id: event.id,
                data: lipsum_words(5),
            };
            sessions::broadcast_down_msg(&DownMsg::EventSelected(stream), cor_id).await;
            // let range = rand::thread_rng().gen_range(7..150);
            // let speaker = ['A', 'B', 'C', 'D', 'E', 'F', 'G'];
            let mut id = 1;
            loop {
                let block = BlockMessage {
                    id,
                    text: lipsum_words(12),
                    speaker: "A".to_string(),
                };
                sessions::broadcast_down_msg(&DownMsg::BlockReceived(block), cor_id).await;
                id += 1;
                sleep(Duration::from_millis(3000)).await;
            }
        }
    }
}
// async fn get_transcript_data(id: usize) {
//     let data = lipsum_words(5);
// }

#[moon::main]
async fn main() -> std::io::Result<()> {
    start(frontend, up_msg_handler, |_| {}).await
}
