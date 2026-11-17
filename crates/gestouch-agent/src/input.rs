//! RawInput PTP HID decoder. Coordinates are HID logical values normalized
//! to 0..10000 using value capabilities; no descriptor hardcoding.
use gestouch_core::gesture::{Contact, Frame};
use gestouch_win::*;
use std::{collections::HashMap, ffi::c_void, mem::size_of};
const HIDP_SUCCESS:i32=0x00110000;
const HIDP_INPUT:i32=0;
const PAGE_DIG:u16=0x0d;
const PAGE_GENERIC:u16=0x01;
const USAGE_CONTACT_COUNT:u16=0x54;
const USAGE_CONTACT_ID:u16=0x51;
const USAGE_TIP:u16=0x42;
const USAGE_X:u16=0x30;
const USAGE_Y:u16=0x31;

#[repr(C)] #[derive(Clone,Copy,Default)]
struct HidpCaps {
    usage:u16,usage_page:u16,input_report_byte_length:u16,
    output_report_byte_length:u16,feature_report_byte_length:u16,reserved:[u16;17],
    number_link_collection_nodes:u16,number_input_button_caps:u16,
    number_input_value_caps:u16,number_input_data_indices:u16,
    number_output_button_caps:u16,number_output_value_caps:u16,
    number_output_data_indices:u16,number_feature_button_caps:u16,
    number_feature_value_caps:u16,number_feature_data_indices:u16,
}
#[repr(C)] #[derive(Clone,Copy,Default)]
struct HidpValueCaps {
    usage_page:u16,report_id:u8,is_alias:u8,bit_field:u16,link_collection:u16,
    link_usage:u16,link_usage_page:u16,is_range:u8,is_string_range:u8,
    is_designator_range:u8,is_absolute:u8,has_null:u8,reserved:u8,
    bit_size:u16,report_count:u16,reserved2:[u16;5],
    units_exp:u32,units:u32,logical_min:i32,logical_max:i32,physical_min:i32,physical_max:i32,
    usage_union:[u16;8],
}
#[link(name="hid")]
extern "system" {
    fn HidP_GetCaps(preparsed:*const c_void,caps:*mut HidpCaps)->i32;
    fn HidP_GetValueCaps(report_type:i32,caps:*mut HidpValueCaps,count:*mut u16,preparsed:*const c_void)->i32;
    fn HidP_GetUsageValue(report_type:i32,page:u16,link:u16,usage:u16,value:*mut u32,
        preparsed:*const c_void,report:*const u8,length:u32)->i32;
    fn HidP_GetUsages(report_type:i32,page:u16,link:u16,usages:*mut u16,count:*mut u32,
        preparsed:*const c_void,report:*const u8,length:u32)->i32;
}
#[derive(Clone,Copy)]
struct Axis {min:i32,max:i32}
impl Axis {
    fn normalize(self,value:i32)->i32 {
        let range=i64::from(self.max)-i64::from(self.min);
        if range<=0 {return value.clamp(0,10000);}
        (((i64::from(value)-i64::from(self.min))*10000)/range).clamp(0,10000) as i32
    }
}
#[derive(Default)]
struct PartialFrame {expected:usize,contacts:HashMap<u32,Contact>,since:u64}
#[derive(Default)]
pub struct RawReader {pending:HashMap<isize,PartialFrame>,pub raw_messages:u64,pub completed_frames:u64,
    pub errors:u64,pub last_raw_ms:u64}
