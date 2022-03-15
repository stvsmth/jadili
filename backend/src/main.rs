use fake::faker::lorem::en::*;
use fake::Fake;
use moon::tokio::time::{sleep, Duration};
use moon::*;
use serde::{Deserialize, Serialize};
use shared::{BlockMessage, DownMsg, EventStreamMessage, UpMsg};
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

fn get_sample_json(id: usize) -> Option<BlockMessage> {
    let path = format!("./sample_{:03}.json", id);
    match read_user_from_file(path) {
        Ok(block) => {
            let speaker = block.speaker.clone().unwrap_or_else(|| "".to_string());
            Some(BlockMessage {
                id,
                text: block.text,
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
                    if let Some(block) = get_sample_json(NEXT_ID.load(Ordering::SeqCst)) {
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

// ////////////////////////////////////////////////////////////////////////////////////////////
// Types AAI data structures (used in deserialize calls)

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "serde")]
struct UploadResp {
    upload_url: String, // url of file we uploaded (only accessible from AAI servers)
}

type Speaker = Option<String>; // A, B, C ... will revisit; maybe char? or char[2]?

// Discussion of validators for Rust json/structs
// https://blog.logrocket.com/json-input-validation-in-rust-web-services/
//
// Serde has the ability to add default values, maybe this is handy? Maybe we want null?
// https://serde.rs/attr-default.html
// For example, probably better to have `null` words rather than an empty Vec we need to get
// length of before moving forward.
//
// Also think about escape sequences (\n)
// https://d3lm.medium.com/rust-beware-of-escape-sequences-85ec90e9e243
#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "serde")]
struct Word {
    confidence: f32,
    end: usize,
    speaker: Speaker,
    start: usize,
    text: String,
}

#[derive(Serialize, Deserialize, Debug)]
// TODO: Utterances appear to be grouped by speaker, while words
// seems to be a stream of ungrouped speakers
#[serde(crate = "serde")]
struct Utterance {
    confidence: f32,
    audio_end: usize, // realtime wants audio_start, upload just start
    speaker: Speaker,
    audio_start: usize,
    text: String,
    words: Vec<Word>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "serde")]
struct Sentiment {
    text: String,
    start: usize,
    end: usize,
    sentiment: String, // POSITIVE, NEGATIVE, NEUTRAL
    confidence: f32,
    speaker: Option<Speaker>,
}

// #[derive(Serialize, Deserialize, Debug)]
// struct SafetyLabels {
//   results: Vec<???>,
//   status: String,  // unavailable,???
//   summary: {},
// },

// #[derive(Serialize, Deserialize, Debug)]
// struct IabCategories: {
//   results: Vec<???>,
//   status: String,  // unavailable,???
//   summary: {},
// }

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "serde")]
struct Transcript {
    // https://docs.assemblyai.com/core-transcription
    acoustic_model: String,
    audio_duration: Option<usize>,   // in seconds
    audio_end_at: Option<usize>,     // in ms
    audio_start_from: Option<usize>, // in ms
    audio_url: String,
    auto_chapters: bool,
    auto_highlights: bool,
    auto_highlights_result: Option<Vec<serde_json::Value>>, // Option(Vec<Highlights>)
    boost_param: Option<String>,                            // low, default, hight
    chapters: Option<Vec<serde_json::Value>>,               // Some<Vec<Chapter>>
    confidence: Option<f32>,
    content_safety: bool,
    content_safety_labels: serde_json::Value, // Option(Vec<SafetyLabel>>)
    disfluencies: bool,
    dual_channel: Option<bool>,
    entities: Option<Vec<Speaker>>,
    entity_detection: bool,
    filter_profanity: bool,
    format_text: bool,
    iab_categories: bool,
    iab_categories_result: serde_json::Value, // Option(<Vec<IabCategories>>)
    id: String,
    language_code: String, // default: en_us:  en, en_au, en_uk, en_us, fr, de, it, es
    language_model: String, // default: assemblyai_default:
    punctuate: bool,
    redact_pii: bool,
    redact_pii_audio: bool,
    redact_pii_audio_quality: Option<String>, // TODO: ????
    redact_pii_policies: Option<bool>,
    redact_pii_sub: Option<String>, // entity_type, hash
    sentiment_analysis: bool,
    sentiment_analysis_results: Option<Vec<Sentiment>>,
    speaker_labels: bool,
    speed_boost: bool,
    status: String, // queued, processing, completed, error
    text: Option<String>,
    utterances: Vec<Utterance>,
    webhook_status_code: Option<usize>,
    webhook_url: Option<String>,
    words: Option<Vec<Word>>,
    word_boost: Vec<String>,
}
