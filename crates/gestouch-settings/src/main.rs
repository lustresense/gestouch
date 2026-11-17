#![cfg(windows)]
#![windows_subsystem = "windows"]
//! Native Win32 Settings for the 0.0.1 developer preview.
//! Engine lifecycle is independent of this window.
use gestouch_core::{config,health,Action,Config,GestureRule,Trigger,GestureKind,Direction};
use gestouch_win::*;
use std::{process::Command,ptr,time::{SystemTime,UNIX_EPOCH}};
const NAV_BASE:usize=100;
const ADD_RULE:usize=210;
const DELETE_RULE:usize=211;
const OPEN_AGENT:usize=212;
const TOGGLE_ENABLED:usize=213;
const STARTUP_ON:usize=214;
const STARTUP_OFF:usize=215;
const REFRESH:usize=216;
const TIMER_REFRESH:usize=99;

struct Ui {
    config:Config, page:usize, controls:Vec<(HWND,usize)>,
    rule_list:HWND,fingers:HWND,kind:HWND,action:HWND,
    dashboard_status:HWND,diag_status:HWND,enabled_status:HWND,startup_status:HWND,
}
impl Ui {
    fn new()->Self{Self{config:config::load().unwrap_or_default(),page:0,controls:vec![],
        rule_list:0,fingers:0,kind:0,action:0,dashboard_status:0,diag_status:0,enabled_status:0,startup_status:0}}
    unsafe fn child(&mut self,hwnd:HWND,page:usize,class:&str,label:&str,x:i32,y:i32,w:i32,h:i32,id:usize,style:u32)->HWND{
        let class=wide(class);let label=wide(label);
        let child=CreateWindowExW(0,class.as_ptr(),label.as_ptr(),WS_CHILD|WS_VISIBLE|style,x,y,w,h,hwnd,id as isize,GetModuleHandleW(ptr::null()),ptr::null());
        self.controls.push((child,page));child
    }
    unsafe fn label(&mut self,hwnd:HWND,page:usize,text:&str,x:i32,y:i32,w:i32,h:i32)->HWND{
        self.child(hwnd,page,"STATIC",text,x,y,w,h,0,0)
    }
    unsafe fn button(&mut self,hwnd:HWND,page:usize,text:&str,x:i32,y:i32,w:i32,id:usize)->HWND{
        self.child(hwnd,page,"BUTTON",text,x,y,w,37,id,BS_PUSHBUTTON)
    }
    unsafe fn combo(&mut self,hwnd:HWND,page:usize,values:&[&str],x:i32,y:i32,w:i32)->HWND {
        let box_hwnd=self.child(hwnd,page,"COMBOBOX","",x,y,w,250,0,CBS_DROPDOWNLIST|WS_VSCROLL);
        for v in values {let w=wide(v);SendMessageW(box_hwnd,CB_ADDSTRING,0,w.as_ptr() as isize);}
        SendMessageW(box_hwnd,CB_SETCURSEL,0,0);box_hwnd
    }
    unsafe fn init(&mut self,hwnd:HWND) {
        self.label(hwnd,99,"Gestouch",30,24,185,38);
        self.label(hwnd,99,"0.0.1 · Rust-native developer preview",30,62,220,45);
        for (i,name) in ["Dashboard","Gestures","Diagnostics","Startup","About"].iter().enumerate(){
            self.button(hwnd,99,name,20,146+i as i32*58,207,NAV_BASE+i);
        }
        self.label(hwnd,0,"Dashboard",270,32,620,45);
        self.label(hwnd,0,"Touchpad engine",270,92,410,35);
        self.dashboard_status=self.label(hwnd,0,"Agent status pending...",270,138,620,215);
        self.enabled_status=self.label(hwnd,0,"Rules are enabled",270,350,610,35);
        self.button(hwnd,0,"Enable / pause configured rules",270,400,292,TOGGLE_ENABLED);
        self.button(hwnd,0,"Launch agent",574,400,150,OPEN_AGENT);
        self.label(hwnd,0,"No Windows gesture is replaced unless you add a matching custom rule.",270,464,650,60);

        self.label(hwnd,1,"Gesture library",270,32,620,46);
        self.label(hwnd,1,"Configured rules (first matching enabled trigger wins)",270,88,630,29);
        self.rule_list=self.child(hwnd,1,"LISTBOX","",270,118,630,166,10,WS_BORDER|WS_VSCROLL|LBS_NOTIFY);
        self.button(hwnd,1,"Delete selected",745,292,155,DELETE_RULE);
        self.label(hwnd,1,"Add gesture",270,330,590,34);
        self.label(hwnd,1,"Fingers",270,374,100,24);
        self.label(hwnd,1,"Gesture",400,374,140,24);
        self.label(hwnd,1,"Action",570,374,200,24);
        self.fingers=self.combo(hwnd,1,&["2","3","4","5"],270,400,110);
        SendMessageW(self.fingers,CB_SETCURSEL,1,0);
        self.kind=self.combo(hwnd,1,&["Swipe left","Swipe right","Swipe up","Swipe down","Tap"],400,400,150);
        self.action=self.combo(hwnd,1,&["Volume up","Volume down","Volume mute","Next track","Previous track","Play / Pause","Ctrl+Tab","Ctrl+Shift+Tab","Win+D"],570,400,330);
        self.button(hwnd,1,"Add rule",270,464,150,ADD_RULE);
        self.label(hwnd,1,"Windows can still perform built-in gestures; test conflicts on real hardware.",270,523,630,70);

        self.label(hwnd,2,"Diagnostics",270,32,620,46);
        self.label(hwnd,2,"Agent health snapshot · updated every 30 seconds while running",270,86,610,36);
        self.diag_status=self.label(hwnd,2,"No snapshot available",270,144,620,330);
        self.button(hwnd,2,"Refresh",270,493,165,REFRESH);
        self.label(hwnd,2,"Idle silence is normal. Re-register input only after confirmed mismatch or resume.",270,546,620,50);

        self.label(hwnd,3,"Startup and service",270,32,630,46);
        self.label(hwnd,3,"Per-user startup (recommended: lowest overhead)",270,90,650,35);
        self.startup_status=self.label(hwnd,3,"Startup status",270,140,620,38);
        self.button(hwnd,3,"Enable at login",270,203,205,STARTUP_ON);
        self.button(hwnd,3,"Disable at login",495,203,205,STARTUP_OFF);
        self.label(hwnd,3,"Optional genuine Windows Service (administrator install):",270,283,630,34);
        self.label(hwnd,3,"Run scripts\\install.ps1 -Service from elevated PowerShell. The service is a\nsupervisor only; gestures are handled by this user's agent, never Session 0.\nService Mode is experimental in 0.0.1.",270,335,650,115);

        self.label(hwnd,4,"About Gestouch",270,32,620,46);
        self.label(hwnd,4,"Gestouch 0.0.1 · Independent Rust implementation",270,103,620,44);
        self.label(hwnd,4,"Windows 11 x64 · native Win32 GUI and per-user background engine\nLicense: MIT OR Apache-2.0\nUpstream GestureSign V2 remains GPL-2.0 and is NOT included.\n\nStatus: Developer preview. HID behavior and memory usage require real Windows validation.\nUI: Native shell in alpha; complete design flows are documented in docs/WIREFRAMES.html.",270,170,640,320);
        SetTimer(hwnd,TIMER_REFRESH,3500,None);
        self.render();self.refresh();
    }
    unsafe fn render(&self){
        for (control, page) in &self.controls {ShowWindow(*control,if *page==99||*page==self.page{SW_SHOW}else{SW_HIDE});}
    }
    unsafe fn refresh_status(&mut self) {
        SetWindowTextW(self.dashboard_status,wide(&health::read_status()).as_ptr());
        SetWindowTextW(self.diag_status,wide(&health::read_status()).as_ptr());
    }
    unsafe fn refresh(&mut self) {
        self.config=config::load().unwrap_or_default();
        self.refresh_status();
        SetWindowTextW(self.enabled_status,wide(if self.config.enabled{"Gesture rules: enabled"}else{"Gesture rules: paused"}).as_ptr());
        SetWindowTextW(self.startup_status,wide(if self.config.start_at_login{"Login task: configured by Gestouch"}else{"Login task: not enabled in Gestouch settings"}).as_ptr());
        SendMessageW(self.rule_list,LB_RESETCONTENT,0,0);
        for rule in &self.config.rules {
            let text=format!("{} · {} fingers · {:?} {:?} → {:?}",if rule.enabled{"●"}else{"○"},rule.trigger.fingers,rule.trigger.kind,rule.trigger.direction,rule.action);
            let w=wide(&text);SendMessageW(self.rule_list,LB_ADDSTRING,0,w.as_ptr() as isize);
        }
    }
    unsafe fn on_command(&mut self,hwnd:HWND,id:usize){
        if (NAV_BASE..NAV_BASE+5).contains(&id){self.page=id-NAV_BASE;self.render();self.refresh();return;}
        match id {
            ADD_RULE=>{
                let f=(SendMessageW(self.fingers,CB_GETCURSEL,0,0).max(0) as u8)+2;
                let trigger_index=SendMessageW(self.kind,CB_GETCURSEL,0,0);
                let (kind,dir)=match trigger_index {
                    1=>(GestureKind::Swipe,Direction::Right),
                    2=>(GestureKind::Swipe,Direction::Up),
                    3=>(GestureKind::Swipe,Direction::Down),
                    4=>(GestureKind::Tap,Direction::None),
                    _=>(GestureKind::Swipe,Direction::Left),
                };
                let action=match SendMessageW(self.action,CB_GETCURSEL,0,0) {
                    1=>Action::VolumeDown,2=>Action::VolumeMute,3=>Action::MediaNext,
                    4=>Action::MediaPrevious,5=>Action::MediaPlayPause,
                    6=>Action::KeyCombo{keys:"CTRL+TAB".into()},
                    7=>Action::KeyCombo{keys:"CTRL+SHIFT+TAB".into()},
                    8=>Action::KeyCombo{keys:"WIN+D".into()},
                    _=>Action::VolumeUp,
                };
                if self.config.rules.iter().any(|r|r.trigger.kind==kind&&r.trigger.fingers==f&&r.trigger.direction==dir&&r.enabled){
                    self.alert(hwnd,"This gesture already has an enabled rule. Remove it first.");return;
                }
                let id=SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
                self.config.rules.push(GestureRule{id:format!("rule-{id}"),name:format!("{f}-finger {kind:?} {dir:?}"),enabled:true,trigger:Trigger{fingers:f,kind,direction:dir},action});
                self.save_and_refresh(hwnd);
            }
            DELETE_RULE=>{
                let sel=SendMessageW(self.rule_list,LB_GETCURSEL,0,0);
                if sel>=0 && (sel as usize)<self.config.rules.len(){self.config.rules.remove(sel as usize);self.save_and_refresh(hwnd);}
            }
            TOGGLE_ENABLED=>{self.config.enabled=!self.config.enabled;self.save_and_refresh(hwnd);}
            OPEN_AGENT=>{if let Ok(me)=std::env::current_exe(){let exe=me.with_file_name("gestouch-agent.exe");if let Err(e)=Command::new(exe).spawn(){self.alert(hwnd,&format!("Cannot launch agent: {e}"));}}}
            STARTUP_ON=>{self.set_startup(hwnd,true);}
            STARTUP_OFF=>{self.set_startup(hwnd,false);}
            REFRESH=>self.refresh(),
            _=>{}
        }
    }
    unsafe fn save_and_refresh(&mut self,hwnd:HWND){
        if let Err(e)=config::save(&self.config){self.alert(hwnd,&e);}self.refresh();
    }
    unsafe fn alert(&self,hwnd:HWND,text:&str){MessageBoxW(hwnd,wide(text).as_ptr(),wide("Gestouch").as_ptr(),0);}
    unsafe fn set_startup(&mut self,hwnd:HWND,enable:bool){
        let Some(exe)=std::env::current_exe().ok().map(|e|e.with_file_name("gestouch-agent.exe"))else{return;};
        if !exe.is_file(){self.alert(hwnd,"gestouch-agent.exe must be in the same directory.");return;}
        let mut cmd=Command::new("schtasks.exe");
        if enable {
            cmd.args(["/Create","/F","/SC","ONLOGON","/IT","/RL","LIMITED","/TN","GestouchAgent","/TR"]);
            cmd.arg(format!("\"{}\"",exe.display()));
        }else{cmd.args(["/Delete","/F","/TN","GestouchAgent"]);}
        match cmd.output(){
            Ok(out) if out.status.success()=>{self.config.start_at_login=enable;self.save_and_refresh(hwnd);}
            Ok(out)=>self.alert(hwnd,&format!("Task Scheduler failed:\n{}",String::from_utf8_lossy(&out.stderr))),
            Err(e)=>self.alert(hwnd,&format!("Could not use Task Scheduler: {e}")),
        }
    }
}
static mut UI:Option<Ui>=None;
unsafe extern "system" fn window_proc(hwnd:HWND,msg:u32,wp:usize,lp:isize)->isize {
    match msg {
        WM_CREATE=>{if let Some(ui)=UI.as_mut(){ui.init(hwnd)};0}
        WM_COMMAND=>{if let Some(ui)=UI.as_mut(){ui.on_command(hwnd,wp&0xffff)};0}
        WM_TIMER=>{if wp==TIMER_REFRESH {if let Some(ui)=UI.as_mut(){ui.refresh_status()}};0}
        WM_DESTROY=>{KillTimer(hwnd,TIMER_REFRESH);PostQuitMessage(0);0}
        _=>DefWindowProcW(hwnd,msg,wp,lp)
    }
}
fn main(){unsafe {
    UI=Some(Ui::new());
    let name=wide("GestouchSettingsMainWindow");
    match create_class(&name,window_proc){
        Ok(instance)=>{
            let window=CreateWindowExW(0,name.as_ptr(),wide("Gestouch 0.0.1 — Touchpad settings").as_ptr(),
                WS_OVERLAPPEDWINDOW,120,90,980,690,0,0,instance,ptr::null());
            if window!=0 {ShowWindow(window,SW_SHOW);UpdateWindow(window);message_loop();}
            else {MessageBoxW(0,wide("CreateWindowExW failed").as_ptr(),wide("Gestouch").as_ptr(),0);}
        }
        Err(e)=>{MessageBoxW(0,wide(&e).as_ptr(),wide("Gestouch").as_ptr(),0);}
    }
    UI=None;
}}
