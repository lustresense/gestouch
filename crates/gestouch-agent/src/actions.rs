//! Actions run only in the current interactive user's process. No SYSTEM/UIAccess escalation.
use gestouch_core::Action;
use gestouch_win::{Input,InputUnion,KeybdInput,SendInput};
use std::mem::size_of;

fn press(vks:&[u16]) -> bool {
    if vks.is_empty() || vks.len()>6 {return false;}
    let mut events=Vec::with_capacity(vks.len()*2);
    for &vk in vks {events.push(Input{kind:1,data:InputUnion{keybd:KeybdInput{vk,scan:0,flags:0,time:0,extra:0}}});}
    for &vk in vks.iter().rev() {events.push(Input{kind:1,data:InputUnion{keybd:KeybdInput{vk,scan:0,flags:2,time:0,extra:0}}});}
    unsafe {SendInput(events.len() as u32,events.as_ptr(),size_of::<Input>() as i32)==events.len() as u32}
}
fn vk(key:&str)->Option<u16> {
    match key {
        "CTRL"|"CONTROL" => Some(0x11),"ALT"=>Some(0x12),"SHIFT"=>Some(0x10),"WIN"|"META"=>Some(0x5B),
        "TAB"=>Some(0x09),"ENTER"=>Some(0x0D),"ESC"|"ESCAPE"=>Some(0x1B),"SPACE"=>Some(0x20),
        "LEFT"=>Some(0x25),"UP"=>Some(0x26),"RIGHT"=>Some(0x27),"DOWN"=>Some(0x28),
        "BACKSPACE"=>Some(0x08),"DELETE"=>Some(0x2E),"HOME"=>Some(0x24),"END"=>Some(0x23),
        "PGUP"=>Some(0x21),"PGDN"=>Some(0x22),
        s if s.len()==1 && s.as_bytes()[0].is_ascii_alphanumeric()=>Some(s.as_bytes()[0].to_ascii_uppercase() as u16),
        s if s.starts_with('F')=>s[1..].parse::<u16>().ok().filter(|n|(1..=12).contains(n)).map(|n|0x6f+n),
        _=>None,
    }
}
pub fn execute(a:&Action)->bool {
    match a {
        Action::VolumeUp=>press(&[0xaf]),Action::VolumeDown=>press(&[0xae]),Action::VolumeMute=>press(&[0xad]),
        Action::MediaNext=>press(&[0xb0]),Action::MediaPrevious=>press(&[0xb1]),Action::MediaPlayPause=>press(&[0xb3]),
        Action::KeyCombo{keys}=>{
            let parsed:Option<Vec<u16>>=keys.split('+').map(|s|vk(s.trim().to_ascii_uppercase().as_str())).collect();
            parsed.is_some_and(|keys|press(&keys))
        }
    }
}
