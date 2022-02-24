#!/usr/bin/env python3

# https://www.assemblyai.com/blog/real-time-speech-recognition-with-python/

from configure import auth_key
from pprint import pprint
import asyncio
import base64
import json
import pyaudio
import sys
import websockets

URL = "wss://api.assemblyai.com/v2/realtime/ws?sample_rate=16000"

FRAMES_PER_BUFFER = 3200
FORMAT = pyaudio.paInt16
CHANNELS = 1
RATE = 16000
p = pyaudio.PyAudio()

stream = p.open(
    format=FORMAT,
    channels=CHANNELS,
    rate=RATE,
    input=True,
    frames_per_buffer=FRAMES_PER_BUFFER,
)


def spinning_cursor():
    while True:
        for cursor in "|/-\\":
            yield cursor


async def send_receive():
    print("\nAllow your computer to use the microphone, then start talking.")
    print()

    spinner = spinning_cursor()
    async with websockets.connect(
        URL,
        extra_headers=[("Authorization", auth_key)],
        ping_interval=5,
        ping_timeout=20,
    ) as _ws:

        await asyncio.sleep(0.01)

        async def send():
            while True:

                try:
                    data = stream.read(FRAMES_PER_BUFFER)
                    data = base64.b64encode(data).decode("utf-8")
                    json_data = json.dumps({"audio_data": str(data)})
                    await _ws.send(json_data)
                except websockets.exceptions.ConnectionClosedError as e:
                    print(e)
                    assert e.code == 4008
                    break
                except Exception as e:
                    assert False, "Not a websocket 4008 error"

                sys.stdout.write(next(spinner))
                sys.stdout.flush()
                await asyncio.sleep(0.1)
                sys.stdout.write("\b")
            return True

        async def receive():
            last_text = ""
            while True:
                try:
                    result_str = await _ws.recv()
                    result_json = json.loads(result_str)
                    text = result_json.get("text", "")

                    if not text:
                        continue

                    if result_json.get("message_type") == "FinalTranscript":
                        sys.stdout.write("\b")
                        print(80 * "=")
                        print(text)
                        print(80 * "=")
                    else:
                        if text and text != last_text:
                            sys.stdout.write("\b")
                            print(f"... {text}")

                    # Keep track of last text, otherwise, we'll just be reprinting an identical
                    # in-progress translation line during moments of silence.
                    last_text = text

                except websockets.exceptions.ConnectionClosedError as e:
                    print(e)
                    assert e.code == 4008
                    break
                except Exception as e:
                    assert False, "Not a websocket 4008 error"

        await asyncio.gather(send(), receive())


if __name__ == "__main__":

    try:
        asyncio.run(send_receive())
    except KeyboardInterrupt:
        sys.exit(0)