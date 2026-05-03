use std::ffi::c_void;
use std::sync::OnceLock;

use dashmap::DashMap;
use retour::{Function, GenericDetour};
use windows_sys::Win32::System::Memory::{PAGE_EXECUTE_READWRITE, VirtualProtect};

use super::hid_check;

// currently, all we are doing is unpatching Steams hook
// and replacing it with our own trampoline
// our own trampoline calls the ORIGINAL function (currently)
// this seems to be enough to get the "REAL" controllers via HIDAPI
// and steam emulated controllers via XINPUT
// (SteamInput doesn't support more than what an xbox360 controller has anyway...)

pub fn rehook(export_name: &str) {
    match export_name {
        "HidD_FreePreparsedData" => {
            unsafe fn addr_to_fn(addr: usize) -> HidDFreePreparsedDataFn {
                unsafe { std::mem::transmute(addr) }
            }
            rehook_common(
                "HidD_FreePreparsedData",
                hook_hidd_freepreparseddata as HidDFreePreparsedDataFn,
                addr_to_fn,
            );
        }
        "HidD_GetAttributes" => {
            unsafe fn addr_to_fn(addr: usize) -> HidDGetAttributesFn {
                unsafe { std::mem::transmute(addr) }
            }
            rehook_common(
                "HidD_GetAttributes",
                hook_hidd_getattributes as HidDGetAttributesFn,
                addr_to_fn,
            );
        }
        "HidD_GetPreparsedData" => {
            unsafe fn addr_to_fn(addr: usize) -> HidDGetPreparsedDataFn {
                unsafe { std::mem::transmute(addr) }
            }
            rehook_common(
                "HidD_GetPreparsedData",
                hook_hidd_getpreparseddata as HidDGetPreparsedDataFn,
                addr_to_fn,
            );
        }
        "HidD_GetProductString" => {
            unsafe fn addr_to_fn(addr: usize) -> HidDGetProductStringFn {
                unsafe { std::mem::transmute(addr) }
            }
            rehook_common(
                "HidD_GetProductString",
                hook_hidd_getproductstring as HidDGetProductStringFn,
                addr_to_fn,
            );
        }
        "HidP_GetButtonCaps" => {
            unsafe fn addr_to_fn(addr: usize) -> HidPGetButtonCapsFn {
                unsafe { std::mem::transmute(addr) }
            }
            rehook_common(
                "HidP_GetButtonCaps",
                hook_hidp_getbuttoncaps as HidPGetButtonCapsFn,
                addr_to_fn,
            );
        }
        "HidP_GetCaps" => {
            unsafe fn addr_to_fn(addr: usize) -> HidPGetCapsFn {
                unsafe { std::mem::transmute(addr) }
            }

            rehook_common(
                "HidP_GetCaps",
                hook_hidp_getcaps as HidPGetCapsFn,
                addr_to_fn,
            );
        }
        "HidP_GetData" => {
            unsafe fn addr_to_fn(addr: usize) -> HidPGetDataFn {
                unsafe { std::mem::transmute(addr) }
            }
            rehook_common(
                "HidP_GetData",
                hook_hidp_getdata as HidPGetDataFn,
                addr_to_fn,
            );
        }
        "HidP_GetUsageValue" => {
            unsafe fn addr_to_fn(addr: usize) -> HidPGetUsageValueFn {
                unsafe { std::mem::transmute(addr) }
            }
            rehook_common(
                "HidP_GetUsageValue",
                hook_hidp_getusagevalue as HidPGetUsageValueFn,
                addr_to_fn,
            );
        }
        "HidP_GetUsages" => {
            unsafe fn addr_to_fn(addr: usize) -> HidPGetUsagesFn {
                unsafe { std::mem::transmute(addr) }
            }
            rehook_common(
                "HidP_GetUsages",
                hook_hidp_getusages as HidPGetUsagesFn,
                addr_to_fn,
            );
        }
        "HidP_GetValueCaps" => {
            unsafe fn addr_to_fn(addr: usize) -> HidPGetValueCapsFn {
                unsafe { std::mem::transmute(addr) }
            }
            rehook_common(
                "HidP_GetValueCaps",
                hook_hidp_getvaluecaps as HidPGetValueCapsFn,
                addr_to_fn,
            );
        }
        "HidP_MaxDataListLength" => {
            unsafe fn addr_to_fn(addr: usize) -> HidPMaxDataListLengthFn {
                unsafe { std::mem::transmute(addr) }
            }
            rehook_common(
                "HidP_MaxDataListLength",
                hook_hidp_maxdatalistlength as HidPMaxDataListLengthFn,
                addr_to_fn,
            );
        }
        _ => {
            tracing::error!("rehook: unknown export name: {}", export_name);
        }
    }
}

