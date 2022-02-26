use hyper::header;
use reqwest::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::{env, fs};
use tokio::time::{sleep, Duration};

/* TODO
 * About the as_str().unwrap() stuff on JSON keys
   - See for discussion: https://github.com/serde-rs/json#operating-on-untyped-json-values
     >>>
     The result of square bracket indexing like v["name"] is a borrow of the data at that
     index, so the type is &Value ...

     When a Value is printed, it is printed as a JSON string. So in the code above, the output
     looks like Please call "John Doe" at the number "+44 1234567". The quotation marks appear
     because v["name"] is a &Value containing a JSON string and its JSON representation is
     "John Doe". Printing as a plain string without quotation marks involves converting
     from a JSON string to a Rust string with as_str() or avoiding the use of Value [by
     using strongly typed data structures described] in the following section.
     <<<

*/
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

    let content_type = header::HeaderValue::from_static("application/json");
    headers.insert(header::CONTENT_TYPE, content_type);

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
        .json::<serde_json::Value>()
        .await?;
    println!("Transcript requested");

    // ... get the transcript id, needed for use in our next call
    let tx_id = match tx_resp.get("id") {
        Some(status) => status.as_str().unwrap().to_string(), // TODO: Properly report error when we don't have `status` (i.e. bad id value)
        None => panic!("Bad JSON, no status key"),
    };

    // ////////////////////////////////////////////////////////////////////////////////////////////
    // Poll the endpoint for a finished state
    println!("Polling the transcript");
    loop {
        let poll_url = format!("{}/{}", tx_url, tx_id);
        let poll_resp = client
            .get(&poll_url)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let status = match poll_resp.get("status") {
            Some(tx_status) => (tx_status.as_str().unwrap().to_string()),
            None => panic!("Missing status key."),
        };

        if status == "completed" {
            println!("Transcript: {}", poll_resp.get("text").unwrap());
            let json_filename = format!("{}.json", filename);
            let pretty_json = serde_json::to_string_pretty(&poll_resp).unwrap();
            let out = std::fs::File::create(json_filename).unwrap();
            serde_json::to_writer(out, &pretty_json).unwrap();
            break;
        }
        println!("... status: {}", status);
        sleep(Duration::from_millis(5000)).await;
    }

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct UploadResp {
    upload_url: String, // url of file we uploaded (only accessible from AAI servers)
}
