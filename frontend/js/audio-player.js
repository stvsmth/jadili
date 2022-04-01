// Functions called by Rust, for now

export function load_audio() {
    const player = document.getElementById("audio-player");
    if (player) {
        player.load();
    }
}

export function play_from(position, duration) {
    const player = document.getElementById("audio-player");
    if (player) {
        player.currentTime = position;
        player.play();
        setTimeout(function() {
            player.pause();
        }, duration * 1000);
    }
}
