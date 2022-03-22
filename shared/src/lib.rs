use moonlight::*;

// ------ UpMsg ------

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "serde")]
pub enum UpMsg {
    ChooseEvent(EventChoiceMessage),
    EditBlock(BlockMessage),
    DeleteBlock(BlockMessage),
    MergeBlockAbove(BlockMessage),
}

// ------ DownMsg ------

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "serde")]
pub enum DownMsg {
    EventSelected(EventStreamMessage),
    BlockCreated(BlockMessage),
    BlockEdited(BlockMessage),
    BlockDeleted(BlockMessage),
    BlockMergedWithAbove(BlockMessage),
}

// ------ Message ------

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "serde")]
pub struct BlockMessage {
    pub id: usize,
    pub speaker: String,
    pub words: Vec<Word>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "serde")]
pub struct EventStreamMessage {
    pub id: usize,
    pub data: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "serde")]
pub struct EventChoiceMessage {
    pub id: usize,
}

// ////////////////////////////////////////////////////////////////////////////////////////////
// Types for AAI data structures (used in deserialize calls)
#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "serde")]
struct UploadResp {
    upload_url: String, // url of file we uploaded (only accessible from AAI servers)
}

type Speaker = Option<String>; //  If it's provided, we get A, B, C, ... unclear what happens after Z, AWS allows 10

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "serde")]
pub struct Word {
    pub confidence: f32,
    pub end: usize,
    pub speaker: Speaker,
    pub start: usize,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
// TODO: Utterances appear to be grouped by speaker, while words
// seems to be a stream of ungrouped speakers
#[serde(crate = "serde")]
pub struct Utterance {
    pub confidence: f32,
    pub audio_end: usize, // realtime wants audio_start, upload just start
    pub speaker: Speaker,
    pub audio_start: usize,
    pub text: String,
    pub words: Vec<Word>,
}
