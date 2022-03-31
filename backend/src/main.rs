use fake::faker::lorem::en::*;
use fake::Fake;
use moon::tokio::time::{sleep, Duration};
use moon::*;
use shared::{BlockMessage, DownMsg, EventStreamMessage, UpMsg, Utterance};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

fn read_user_from_file<P: AsRef<Path>>(path: P) -> Result<Utterance, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let tx = serde_json::from_reader(reader)?;
    Ok(tx)
}

fn get_transcription_results(id: usize) -> Option<BlockMessage> {
    let path = format!("./public/assets/block_{:04}.json", id);
    match read_user_from_file(path) {
        Ok(block) => {
            let speaker = block.speaker.clone().unwrap_or_else(|| "".to_string());
            Some(BlockMessage {
                id,
                words: block.words,
                speaker,
            })
        }
        Err(_) => {
            // TODO: Only squash this if it is file not found (2)
            // println!("Err kind: {:?}", err);
            None
        }
    }
}

async fn frontend() -> Frontend {
    Frontend::new()
        .title("Jadili")
        .default_styles(false)
        .append_to_head(r#"<link href="/_api/public/css/currentStyle.css" rel="stylesheet"/>"#)
        .body_content(  // FIXME: Pull JS setCurrentTime once we get the Rust / web_sys stuff
            r#"
            <div id="main"></div>
            <script>
            function setCurrentTime(positionInSeconds) {
                let audio = document.getElementById("audio-player");
                audio.currentTime = positionInSeconds;
            }
            </script>
            "#,
        )
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
        UpMsg::MergeBlockAbove(block) => {
            sessions::broadcast_down_msg(&DownMsg::BlockMergedWithAbove(block), cor_id).await;
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
                    // We may not have the next file on disk, no worries, sleep and come back later
                    let id = NEXT_ID.load(Ordering::SeqCst);
                    if let Some(block) = get_transcription_results(NEXT_ID.load(Ordering::SeqCst)) {
                        sessions::broadcast_down_msg(&DownMsg::BlockCreated(block), cor_id).await;
                        NEXT_ID.store(id + 1, Ordering::SeqCst);
                    }
                    sleep(Duration::from_millis(500)).await; // TODO: tighten this up once it's working
                }
            });
        }
    }
}

#[moon::main]
async fn main() -> std::io::Result<()> {
    start(frontend, up_msg_handler, |_| {}).await
}
