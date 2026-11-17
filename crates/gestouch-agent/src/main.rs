#![cfg(windows)]
#![windows_subsystem = "windows"]
mod actions;
mod input;
use gestouch_core::{config,health,Recognizer,Config};
use gestouch_win::*;
use std::{mem::size_of,process::Command,ptr};
const TRAY_MESSAGE:u32=WM_APP+1;
const TIMER_HEALTH:usize=1;
const TIMER_CONFIG:usize=2;
const TIMER_RECOVER:usize=3;
const ERROR_ALREADY_EXISTS:u32=183;

struct Agent {
    raw:input::RawReader, recognizer:Recognizer, config:Config,
    paused:bool,recognized:u64, note:String, hwnd:HWND, tray:NotifyIconDataW,
}
impl Agent {
    fn new()->Self {
        let loaded=config::load();
        let note=match &loaded {Ok(_)=>"Ready; input feasibility requires real Precision Touchpad testing".into(),Err(e)=>format!("Config error: {e}; no user rules enabled")};
        Self {raw:input::RawReader::default(),recognizer:Recognizer::new(),
            config:loaded.unwrap_or_default(),paused:false,recognized:0,note,hwnd:0,tray:NotifyIconDataW::default()}
    }
    unsafe fn start(&mut self,hwnd:HWND) {
        self.hwnd=hwnd;
        self.register_input();
        self.tray.hwnd=hwnd;self.tray.callback_message=TRAY_MESSAGE;
        self.tray.flags=NIF_ICON|NIF_TIP|NIF_MESSAGE;
        self.tray.icon=LoadIconW(0,32512usize as *const u16);
        let label=wide("Gestouch 0.0.1 - Double click: Settings; right click: Pause/Resume");
        self.tray.tip[..label.len().min(128)].copy_from_slice(&label[..label.len().min(128)]);
        Shell_NotifyIconW(NIM_ADD,&self.tray);
        SetTimer(hwnd,TIMER_HEALTH,30000,None);
        SetTimer(hwnd,TIMER_CONFIG,5000,None);
        self.status();
    }
    unsafe fn register_input(&mut self) {
        let dev=RawInputDevice{usage_page:0x0d,usage:0x05,flags:RIDEV_INPUTSINK|RIDEV_DEVNOTIFY,target:self.hwnd};
        if RegisterRawInputDevices(&dev,1,size_of::<RawInputDevice>() as u32)==0 {
            self.note=format!("Raw input registration failed: {}",GetLastError());
        } else {self.note="Listening for Precision Touchpad reports".into();}
    }
    unsafe fn registered(&self)->Option<bool>{
        let mut count=0u32;
        let step=GetRegisteredRawInputDevices(ptr::null_mut(),&mut count,size_of::<RawInputDevice>() as u32);
        if step==u32::MAX || count>4096 {return None;}
        let mut devices=vec![RawInputDevice{usage_page:0,usage:0,flags:0,target:0};count as usize];
        if count>0 && GetRegisteredRawInputDevices(devices.as_mut_ptr(),&mut count,size_of::<RawInputDevice>() as u32)==u32::MAX{return None;}
        Some(devices.iter().any(|d|d.usage_page==0x0d&&d.usage==0x05&&d.target==self.hwnd&&(d.flags&RIDEV_INPUTSINK)!=0))
    }
    fn status(&self) {
        health::write_status(if self.paused||!self.config.enabled {"paused"} else {"listening"},
            self.raw.raw_messages,self.raw.completed_frames,self.recognized,&self.note);
    }
    unsafe fn on_input(&mut self,lp:isize) {
        for frame in self.raw.ingest(lp,GetTickCount64()) {
            if self.paused||!self.config.enabled {self.recognizer.reset();continue;}
            if let Some(recognized)=self.recognizer.update(frame) {
                if let Some(action)=self.config.matching(recognized) {
                    let succeeded=actions::execute(action);
                    self.recognized+=1;
                    if !succeeded {self.note="Action rejected or SendInput failed (check UIPI/hotkey)".into();}
                }
            }
        }
    }
    unsafe fn recover(&mut self,reason:&str){
        self.raw.reset();self.recognizer.reset();self.register_input();
        self.note=format!("Input re-registered after {reason}");self.status();
    }
    unsafe fn health_check(&mut self){
        if self.registered()==Some(false) {self.recover("confirmed registration mismatch");}
        else {self.status();}
    }
    fn reload(&mut self){
        match config::load(){
            Ok(new_cfg)=>{self.config=new_cfg;},
            Err(e)=>{self.config=Config::default();self.note=format!("Config rejected: {e}");}
        }
    }
    unsafe fn cleanup(&mut self){
        KillTimer(self.hwnd,TIMER_HEALTH);KillTimer(self.hwnd,TIMER_CONFIG);KillTimer(self.hwnd,TIMER_RECOVER);
        Shell_NotifyIconW(NIM_DELETE,&self.tray);self.note="Agent exited".into();self.status();
    }
}
static mut AGENT:Option<Agent>=None;
unsafe extern "system" fn window_proc(hwnd:HWND,msg:u32,wp:usize,lp:isize)->isize {
    match msg {
        WM_CREATE=> {if let Some(app)=AGENT.as_mut(){app.start(hwnd)};0}
        WM_INPUT=> {if let Some(app)=AGENT.as_mut(){app.on_input(lp)};DefWindowProcW(hwnd,msg,wp,lp)}
        WM_INPUT_DEVICE_CHANGE|WM_DEVICECHANGE=>{
            if let Some(app)=AGENT.as_mut(){app.raw.reset();app.recognizer.reset();}
            SetTimer(hwnd,TIMER_RECOVER,700,None);0
        }
        WM_POWERBROADCAST=>{
            if wp==PBT_APMRESUMEAUTOMATIC||wp==PBT_APMRESUMESUSPEND {
                if let Some(app)=AGENT.as_mut(){app.raw.reset();app.recognizer.reset();}
                SetTimer(hwnd,TIMER_RECOVER,700,None);
            } 1
        }
        WM_TIMER=>{
            if let Some(app)=AGENT.as_mut(){match wp {
                TIMER_HEALTH=>app.health_check(),
                TIMER_CONFIG=>app.reload(),
                TIMER_RECOVER=>{KillTimer(hwnd,TIMER_RECOVER);app.recover("device/power event");},
                _=>{}
            }} 0
        }
        TRAY_MESSAGE=>{
            if lp==WM_LBUTTONDBLCLK {open_settings();}
            if lp==WM_RBUTTONUP {if let Some(app)=AGENT.as_mut(){app.paused=!app.paused;app.recognizer.reset();app.note=if app.paused{"Paused by user"}else{"Resumed by user"}.into();app.status();}}
            0
        }
        WM_DESTROY=>{if let Some(app)=AGENT.as_mut(){app.cleanup()};PostQuitMessage(0);0}
        _=>DefWindowProcW(hwnd,msg,wp,lp)
    }
}
fn open_settings() {
    if let Ok(path)=std::env::current_exe() {
        let sibling=path.with_file_name("gestouch-settings.exe");
        let _=Command::new(sibling).spawn();
    }
}
fn main(){
    unsafe {
        let name=wide("Local\\GestouchAgent_0_0_1");
        let mutex=CreateMutexW(ptr::null(),1,name.as_ptr());
        if mutex==0||GetLastError()==ERROR_ALREADY_EXISTS{return;}
        AGENT=Some(Agent::new());
        let cls=wide("GestouchHiddenInputWindow");
        match create_class(&cls,window_proc) {
            Ok(instance)=>{
                // HWND_MESSAGE (-3) gives us a queue without a visible settings window.
                let hwnd=CreateWindowExW(0,cls.as_ptr(),wide("Gestouch Agent").as_ptr(),0,
                    0,0,0,0,-3,0,instance,ptr::null());
                if hwnd==0 {MessageBoxW(0,wide("Could not create input window").as_ptr(),wide("Gestouch").as_ptr(),0);}
                else {message_loop();}
            }
            Err(err)=>{MessageBoxW(0,wide(&err).as_ptr(),wide("Gestouch initialization failed").as_ptr(),0);}
        }
        AGENT=None;
        CloseHandle(mutex);
    }
}