pub fn unhook_all() {
    for target in [
        "HidD_FreePreparsedData",
        "HidD_GetAttributes",
        "HidD_GetPreparsedData",
        "HidD_GetProductString",
        "HidP_GetButtonCaps",
        "HidP_GetCaps",
        "HidP_GetData",
        "HidP_GetUsageValue",
        "HidP_GetUsages",
        "HidP_GetValueCaps",
        "HidP_MaxDataListLength",
    ] {
        unhook(target);
    }
}

pub fn unhook(export_name: &str) {
    match export_name {
        "HidD_FreePreparsedData" => uninstall_detour::<HidDFreePreparsedDataFn>(export_name),
        "HidD_GetAttributes" => uninstall_detour::<HidDGetAttributesFn>(export_name),
        "HidD_GetPreparsedData" => uninstall_detour::<HidDGetPreparsedDataFn>(export_name),
        "HidD_GetProductString" => uninstall_detour::<HidDGetProductStringFn>(export_name),
        "HidP_GetButtonCaps" => uninstall_detour::<HidPGetButtonCapsFn>(export_name),
        "HidP_GetCaps" => uninstall_detour::<HidPGetCapsFn>(export_name),
        "HidP_GetData" => uninstall_detour::<HidPGetDataFn>(export_name),
        "HidP_GetUsageValue" => uninstall_detour::<HidPGetUsageValueFn>(export_name),
        "HidP_GetUsages" => uninstall_detour::<HidPGetUsagesFn>(export_name),
        "HidP_GetValueCaps" => uninstall_detour::<HidPGetValueCapsFn>(export_name),
        "HidP_MaxDataListLength" => uninstall_detour::<HidPMaxDataListLengthFn>(export_name),
        _ => {
            tracing::error!("unhook: unknown export name: {}", export_name);
        }
    }
}

fn resolve_steam_jmp_target(entry: usize) -> Option<usize> {
    // Steam replaces the export entry with a JMP.
    unsafe {
        let bytes = std::slice::from_raw_parts(entry as *const u8, 16);

        // E9 rel32
        if bytes.len() >= 5 && bytes[0] == 0xE9 {
            let rel = i32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as isize;
            let target = (entry as isize + 5 + rel) as usize;
            return Some(target);
        }

        // FF 25 disp32  (jmp qword ptr [rip+disp32])
        if bytes.len() >= 6 && bytes[0] == 0xFF && bytes[1] == 0x25 {
            let disp = i32::from_le_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]) as isize;
            let ptr_loc = (entry as isize + 6 + disp) as usize;
            let target = *(ptr_loc as *const usize);
            return Some(target);
        }

        // 48 B8 imm64; FF E0  (mov rax, imm64; jmp rax)
        if bytes.len() >= 12
            && bytes[0] == 0x48
            && bytes[1] == 0xB8
            && bytes[10] == 0xFF
            && bytes[11] == 0xE0
        {
            let imm = u64::from_le_bytes([
                bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8], bytes[9],
            ]) as usize;
            return Some(imm);
        }
    }

    None
}

