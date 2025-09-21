extern crate hyprland;
extern crate phf;

use std::ffi::{CStr, CString};

use std::os::raw::c_char;

use hyprland::ctl::switch_xkb_layout;
use hyprland::data::{Devices, Keyboard};
use hyprland::shared::HyprData;
use hyprland::Result as HResult;
use phf::{phf_map};

#[derive(Debug)]
enum Error {
    InconsistentLayouts,
    NoKeyboards,
    CannotDetermineLayoutForActiveKeymap
}

#[no_mangle]
pub extern "C" fn Xkb_Switch_getXkbLayout() -> *const c_char {
    let layout = get_cur_layout().unwrap();
    CString::new(layout).unwrap().into_raw()
}

// HACK 
static KEYBOARD_KEYMAP_TO_LAYOUT_MAPPING: phf::Map<&'static str, &'static str> = phf_map! {
    "Russian" => "ru",
    "English (US)" => "us",
};

fn get_cur_layout() -> Result<String, Error> {
    let mut keymaps: Vec<String> = get_keyboards()
        .unwrap() //TODO nicer
        .into_iter()
        .map(|kb| kb.active_keymap)
        .collect();
    keymaps.dedup();
    match keymaps.leak() {
        [] => Err(Error::NoKeyboards),
        [keymap] => KEYBOARD_KEYMAP_TO_LAYOUT_MAPPING.get(keymap).ok_or(Error::CannotDetermineLayoutForActiveKeymap).map(|layout| layout.to_string()),
        _ => Err(Error::InconsistentLayouts),
    }
}

fn get_keyboards() -> HResult<Vec<Keyboard>>{
    let devices = Devices::get()?;
    let all_inputs = devices.keyboards;
    HResult::Ok(all_inputs)
}

#[no_mangle]
pub extern "C" fn Xkb_Switch_setXkbLayout(layout_ptr: *const c_char) {
            let layout = unsafe { CStr::from_ptr(layout_ptr).to_string_lossy().to_string() };
            switch_layout(&layout);
}

fn switch_layout(layout: &String) {
    get_keyboards().unwrap().iter().for_each(|kb| {
        let layout_index = kb
            .layout
            .split(", ")
            .position(|x| x == layout)
            .unwrap();

        switch_xkb_layout::call(kb.name.to_string(), switch_xkb_layout::SwitchXKBLayoutCmdTypes::Id(layout_index as u8)).unwrap();
    });
}
