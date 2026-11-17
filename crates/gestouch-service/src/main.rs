#![cfg(windows)]
#![windows_subsystem = "windows"]
//! Optional real SCM service. It never captures HID or sends keyboard events.
//! Restarts *the registered per-user interactive Task Scheduler task*, rather
//! than spawning a SYSTEM desktop process. The service is opt-in only.
use std::{ffi::c_void,os::windows::ffi::OsStrExt,process::{Command,Stdio},ptr,sync::atomic::{AtomicBool,Ordering},thread,time::Duration};
const NAME:&str="GestouchSupervisor";
const TYPE_WIN32_OWN_PROCESS:u32=0x10;
const STATE_STOPPED:u32=1;
const STATE_START_PENDING:u32=2;
const STATE_STOP_PENDING:u32=3;
const STATE_RUNNING:u32=4;
const ACCEPT_STOP:u32=1;
const CONTROL_STOP:u32=1;
const CREATE_NO_WINDOW:u32=0x08000000;
#[repr(C)]
struct ServiceTableEntry {name:*mut u16,main:Option<unsafe extern "system" fn(u32,*mut *mut u16)>}
#[repr(C)]
struct ServiceStatus {service_type:u32,current_state:u32,controls_accepted:u32,win32_exit_code:u32,
    service_specific_exit_code:u32,check_point:u32,wait_hint:u32}
#[link(name="advapi32")]
extern "system" {
    fn StartServiceCtrlDispatcherW(table:*const ServiceTableEntry)->i32;
    fn RegisterServiceCtrlHandlerW(name:*const u16,handler:Option<unsafe extern "system" fn(u32)>)->isize;
    fn SetServiceStatus(handle:isize,status:*const ServiceStatus)->i32;
}
static STOP:AtomicBool=AtomicBool::new(false);
static mut STATUS_HANDLE:isize=0;
fn wide(s:&str)->Vec<u16>{std::ffi::OsStr::new(s).encode_wide().chain(Some(0)).collect()}
unsafe fn report(state:u32,accept:u32,wait:u32){
    let status=ServiceStatus{service_type:TYPE_WIN32_OWN_PROCESS,current_state:state,
        controls_accepted:accept,win32_exit_code:0,service_specific_exit_code:0,check_point:0,wait_hint:wait};
    if STATUS_HANDLE!=0 {SetServiceStatus(STATUS_HANDLE,&status);}
}
unsafe extern "system" fn control(control:u32){
    if control==CONTROL_STOP {STOP.store(true,Ordering::SeqCst);report(STATE_STOP_PENDING,0,10000);}
}
unsafe extern "system" fn service_main(_argc:u32,_argv:*mut *mut u16){
    let name=wide(NAME);
    STATUS_HANDLE=RegisterServiceCtrlHandlerW(name.as_ptr(),Some(control));
    if STATUS_HANDLE==0{return;}
    report(STATE_START_PENDING,0,2000);
    report(STATE_RUNNING,ACCEPT_STOP,0);
    // The task is registered by install.ps1 under the consented user's token.
    // Task Scheduler prevents concurrent copies; the agent also holds a mutex.
    // Service's only fixed action: request Task Scheduler to start that task.
    while !STOP.load(Ordering::SeqCst) {
        use std::os::windows::process::CommandExt;
        let _=Command::new("schtasks.exe").args(["/Run","/TN","GestouchAgent"])
            .creation_flags(CREATE_NO_WINDOW).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status();
        for _ in 0..12 {
            if STOP.load(Ordering::SeqCst) {break;}
            thread::sleep(Duration::from_secs(5));
        }
    }
    report(STATE_STOPPED,0,0);
}
fn main(){unsafe{
    let mut name=wide(NAME);
    let table=[ServiceTableEntry{name:name.as_mut_ptr(),main:Some(service_main)},
        ServiceTableEntry{name:ptr::null_mut(),main:None}];
    let _=StartServiceCtrlDispatcherW(table.as_ptr());
}}
