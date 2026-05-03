use std::collections::HashMap;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

use windows_sys::Win32::System::{
    Diagnostics::Debug::IMAGE_NT_HEADERS64,
    LibraryLoader::{GetModuleFileNameW, LoadLibraryA},
    SystemServices::{IMAGE_DOS_HEADER, IMAGE_EXPORT_DIRECTORY},
};

pub static EXPORTS_BASELINE: std::sync::OnceLock<std::collections::HashMap<String, [u8; 16]>> =
    std::sync::OnceLock::new();

fn hid_dll_path() -> Option<std::path::PathBuf> {
    unsafe {
        let hmod = LoadLibraryA(c"hid.dll".as_ptr().cast());
        if hmod.is_null() {
            tracing::error!("Failed to load hid.dll");
            return None;
        }

        let mut buf: Vec<u16> = vec![0; 32768];
        let len = GetModuleFileNameW(hmod, buf.as_mut_ptr(), buf.len() as u32) as usize;
        if len == 0 {
            tracing::error!("Failed to get hid.dll module file name");
            return None;
        }
        buf.truncate(len);
        Some(std::path::PathBuf::from(OsString::from_wide(&buf)))
    }
}

#[inline]
fn read_u16_le(buf: &[u8], off: usize) -> Option<u16> {
    let b = buf.get(off..off + 2)?;
    Some(u16::from_le_bytes([b[0], b[1]]))
}

