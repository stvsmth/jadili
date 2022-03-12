use serde::{Deserialize, Serialize};

use std::error::Error;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;

fn read_user_from_file<P: AsRef<Path>>(path: P) -> Result<Transcript, Box<dyn Error>> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let tx = serde_json::from_reader(reader)?;
    Ok(tx)
}

fn main() {
    let paths = fs::read_dir("./").unwrap();

    for path in paths {
        let curr_path = path.unwrap().path();
        let file_stem = curr_path.file_stem().unwrap().to_string_lossy();
        if file_stem.starts_with("sample_") {
            let tx = read_user_from_file(curr_path).unwrap();
            let speaker = tx.utterances[0]
                .speaker
                .clone()
                .unwrap_or_else(|| "n/a".to_string());
            println!("Speaker {} spoketh: {:#?}", speaker, tx.utterances[0].text);
        }
    }
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
// TODO: Utterances appear to be grouped by speaker, while words
// seems to be a stream of ungrouped speakers
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