fn restore_baseline16(entry: usize, baseline: [u8; 16]) -> Result<(), &'static str> {
    let mut old: u32 = 0;
    let ok = unsafe {
        VirtualProtect(
            entry as *const c_void,
            baseline.len(),
            PAGE_EXECUTE_READWRITE,
            &mut old as *mut u32,
        )
    };
    if ok == 0 {
        return Err("VirtualProtect(PAGE_EXECUTE_READWRITE) failed");
    }

    unsafe {
        std::ptr::copy_nonoverlapping(baseline.as_ptr(), entry as *mut u8, baseline.len());
    }

    let mut _tmp: u32 = 0;
    let ok2 = unsafe { VirtualProtect(entry as *const c_void, baseline.len(), old, &mut _tmp) };
    if ok2 == 0 {
        return Err("VirtualProtect(restore) failed");
    }

    Ok(())
}

fn rehook_common<F>(target: &'static str, hook: F, addr_to_fn: unsafe fn(usize) -> F)
where
    F: Copy + Function,
{
    if detour_funcs().contains_key(target) {
        tracing::error!("{target}: detour already installed");
        return;
    }

    let Some(entry) = hid_check::get_hid_export_addr(target) else {
        tracing::error!("failed to locate hid export: {target}");
        return;
    };

    let Some(steam_target) = resolve_steam_jmp_target(entry) else {
        tracing::error!("{target} appears hooked but JMP target could not be decoded");
        return;
    };

    let Some(baseline) = hid_check::get_hid_export_baseline16(target) else {
        tracing::error!("no baseline bytes captured for {target}");
        return;
    };

    tracing::info!("{target}: entry=0x{entry:X} steam_target=0x{steam_target:X}");

    if let Err(e) = restore_baseline16(entry, baseline) {
        tracing::error!("{target}: failed to restore baseline bytes: {e}");
        return;
    }

    let original: F = unsafe { addr_to_fn(entry) };
    steam_funcs().insert(target, steam_target);

    let detour = match unsafe { GenericDetour::new(original, hook) } {
        Ok(d) => d,
        Err(e) => {
            tracing::error!("{target}: detour create failed: {e}");
            return;
        }
    };

    if let Err(e) = unsafe { detour.enable() } {
        tracing::error!("{target}: detour enable failed: {e}");
        return;
    }

    let detour_ptr = Box::into_raw(Box::new(detour)) as usize;
    detour_funcs().insert(target, detour_ptr);

    tracing::info!("{target}: installed");
}

fn uninstall_detour<F>(target: &str)
where
    F: Copy + Function,
{
    let Some(detour_ptr) = detour_funcs().get(target).map(|ptr| *ptr) else {
        tracing::trace!("{target}: no detour installed");
        steam_funcs().remove(target);
        return;
    };

    let detour = unsafe { &*(detour_ptr as *const GenericDetour<F>) };
    if let Err(e) = unsafe { detour.disable() } {
        tracing::error!("{target}: detour disable failed: {e}");
        return;
    }

    if let Some((_, detour_ptr)) = detour_funcs().remove(target) {
        unsafe {
            drop(Box::from_raw(detour_ptr as *mut GenericDetour<F>));
        }
    }
    steam_funcs().remove(target);

    tracing::info!("{target}: uninstalled");
}

type HidPGetCapsFn = extern "system" fn(preparsed_data: *const c_void, caps: *mut c_void) -> i32;

type HidDFreePreparsedDataFn = extern "system" fn(preparsed_data: *mut c_void) -> u8;

type HidDGetAttributesFn =
    extern "system" fn(device_object: *mut c_void, attributes: *mut c_void) -> u8;

type HidDGetPreparsedDataFn =
    extern "system" fn(device_object: *mut c_void, preparsed_data: *mut *mut c_void) -> u8;

type HidDGetProductStringFn =
    extern "system" fn(device_object: *mut c_void, buffer: *mut c_void, buffer_length: u32) -> u8;

type HidPGetButtonCapsFn = extern "system" fn(
    report_type: i32,
    button_caps: *mut c_void,
    button_caps_length: *mut u16,
    preparsed_data: *const c_void,
) -> i32;

type HidPGetDataFn = extern "system" fn(
    report_type: i32,
    data_list: *mut c_void,
    data_length: *mut u32,
    preparsed_data: *const c_void,
    report: *const u8,
    report_length: u32,
) -> i32;

type HidPGetUsageValueFn = extern "system" fn(
    report_type: i32,
    usage_page: u16,
    link_collection: u16,
    usage: u16,
    usage_value: *mut u32,
    preparsed_data: *const c_void,
    report: *const u8,
    report_length: u32,
) -> i32;

type HidPGetUsagesFn = extern "system" fn(
    report_type: i32,
    usage_page: u16,
    link_collection: u16,
    usage_list: *mut u16,
    usage_length: *mut u32,
    preparsed_data: *const c_void,
    report: *const u8,
    report_length: u32,
) -> i32;

type HidPGetValueCapsFn = extern "system" fn(
    report_type: i32,
    value_caps: *mut c_void,
    value_caps_length: *mut u16,
    preparsed_data: *const c_void,
) -> i32;

type HidPMaxDataListLengthFn =
    extern "system" fn(report_type: i32, preparsed_data: *const c_void) -> u32;

static STEAM_FUNCS: OnceLock<DashMap<&'static str, usize>> = OnceLock::new();
static DETOUR_FUNCS: OnceLock<DashMap<&'static str, usize>> = OnceLock::new();

fn steam_funcs() -> &'static DashMap<&'static str, usize> {
    STEAM_FUNCS.get_or_init(DashMap::new)
}

fn detour_funcs() -> &'static DashMap<&'static str, usize> {
    DETOUR_FUNCS.get_or_init(DashMap::new)
}

extern "system" fn hook_hidp_getcaps(preparsed_data: *const c_void, caps: *mut c_void) -> i32 {
    let ret = unsafe {
        let detour_ptr = *detour_funcs().get("HidP_GetCaps").unwrap();
        (&*(detour_ptr as *const GenericDetour<HidPGetCapsFn>)).call(preparsed_data, caps)
    };
    // let ret = steam(preparsed_data, caps);

    // tracing::trace!(
    //     "hook_hidp_getcaps called: preparsed_data={:?} caps={:?} -> {}",
    //     preparsed_data,
    //     caps,
    //     ret
    // );

    ret
}

extern "system" fn hook_hidd_freepreparseddata(preparsed_data: *mut c_void) -> u8 {
    let ret = unsafe {
        let detour_ptr = *detour_funcs().get("HidD_FreePreparsedData").unwrap();
        (&*(detour_ptr as *const GenericDetour<HidDFreePreparsedDataFn>)).call(preparsed_data)
    };
    // let ret = steam(preparsed_data);
    ret
}

extern "system" fn hook_hidd_getattributes(
    device_object: *mut c_void,
    attributes: *mut c_void,
) -> u8 {
    let ret = unsafe {
        let detour_ptr = *detour_funcs().get("HidD_GetAttributes").unwrap();
        (&*(detour_ptr as *const GenericDetour<HidDGetAttributesFn>))
            .call(device_object, attributes)
    };
    // let ret = steam(device_object, attributes);
    ret
}

extern "system" fn hook_hidd_getpreparseddata(
    device_object: *mut c_void,
    preparsed_data: *mut *mut c_void,
) -> u8 {
    let ret = unsafe {
        let detour_ptr = *detour_funcs().get("HidD_GetPreparsedData").unwrap();
        (&*(detour_ptr as *const GenericDetour<HidDGetPreparsedDataFn>))
            .call(device_object, preparsed_data)
    };
    // let ret = steam(device_object, preparsed_data);
    ret
}

extern "system" fn hook_hidd_getproductstring(
    device_object: *mut c_void,
    buffer: *mut c_void,
    buffer_length: u32,
) -> u8 {
    let ret = unsafe {
        let detour_ptr = *detour_funcs().get("HidD_GetProductString").unwrap();
        (&*(detour_ptr as *const GenericDetour<HidDGetProductStringFn>)).call(
            device_object,
            buffer,
            buffer_length,
        )
    };
    // let ret = steam(device_object, buffer, buffer_length);
    ret
}

extern "system" fn hook_hidp_getbuttoncaps(
    report_type: i32,
    button_caps: *mut c_void,
    button_caps_length: *mut u16,
    preparsed_data: *const c_void,
) -> i32 {
    let ret = unsafe {
        let detour_ptr = *detour_funcs().get("HidP_GetButtonCaps").unwrap();
        (&*(detour_ptr as *const GenericDetour<HidPGetButtonCapsFn>)).call(
            report_type,
            button_caps,
            button_caps_length,
            preparsed_data,
        )
    };
    // let ret = steam(report_type, button_caps, button_caps_length, preparsed_data);
    ret
}

extern "system" fn hook_hidp_getdata(
    report_type: i32,
    data_list: *mut c_void,
    data_length: *mut u32,
    preparsed_data: *const c_void,
    report: *const u8,
    report_length: u32,
) -> i32 {
    let ret = unsafe {
        let detour_ptr = *detour_funcs().get("HidP_GetData").unwrap();
        (&*(detour_ptr as *const GenericDetour<HidPGetDataFn>)).call(
            report_type,
            data_list,
            data_length,
            preparsed_data,
            report,
            report_length,
        )
    };
    // let ret = steam(report_type, data_list, data_length, preparsed_data, report, report_length);
    ret
}

extern "system" fn hook_hidp_getusagevalue(
    report_type: i32,
    usage_page: u16,
    link_collection: u16,
    usage: u16,
    usage_value: *mut u32,
    preparsed_data: *const c_void,
    report: *const u8,
    report_length: u32,
) -> i32 {
    let ret = unsafe {
        let detour_ptr = *detour_funcs().get("HidP_GetUsageValue").unwrap();
        (&*(detour_ptr as *const GenericDetour<HidPGetUsageValueFn>)).call(
            report_type,
            usage_page,
            link_collection,
            usage,
            usage_value,
            preparsed_data,
            report,
            report_length,
        )
    };
    // let ret = steam(report_type, usage_page, link_collection, usage, usage_value, preparsed_data, report, report_length);
    ret
}

extern "system" fn hook_hidp_getusages(
    report_type: i32,
    usage_page: u16,
    link_collection: u16,
    usage_list: *mut u16,
    usage_length: *mut u32,
    preparsed_data: *const c_void,
    report: *const u8,
    report_length: u32,
) -> i32 {
    let ret = unsafe {
        let detour_ptr = *detour_funcs().get("HidP_GetUsages").unwrap();
        (&*(detour_ptr as *const GenericDetour<HidPGetUsagesFn>)).call(
            report_type,
            usage_page,
            link_collection,
            usage_list,
            usage_length,
            preparsed_data,
            report,
            report_length,
        )
    };
    // let ret = steam(report_type, usage_page, link_collection, usage_list, usage_length, preparsed_data, report, report_length);
    ret
}

extern "system" fn hook_hidp_getvaluecaps(
    report_type: i32,
    value_caps: *mut c_void,
    value_caps_length: *mut u16,
    preparsed_data: *const c_void,
) -> i32 {
    let ret = unsafe {
        let detour_ptr = *detour_funcs().get("HidP_GetValueCaps").unwrap();
        (&*(detour_ptr as *const GenericDetour<HidPGetValueCapsFn>)).call(
            report_type,
            value_caps,
            value_caps_length,
            preparsed_data,
        )
    };
    // let ret = steam(report_type, value_caps, value_caps_length, preparsed_data);
    ret
}

extern "system" fn hook_hidp_maxdatalistlength(
    report_type: i32,
    preparsed_data: *const c_void,
) -> u32 {
    let ret = unsafe {
        let detour_ptr = *detour_funcs().get("HidP_MaxDataListLength").unwrap();
        (&*(detour_ptr as *const GenericDetour<HidPMaxDataListLengthFn>))
            .call(report_type, preparsed_data)
    };
    // let ret = steam(report_type, preparsed_data);
    ret
}
