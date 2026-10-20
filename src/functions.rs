use std::collections::BTreeSet;
use hidapi::{HidApi, HidDevice};

pub fn load_embedded_icon() -> Result<crate::egui::IconData, String> {
    let img = image::load_from_memory(crate::ICON_BYTES).map_err(|e| e.to_string())?.into_rgba8();
    let (width, height) = img.dimensions();
    let rgba = img.into_raw();
    Ok(crate::egui::IconData { rgba, width, height })
}

pub fn enable_per_key_lighting(device: &HidDevice) -> Result<(), String> {
    let mut select = vendor_report(0x17);
    select[5] = 0x02;
    device
        .write(&select)
        .map_err(|error| format!("Failed to select user profile: {error}"))?;
    let mut lighting = vendor_report(0x07);
    lighting[5] = 14;
    lighting[6] = 10;
    lighting[7] = 4;
    lighting[8] = 4;
    lighting[17] = 0;
    device
        .write(&lighting)
        .map_err(|error| format!("Failed to enable per-key mode: {error}"))?;
    Ok(())
}

pub fn write_rgb_slot(device: &HidDevice, slot: u8, table: &[u8]) -> Result<(), String> {
    if slot > 1 || table.len() != crate::RGB_TABLE_LEN {
        return Err("Invalid RGB slot or table size".to_string());
    }

    for (page, chunk) in table.chunks(crate::RGB_PAGE_LEN).enumerate() {
        let mut report = vendor_report(9);
        report[2] = slot;
        report[3] = ((page >> 8) & 0xff) as u8;
        report[4] = (page & 0xff) as u8;
        report[5] = chunk.len() as u8;
        report[6..6 + chunk.len()].copy_from_slice(chunk);
        let written = device
            .write(&report)
            .map_err(|error| format!("Failed to write RGB page {page}: {error}"))?;
        let len = crate::REPORT_LEN;
        if written != len {
            return Err(format!(
                "Device accepted {written}/{len} bytes of page {page}"
            ));
        }
    }
    Ok(())
}

pub fn open_keyboard() -> Result<HidDevice, String> {
    let api = HidApi::new().map_err(|error| format!("HID API: {error}"))?;
    let info = api.device_list().find(|info| {
            info.vendor_id() == crate::DEVICE_VID
                && info.product_id() == crate::DEVICE_PID
                && info.interface_number() == crate::VENDOR_INTERFACE
        })
        .ok_or_else(|| "K686BG-RGB-M vendor not found".to_string())?;
    api.open_path(info.path())
        .map_err(|error| format!("Failed to open HID interface: {error}"))
}

fn vendor_report(command: u8) -> [u8; crate::REPORT_LEN] {
    let mut report = [0; crate::REPORT_LEN];
    report[0] = crate::REPORT_ID;
    report[1] = command;
    report
}

pub fn read_rgb_slot(device: &HidDevice, slot: u8) -> Result<Vec<u8>, String> {
    if slot > 1 {
        return Err("RGB slot number must be 0 or 1".to_string());
    }

    let mut request = vendor_report(9);
    request[2] = 0x80 | slot;
    device
        .write(&request)
        .map_err(|error| format!("RGB slot {slot} request error: {error}"))?;

    let mut table = vec![0; crate::RGB_TABLE_LEN];
    let mut received_pages = [false; 8];
    let mut report = [0; crate::REPORT_LEN];
    let mut idle_reads = 0;
    while received_pages.iter().any(|received| !received) && idle_reads < 20 {
        let length = device
            .read_timeout(&mut report, 100)
            .map_err(|error| format!("RGB slot {slot} read error: {error}"))?;
        if length == 0 {
            idle_reads += 1;
            continue;
        }
        idle_reads = 0;
        if length < 6 || report[0] != crate::REPORT_ID || report[1] != 9 || report[2] != (0x80 | slot) {
            continue;
        }

        let page = u16::from_be_bytes([report[3], report[4]]) as usize;
        let data_len = report[5] as usize;
        if page >= received_pages.len() || data_len == 0 || 6 + data_len > length {
            continue;
        }
        let offset = page * crate::RGB_PAGE_LEN;
        let copy_len = data_len.min(crate::RGB_TABLE_LEN.saturating_sub(offset));
        if copy_len == 0 {
            continue;
        }
        table[offset..offset + copy_len].copy_from_slice(&report[6..6 + copy_len]);
        received_pages[page] = true;
    }

    if received_pages.iter().all(|received| *received) {
        Ok(table)
    } else {
        let pages = received_pages.iter().filter(|received| **received).count();
        Err(format!(
            "Incomplete RGB slot {slot} read: received {pages}/8 pages"
        ))
    }
}

pub fn load_keyboard_slots() -> Result<[Vec<u8>; 2], String> {
    let device = open_keyboard()?;
    let slot_0 = read_rgb_slot(&device, 0)?;
    let slot_1 = read_rgb_slot(&device, 1)?;
    Ok([slot_0, slot_1])
}

pub fn display_key_label(name: &str) -> &str {
    match name {
        "▲" => "⬆",
        "▼" => "⬇",
        "◀" => "⬅",
        "▶" => "⮕",
        _ => name,
    }
}

pub fn update_key_selection(selected: &mut BTreeSet<String>, key_name: &str, ctrl_held: bool) {
    if ctrl_held {
        if selected.contains(key_name) {
            if selected.len() > 1 {
                selected.remove(key_name);
            }
        } else {
            selected.insert(key_name.to_string());
        }
    } else {
        selected.clear();
        selected.insert(key_name.to_string());
    }
}
