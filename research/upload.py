#!/usr/bin/env python3

from configure import auth_key
from pathlib import Path
from time import sleep
import pprint
import requests
import sys

headers = {"authorization": auth_key, "content-type": "application/json"}
transcript_endpoint = "https://api.assemblyai.com/v2/transcript"
upload_endpoint = "https://api.assemblyai.com/v2/upload"


def read_file(filename):
    with open(filename, "rb") as f:
        while True:
            data = f.read(5242880)
            if not data:
                break
            yield data


input_filename = sys.argv[1]
base_filename = Path(input_filename).stem
output_filename = base_filename + ".json"

upload_response = requests.post(
    upload_endpoint, headers=headers, data=read_file(input_filename)
)
upload_url = upload_response.json()["upload_url"]
print(f"Audio file uploaded: {upload_url}")

# send a request to transcribe the audio file
transcript_request = {"audio_url": upload_url, "speaker_labels": True}
transcript_response = requests.post(
    transcript_endpoint, json=transcript_request, headers=headers
)
print("Transcription Requested")
pprint.pprint(transcript_response.json())

# set up polling
polling_response = requests.get(
    transcript_endpoint + "/" + transcript_response.json()["id"], headers=headers
)

# if our status isn’t complete, sleep and then poll again
while polling_response.json()["status"] != "completed":
    sleep(20)
    polling_response = requests.get(
        transcript_endpoint + "/" + transcript_response.json()["id"], headers=headers
    )
    print("File is", polling_response.json()["status"])

with open(output_filename, "w") as f:
    f.write(polling_response.text)
print("Transcript saved to", output_filename)
