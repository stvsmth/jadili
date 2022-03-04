use lipsum::lipsum_words;
use moon::*;
use shared::{DownMsg, UpMsg, EventStreamMessage};

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
            let stream = EventStreamMessage{id: event.id, data: 42};
            sessions::broadcast_down_msg(&DownMsg::EventSelected(stream), cor_id).await;
        }
    }
}

#[moon::main]
async fn main() -> std::io::Result<()> {
    start(frontend, up_msg_handler, |_| {}).await
}