impl RawReader {
    pub fn reset(&mut self) {self.pending.clear();}
    pub fn ingest(&mut self, handle:isize, now:u64)->Vec<Frame> {
        let mut results=Vec::new();
        unsafe {
            let mut length=0u32;
            let header_size=size_of::<RawInputHeader>() as u32;
            let query=GetRawInputData(handle,RID_INPUT,std::ptr::null_mut(),&mut length,header_size);
            if query==u32::MAX || length < header_size+8 || length>65536 {self.errors+=1;return results;}
            let mut data=vec![0usize; (length as usize+size_of::<usize>()-1)/size_of::<usize>()];
            let result=GetRawInputData(handle,RID_INPUT,data.as_mut_ptr().cast(),&mut length,header_size);
            if result==u32::MAX || result!=length {self.errors+=1;return results;}
            let base=data.as_ptr() as *const u8;
            let header=&*(base as *const RawInputHeader);
            if header.data_type!=2 {return results;}
            let offset=header_size as usize;
            let hid_size=*(base.add(offset) as *const u32) as usize;
            let report_count=*(base.add(offset+4) as *const u32) as usize;
            if hid_size==0 || report_count==0 || report_count>64 ||
                hid_size.checked_mul(report_count).is_none_or(|n|n+offset+8>length as usize) {self.errors+=1;return results;}
            self.raw_messages+=1;self.last_raw_ms=now;
            let device=header.device;
            let Some((preparsed_storage,caps,value_caps))=Self::device_caps(device) else {self.errors+=1;return results;};
            let pre=preparsed_storage.as_ptr().cast::<c_void>();
            for i in 0..report_count {
                let report=std::slice::from_raw_parts(base.add(offset+8+i*hid_size),hid_size);
                let count=Self::value(pre,report,PAGE_DIG,0,USAGE_CONTACT_COUNT).unwrap_or(0) as usize;
                let mut contacts=Vec::new();
                for link in 1..=caps.number_link_collection_nodes.min(64) {
                    let Some(id)=Self::value(pre,report,PAGE_DIG,link,USAGE_CONTACT_ID) else {continue};
                    let Some(x)=Self::value(pre,report,PAGE_GENERIC,link,USAGE_X) else {continue};
                    let Some(y)=Self::value(pre,report,PAGE_GENERIC,link,USAGE_Y) else {continue};
                    if let Some(false)=Self::tip_active(pre,report,link) {continue;}
                    let x_axis=Self::axis(&value_caps,link,USAGE_X);
                    let y_axis=Self::axis(&value_caps,link,USAGE_Y);
                    contacts.push(Contact {id,x:x_axis.normalize(x as i32),y:y_axis.normalize(y as i32)});
                }
                if count>5 {self.errors+=1;self.pending.remove(&device);continue;}
                if count==0 && !self.pending.contains_key(&device) {
                    // A no-contact report completes the previous contact sequence.
                    if contacts.is_empty() {
                        results.push(Frame{device:device as usize,timestamp_ms:now,contacts:vec![],complete:true});
                        self.completed_frames+=1;
                    }
                    continue;
                }
                if count>0 {
                    self.pending.insert(device,PartialFrame{expected:count,contacts:HashMap::new(),since:now});
                }
                let Some(pending)=self.pending.get_mut(&device) else {continue};
                if now.saturating_sub(pending.since)>500 {self.pending.remove(&device);self.errors+=1;continue;}
                for c in contacts {pending.contacts.insert(c.id,c);}
                if pending.contacts.len()>=pending.expected {
                    let p=self.pending.remove(&device).unwrap();
                    let mut points:Vec<Contact>=p.contacts.into_values().collect();points.sort_by_key(|c|c.id);
                    results.push(Frame{device:device as usize,timestamp_ms:now,contacts:points,complete:true});
                    self.completed_frames+=1;
                }
            }
        }
        results
    }
    unsafe fn device_caps(device:isize)->Option<(Vec<usize>,HidpCaps,Vec<HidpValueCaps>)> {
        let mut length=0u32;
        let q=GetRawInputDeviceInfoW(device,RIDI_PREPARSEDDATA,std::ptr::null_mut(),&mut length);
        if q==u32::MAX || length==0 || length>65536 {return None;}
        let mut storage=vec![0usize;(length as usize+size_of::<usize>()-1)/size_of::<usize>()];
        let result=GetRawInputDeviceInfoW(device,RIDI_PREPARSEDDATA,storage.as_mut_ptr().cast(),&mut length);
        if result==u32::MAX {return None;}
        let mut caps=HidpCaps::default();let pre=storage.as_ptr().cast::<c_void>();
        if HidP_GetCaps(pre,&mut caps)!=HIDP_SUCCESS || caps.number_input_value_caps>1024 {return None;}
        let mut values=vec![HidpValueCaps::default();caps.number_input_value_caps as usize];
        let mut n=caps.number_input_value_caps;
        if n>0 && HidP_GetValueCaps(HIDP_INPUT,values.as_mut_ptr(),&mut n,pre)!=HIDP_SUCCESS {return None;}
        values.truncate(n as usize);
        Some((storage,caps,values))
    }
    unsafe fn value(pre:*const c_void, report:&[u8],page:u16,link:u16,usage:u16)->Option<u32>{
        let mut value=0u32;
        if HidP_GetUsageValue(HIDP_INPUT,page,link,usage,&mut value,pre,report.as_ptr(),report.len() as u32)==HIDP_SUCCESS {Some(value)}else{None}
    }
    unsafe fn tip_active(pre:*const c_void,report:&[u8],link:u16)->Option<bool>{
        let mut usages=[0u16;32];let mut len=usages.len() as u32;
        if HidP_GetUsages(HIDP_INPUT,PAGE_DIG,link,usages.as_mut_ptr(),&mut len,pre,report.as_ptr(),report.len() as u32)!=HIDP_SUCCESS {return None;}
        Some(usages[..(len as usize).min(32)].contains(&USAGE_TIP))
    }
    fn axis(values:&[HidpValueCaps],link:u16,usage:u16)->Axis {
        for c in values {
            if c.usage_page!=PAGE_GENERIC || c.link_collection!=link {continue;}
            let (min,max)=if c.is_range!=0 {(c.usage_union[0],c.usage_union[1])}else{(c.usage_union[0],c.usage_union[0])};
            if (min..=max).contains(&usage) {return Axis{min:c.logical_min,max:c.logical_max};}
        }
        Axis{min:0,max:10000}
    }
}
