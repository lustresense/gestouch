//! Minimal Win32 FFI shared by the per-user Rust programs.
//! All external functions are unsafe; caller owns HWND and message-loop lifetime.
#![cfg(windows)]
use std::{ffi::c_void, os::windows::ffi::OsStrExt};

pub type HWND = isize;
pub type HINSTANCE = isize;
pub type HICON = isize;
pub type HMENU = isize;
pub type HANDLE = isize;

pub const WM_CREATE: u32 = 0x0001;
pub const WM_DESTROY: u32 = 0x0002;
pub const WM_CLOSE: u32 = 0x0010;
pub const WM_COMMAND: u32 = 0x0111;
pub const WM_TIMER: u32 = 0x0113;
pub const WM_INPUT: u32 = 0x00ff;
pub const WM_DEVICECHANGE: u32 = 0x0219;
pub const WM_POWERBROADCAST: u32 = 0x0218;
pub const WM_INPUT_DEVICE_CHANGE: u32 = 0x00fe;
pub const WM_APP: u32 = 0x8000;
pub const WM_LBUTTONDBLCLK: isize = 0x0203;
pub const WM_RBUTTONUP: isize = 0x0205;
pub const PBT_APMRESUMEAUTOMATIC: usize = 0x0012;
pub const PBT_APMRESUMESUSPEND: usize = 0x0007;
pub const WS_VISIBLE: u32 = 0x10000000;
pub const WS_CHILD: u32 = 0x40000000;
pub const WS_OVERLAPPEDWINDOW: u32 = 0x00cf0000;
pub const WS_CAPTION: u32 = 0x00c00000;
pub const WS_SYSMENU: u32 = 0x00080000;
pub const WS_MINIMIZEBOX: u32 = 0x00020000;
pub const WS_BORDER: u32 = 0x00800000;
pub const WS_VSCROLL: u32 = 0x00200000;
pub const WS_EX_CLIENTEDGE: u32 = 0x00000200;
pub const BS_PUSHBUTTON: u32 = 0;
pub const CBS_DROPDOWNLIST: u32 = 0x0003;
pub const LBS_NOTIFY: u32 = 0x0001;
pub const WM_SETFONT: u32 = 0x0030;
pub const CB_ADDSTRING: u32 = 0x0143;
pub const CB_SETCURSEL: u32 = 0x014e;
pub const CB_GETCURSEL: u32 = 0x0147;
pub const LB_ADDSTRING: u32 = 0x0180;
pub const LB_RESETCONTENT: u32 = 0x0184;
pub const LB_GETCURSEL: u32 = 0x0188;
pub const SW_HIDE: i32 = 0;
pub const SW_SHOW: i32 = 5;
pub const RIDEV_INPUTSINK: u32 = 0x00000100;
pub const RIDEV_DEVNOTIFY: u32 = 0x00002000;
pub const RIDEV_REMOVE: u32 = 0x00000001;
pub const RID_INPUT: u32 = 0x10000003;
pub const RIDI_PREPARSEDDATA: u32 = 0x20000005;
pub const NIM_ADD: u32 = 0;
pub const NIM_DELETE: u32 = 2;
pub const NIF_MESSAGE: u32 = 1;
pub const NIF_ICON: u32 = 2;
pub const NIF_TIP: u32 = 4;

#[repr(C)] #[derive(Clone, Copy, Default)]
pub struct Point { pub x: i32, pub y: i32 }
#[repr(C)] #[derive(Clone, Copy)]
pub struct Msg {
    pub hwnd: HWND, pub message: u32, pub wparam: usize, pub lparam: isize,
    pub time: u32, pub pt: Point, pub lprivate: u32,
}
#[repr(C)]
pub struct WndClassW {
    pub style: u32,
    pub wndproc: Option<unsafe extern "system" fn(HWND,u32,usize,isize)->isize>,
    pub cls_extra: i32, pub wnd_extra: i32,
    pub instance: HINSTANCE, pub icon: HICON, pub cursor: isize, pub background: isize,
    pub menu_name: *const u16, pub class_name: *const u16,
}
#[repr(C)] #[derive(Clone, Copy)]
pub struct RawInputDevice { pub usage_page: u16, pub usage: u16, pub flags: u32, pub target: HWND }
#[repr(C)] #[derive(Clone, Copy)]
pub struct RawInputHeader { pub data_type: u32, pub size: u32, pub device: HANDLE, pub wparam: usize }
#[repr(C)]
pub struct NotifyIconDataW {
    pub cb_size: u32, pub hwnd: HWND, pub uid: u32, pub flags: u32,
    pub callback_message: u32, pub icon: HICON, pub tip: [u16; 128],
    pub state: u32, pub state_mask: u32, pub info: [u16; 256],
    pub timeout_or_version: u32, pub info_title: [u16; 64],
    pub info_flags: u32, pub guid: [u8; 16], pub balloon_icon: HICON,
}
impl Default for NotifyIconDataW {
    fn default() -> Self { Self { cb_size:std::mem::size_of::<Self>() as u32, hwnd:0,uid:1,flags:0,
        callback_message:0,icon:0,tip:[0;128],state:0,state_mask:0,info:[0;256],
        timeout_or_version:0,info_title:[0;64],info_flags:0,guid:[0;16],balloon_icon:0 } }
}

