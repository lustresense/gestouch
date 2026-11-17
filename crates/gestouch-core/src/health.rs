use std::{fs, time::{SystemTime, UNIX_EPOCH}};
use crate::config::data_dir;

pub fn write_status(state: &str, messages: u64, frames: u64, gestures: u64, note: &str) {
    let root = data_dir();
    if fs::create_dir_all(&root).is_err() { return; }
    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    // Status is informational, not used as an authorization or liveness proof.
    let safe_note = note.replace(['\n','\r'], " ");
    let data = format!("updated_unix={time}\nstate={state}\nraw_messages={messages}\nframes={frames}\ngestures={gestures}\nnote={safe_note}\n");
    let _ = fs::write(root.join("agent-status.txt"), data);
}
pub fn read_status() -> String {
    fs::read_to_string(data_dir().join("agent-status.txt"))
        .unwrap_or_else(|_| "Agent status not available. Start gestouch-agent.exe.".to_owned())
}
