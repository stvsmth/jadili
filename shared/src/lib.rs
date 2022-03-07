use moonlight::*;

// ------ UpMsg ------

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "serde")]
pub enum UpMsg {
    ChooseEvent(EventChoiceMessage),
    SendBlock(BlockMessage),
}

// ------ DownMsg ------

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "serde")]
pub enum DownMsg {
    BlockReceived(BlockMessage),
    EventSelected(EventStreamMessage),
}

// ------ Message ------

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "serde")]
pub struct BlockMessage {
    pub id: usize,
    pub speaker: String,
    pub text: String,
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