#[inline]
fn read_u32_le(buf: &[u8], off: usize) -> Option<u32> {
    let b = buf.get(off..off + 4)?;
    Some(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

#[inline]
fn cstr_at(buf: &[u8], off: usize) -> Option<String> {
    let slice = buf.get(off..)?;
    let nul = slice.iter().position(|&b| b == 0)?;
    Some(String::from_utf8_lossy(&slice[..nul]).to_string())
}

fn rva_to_file_off(rva: u32, sections: &[(u32, u32, u32, u32)]) -> Option<usize> {
    for &(va, vs, raw_ptr, raw_size) in sections {
        let span = vs.max(raw_size);
        if rva >= va && rva < va.saturating_add(span) {
            let delta = rva - va;
            return Some((raw_ptr + delta) as usize);
        }
    }
    None
}

fn enumerate_hid_exports_from_disk() -> Option<HashMap<String, [u8; 16]>> {
    let path = hid_dll_path()?;
    let bytes = std::fs::read(&path).ok()?;
    let e_lfanew = read_u32_le(&bytes, 0x3c)? as usize;

    let file_header_off = e_lfanew + 4;
    let number_of_sections = read_u16_le(&bytes, file_header_off + 2)? as usize;
    let size_of_optional_header = read_u16_le(&bytes, file_header_off + 16)? as usize;

    let off = file_header_off + 20;

    let export_rva = read_u32_le(&bytes, off + 0x70)?;
    if export_rva == 0 {
        tracing::error!("No export directory found in hid.dll");
        return None;
    }

    let section_off = off + size_of_optional_header;
    let mut sections: Vec<(u32, u32, u32, u32)> = Vec::with_capacity(number_of_sections);
    for i in 0..number_of_sections {
        let off = section_off + i * 40;
        let virtual_size = read_u32_le(&bytes, off + 8)?;
        let virtual_address = read_u32_le(&bytes, off + 12)?;
        let size_of_raw = read_u32_le(&bytes, off + 16)?;
        let ptr_raw = read_u32_le(&bytes, off + 20)?;
        sections.push((virtual_address, virtual_size, ptr_raw, size_of_raw));
    }

    let export_dir_off = rva_to_file_off(export_rva, &sections)?;
    if export_dir_off + std::mem::size_of::<IMAGE_EXPORT_DIRECTORY>() > bytes.len() {
        return None;
    }

    let number_of_names = read_u32_le(&bytes, export_dir_off + 24)? as usize;
    let address_of_functions = read_u32_le(&bytes, export_dir_off + 28)?;
    let address_of_names = read_u32_le(&bytes, export_dir_off + 32)?;

    let names_off = rva_to_file_off(address_of_names, &sections)?;
    let funcs_off = rva_to_file_off(address_of_functions, &sections)?;

    let mut exports_map: HashMap<String, [u8; 16]> = HashMap::new();
    for i in 0..number_of_names {
        let name_rva = read_u32_le(&bytes, names_off + i * 4)?;
        let name_off = rva_to_file_off(name_rva, &sections)?;
        let name = cstr_at(&bytes, name_off)?;

        let func_rva = read_u32_le(&bytes, funcs_off + i * 4)?;
        if func_rva == 0 {
            continue;
        }
        let func_off = match rva_to_file_off(func_rva, &sections) {
            Some(v) => v,
            None => continue,
        };
        let first_bytes = match bytes.get(func_off..func_off + 16) {
            Some(b) => b,
            None => continue,
        };

        exports_map.insert(name, first_bytes.try_into().ok()?);
    }

    Some(exports_map)
}

#[allow(unsafe_code, unsafe_op_in_unsafe_fn)]
unsafe fn get_hid_export_table() -> Option<(*const u32, *const u32, usize, usize)> {
    let hid_base = LoadLibraryA(c"hid.dll".as_ptr().cast()) as usize;
    if hid_base == 0 {
        return None;
    }

    let dos = &*(hid_base as *const IMAGE_DOS_HEADER);
    let nt = (hid_base + dos.e_lfanew as usize) as *const IMAGE_NT_HEADERS64;
    let exp_rva = (*nt).OptionalHeader.DataDirectory[0].VirtualAddress;
    if exp_rva == 0 {
        return None;
    }
    let exp = (hid_base + exp_rva as usize) as *const IMAGE_EXPORT_DIRECTORY;

    Some((
        (hid_base + (*exp).AddressOfNames as usize) as *const u32,
        (hid_base + (*exp).AddressOfFunctions as usize) as *const u32,
        (*exp).NumberOfNames as usize,
        hid_base,
    ))
}

pub fn enumerate_hid_exports() {
    EXPORTS_BASELINE.get_or_init(|| {
        enumerate_hid_exports_from_disk().unwrap_or_else(|| {
            tracing::error!("Failed to enumerate hid.dll exports baseline from disk");
            HashMap::new()
        })
    });
}

pub fn detect_hid_hooks() -> Vec<String> {
    let Some(baseline) = EXPORTS_BASELINE.get() else {
        return Vec::new();
    };
    let mut hooked = Vec::new();

    unsafe {
        let Some((names, funcs, count, hid_base)) = get_hid_export_table() else {
            return baseline.keys().cloned().collect();
        };

        for i in 0..count {
            let name_rva = *names.add(i) as usize;
            let name_ptr = (hid_base + name_rva) as *const i8;
            let Ok(name) = std::ffi::CStr::from_ptr(name_ptr).to_str() else {
                continue;
            };
            let func_rva = *funcs.add(i) as usize;
            if func_rva == 0 {
                continue;
            }

            let entry = hid_base + func_rva;
            let mut cur = [0u8; 16];
            cur.copy_from_slice(std::slice::from_raw_parts(entry as *const u8, 16));

            if let Some(base_bytes) = baseline.get(name)
                && &cur != base_bytes
            {
                hooked.push(name.to_string());
            }
        }
    }

    hooked
}

pub fn get_hid_export_addr(export_name: &str) -> Option<usize> {
    unsafe {
        let (names, funcs, count, hid_base) = get_hid_export_table()?;

        for i in 0..count {
            let name_rva = *names.add(i) as usize;
            let name_ptr = (hid_base + name_rva) as *const i8;
            let Ok(name) = std::ffi::CStr::from_ptr(name_ptr).to_str() else {
                continue;
            };
            if name != export_name {
                continue;
            }

            let func_rva = *funcs.add(i) as usize;
            if func_rva == 0 {
                return None;
            }
            return Some(hid_base + func_rva);
        }
    }

    None
}

pub fn get_hid_export_baseline16(export_name: &str) -> Option<[u8; 16]> {
    EXPORTS_BASELINE
        .get()
        .and_then(|m| m.get(export_name).copied())
}