#[link(name="user32")]
extern "system" {
    pub fn RegisterClassW(class: *const WndClassW) -> u16;
    pub fn CreateWindowExW(ex_style:u32,class:*const u16,title:*const u16,style:u32,
        x:i32,y:i32,width:i32,height:i32,parent:HWND,menu:HMENU,instance:HINSTANCE,param:*const c_void)->HWND;
    pub fn DefWindowProcW(hwnd:HWND,msg:u32,wparam:usize,lparam:isize)->isize;
    pub fn DestroyWindow(hwnd:HWND)->i32;
    pub fn PostQuitMessage(code:i32);
    pub fn GetMessageW(message:*mut Msg,hwnd:HWND,min:u32,max:u32)->i32;
    pub fn TranslateMessage(message:*const Msg)->i32;
    pub fn DispatchMessageW(message:*const Msg)->isize;
    pub fn ShowWindow(hwnd:HWND,command:i32)->i32;
    pub fn UpdateWindow(hwnd:HWND)->i32;
    pub fn SetWindowTextW(hwnd:HWND,title:*const u16)->i32;
    pub fn GetWindowTextW(hwnd:HWND,buf:*mut u16,size:i32)->i32;
    pub fn SendMessageW(hwnd:HWND,msg:u32,wparam:usize,lparam:isize)->isize;
    pub fn MessageBoxW(hwnd:HWND,text:*const u16,caption:*const u16,flags:u32)->i32;
    pub fn SetTimer(hwnd:HWND,id:usize,ms:u32,callback:Option<unsafe extern "system" fn(HWND,u32,usize,u32)>)->usize;
    pub fn KillTimer(hwnd:HWND,id:usize)->i32;
    pub fn RegisterRawInputDevices(devices:*const RawInputDevice,num:u32,size:u32)->i32;
    pub fn GetRawInputData(handle:HANDLE,command:u32,data:*mut c_void,size:*mut u32,header_size:u32)->u32;
    pub fn GetRawInputDeviceInfoW(device:HANDLE,command:u32,data:*mut c_void,size:*mut u32)->u32;
    pub fn GetRegisteredRawInputDevices(devices:*mut RawInputDevice,count:*mut u32,size:u32)->u32;
    pub fn LoadIconW(instance:HINSTANCE,name:*const u16)->HICON;
    pub fn GetSysColorBrush(index:i32)->isize;
    pub fn SendInput(count:u32,inputs:*const Input,size:i32)->u32;
}
#[link(name="kernel32")]
extern "system" {
    pub fn GetModuleHandleW(name:*const u16)->HINSTANCE;
    pub fn GetTickCount64()->u64;
    pub fn CreateMutexW(attributes:*const c_void,initial_owner:i32,name:*const u16)->HANDLE;
    pub fn GetLastError()->u32;
    pub fn CloseHandle(handle:HANDLE)->i32;
}
#[link(name="shell32")]
extern "system" { pub fn Shell_NotifyIconW(message:u32,data:*const NotifyIconDataW)->i32; }

#[repr(C)] #[derive(Clone, Copy)]
pub struct KeybdInput {pub vk:u16,pub scan:u16,pub flags:u32,pub time:u32,pub extra:usize}
#[repr(C)] #[derive(Clone, Copy)]
pub union InputUnion {pub keybd:KeybdInput,pub pad:[usize;4]}
#[repr(C)] #[derive(Clone, Copy)]
pub struct Input {pub kind:u32,pub data:InputUnion}

pub fn wide(s: &str)->Vec<u16> { std::ffi::OsStr::new(s).encode_wide().chain(Some(0)).collect() }
pub unsafe fn message_loop() {
    let mut msg = std::mem::MaybeUninit::<Msg>::zeroed();
    while GetMessageW(msg.as_mut_ptr(),0,0,0)>0 {
        let msg = msg.assume_init();TranslateMessage(&msg);DispatchMessageW(&msg);
    }
}
pub unsafe fn create_class(name:&[u16],proc:unsafe extern "system" fn(HWND,u32,usize,isize)->isize)->Result<HINSTANCE,String>{
    let instance=GetModuleHandleW(std::ptr::null());
    let c=WndClassW {style:0, wndproc:Some(proc),cls_extra:0,wnd_extra:0,instance,
        icon:LoadIconW(0,32512usize as *const u16), cursor:0,
        background:GetSysColorBrush(5),menu_name:std::ptr::null(),class_name:name.as_ptr()};
    if RegisterClassW(&c)==0 {return Err(format!("RegisterClassW failed: {}",GetLastError()));}
    Ok(instance)
}
