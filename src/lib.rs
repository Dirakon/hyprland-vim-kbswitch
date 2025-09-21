extern crate hyprland;
extern crate phf;
extern crate getopts;

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::process;

use hyprland::ctl::switch_xkb_layout;
use hyprland::data::{Devices, Keyboard};
use hyprland::shared::HyprData;
use hyprland::Result as HResult;
use phf::{phf_map};
use getopts::Options;

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

// New CLI functionality
fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [OPTIONS]", program);
    print!("{}", opts.usage(&brief));
}

#[allow(dead_code)]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let program = args[0].clone();
    
    let mut opts = Options::new();
    opts.optflag("g", "get", "Get current XKB layout");
    opts.optopt("s", "set", "Set XKB layout to LAYOUT", "LAYOUT");
    opts.optflag("h", "help", "Print this help menu");
    
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            eprintln!("Error: {}", f);
            process::exit(1);
        }
    };
    
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    
    if matches.opt_present("g") {
        match get_cur_layout() {
            Ok(layout) => println!("{}", layout),
            Err(e) => {
                eprintln!("Error getting layout: {:?}", e);
                process::exit(1);
            }
        }
    } else if let Some(layout) = matches.opt_str("s") {
        let c_layout = CString::new(layout.clone()).unwrap();
        Xkb_Switch_setXkbLayout(c_layout.as_ptr());
        println!("Layout set to: {}", layout);
    } else {
        eprintln!("Error: Either --get or --set option must be specified");
        print_usage(&program, opts);
        process::exit(1);
    }
}
