// Functions called by Rust, for now

export function loadAudio() {
    const player = document.getElementById("audio-player");
    if (player) {
        player.load();
    }
}

export function playFrom(position, duration) {
    const player = document.getElementById("audio-player");
    if (player) {
        player.currentTime = position;
        player.play();
        setTimeout(function() {
            player.pause();
        }, duration * 1000);
    }
}
