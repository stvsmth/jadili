use hyper::header;
use reqwest::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::{env, fs};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    // ////////////////////////////////////////////////////////////////////////////////////////////
    // Get filename, api key
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Need to pass in a filename.");
    }

    let filename = &args[1];
    let mut f = File::open(filename).expect("Problem opening sound file.");
    let mut recording = Vec::new();
    f.read_to_end(&mut recording)
        .expect("Problem reading sound data");

    // ... grab our API key from the configuration file (not in VCS)
    let auth_key = fs::read_to_string("auth_keys.txt").expect("Problem reading auth key");

    // ////////////////////////////////////////////////////////////////////////////////////////////
    // Build a client with persistent headers
    let mut headers = header::HeaderMap::new();

    let mut auth_value = header::HeaderValue::from_str(auth_key.as_str()).unwrap();
    auth_value.set_sensitive(true);
    headers.insert(header::AUTHORIZATION, auth_value);

    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json"),
    );

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    // ////////////////////////////////////////////////////////////////////////////////////////////
    // POST our file to the server, and get the location from the response
    let up_url = "https://api.assemblyai.com/v2/upload";
    let up_resp = client.post(up_url).body(recording).send().await.unwrap(); // TODO ...handle this

    let upload_loc = match up_resp.status() {
        reqwest::StatusCode::OK => match up_resp.json::<UploadResp>().await {
            Ok(up_resp) => up_resp.upload_url,
            Err(_) => panic!("Hmm, parsing failure."),
        },
        other => panic!("Bad request {:?}", other),
    };
    println!("Upload location: {:?}", upload_loc);

    // ////////////////////////////////////////////////////////////////////////////////////////////
    // Submit uploaded file for transcription, and get transcript id for which we will poll
    let tx_url = "https://api.assemblyai.com/v2/transcript";
    let mut params = HashMap::new();
    params.insert("audio_url", upload_loc.as_str());
    params.insert("speaker_labels", "true");

    let tx_resp = client
        .post(tx_url)
        .json(&params)
        .send()
        .await?
        .json::<serde_json::Value>() // TODO: create TranscriptPost struct
        .await?;
    println!("Transcript requested");

    // ... get the transcript id, needed for use in our next call
    let tx_id = match tx_resp.get("id") {
        Some(status) => status.as_str().unwrap().to_string(), // TODO: Properly report error when we don't have `status` (i.e. bad id value)
        None => panic!("Bad JSON, no status key"),
    };
    println!("tx_id {:?}", tx_id);

    // ////////////////////////////////////////////////////////////////////////////////////////////
    // Poll the endpoint for a finished state
    println!("Polling the transcript");
    loop {
        let poll_url = format!("{}/{}", tx_url, tx_id);
        let poll_resp = client
            .get(&poll_url)
            .send()
            .await?
            .json::<TranscriptResp>()
            .await?;

        if poll_resp.status == "completed" {
            // println!("Transcript: {}", poll_resp.text.unwrap());
            let json_filename = format!("{}.json", filename);
            let out = File::create(json_filename).unwrap();
            serde_json::to_writer(out, &poll_resp).unwrap();
            break;
        }
        println!("... status: {}", poll_resp.status);
        sleep(Duration::from_millis(3000)).await;
    }

    Ok(())
}

// ////////////////////////////////////////////////////////////////////////////////////////////
// Types AAI data structures (used in deserialize calls)

#[derive(Serialize, Deserialize, Debug)]
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
struct Word {
    confidence: f32,
    end: usize,
    speaker: Speaker,
    start: usize,
    text: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Utterance {
    confidence: f32,
    end: usize,
    speaker: Speaker,
    start: usize,
    text: String,
    words: Vec<Word>,
}

#[derive(Serialize, Deserialize, Debug)]
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
struct TranscriptResp {
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
    utterances: Option<Vec<Utterance>>,
    webhook_status_code: Option<usize>,
    webhook_url: Option<String>,
    words: Option<Vec<Word>>,
    word_boost: Vec<String>,
}
