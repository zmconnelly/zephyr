#![allow(unused)]

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HC_ACTION, HHOOK,
    KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP,
};

use crate::key_chord_parser::KeyChordParser;

type Callback = Arc<dyn Fn() -> bool + Send + Sync + 'static>;

struct HookData {
    callback: Callback,
    vk_codes: Vec<i32>,
    interval: Duration,
    last_trigger: Arc<RwLock<Instant>>,
    hook_handle: Arc<RwLock<Option<isize>>>,
}

lazy_static! {
    static ref HOOKS: Arc<RwLock<Vec<HookData>>> = Arc::new(RwLock::new(Vec::new()));
    static ref GLOBAL_KEY_STATES: Arc<RwLock<HashMap<i32, bool>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

fn handle_chord_pressed(hook: &HookData) -> bool {
    GLOBAL_KEY_STATES.write().unwrap().clear();

    // Check if we should trigger based on the interval
    let should_trigger = {
        let mut last_trigger = hook.last_trigger.write().unwrap();
        let now = Instant::now();
        let should_fire = now.duration_since(*last_trigger) >= hook.interval;
        if should_fire {
            *last_trigger = now;
        }
        should_fire
    };

    if should_trigger {
        println!("TRIGGERING CALLBACK");
        let callback = Arc::clone(&hook.callback);
        let callback_result = callback();
        if callback_result {
            println!("Stopping propagation");
            return true;
        } else {
            println!("Allowing propagation");
            return false;
        }
    } else {
        println!("Skipping callback due to interval");
    }

    return false;
}

fn normalize_keys(keys: Vec<i32>) -> Vec<i32> {
    keys.iter()
        .map(|key| match key {
            160 | 161 => 16, // Shift
            162 | 163 => 17, // Ctrl
            164 | 165 => 18, // Alt
            91 | 92 => 91,   // Windows
            _ => *key,
        })
        .collect()
}

unsafe fn get_pressed_keys() -> Vec<i32> {
    let global_key_states = GLOBAL_KEY_STATES.read().unwrap();
    global_key_states.keys().cloned().collect()
}

unsafe extern "system" fn keyboard_hook(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if n_code != HC_ACTION as i32 {
        return CallNextHookEx(None, n_code, w_param, l_param);
    }

    let kb_struct = &*(l_param.0 as *const KBDLLHOOKSTRUCT);
    let virtual_key_code = kb_struct.vkCode as i32;

    let is_key_down = w_param.0 as usize == WM_KEYDOWN as usize;
    let is_key_up = w_param.0 as usize == WM_KEYUP as usize;

    if is_key_down {
        println!("KEY DOWN: {:?}", virtual_key_code);
        GLOBAL_KEY_STATES
            .write()
            .unwrap()
            .insert(virtual_key_code, true);
    }

    if is_key_up {
        println!("KEY UP: {:?}", virtual_key_code);
        GLOBAL_KEY_STATES.write().unwrap().remove(&virtual_key_code);
    }

    let pressed_keys = get_pressed_keys();
    let normalized_pressed_keys = normalize_keys(pressed_keys);
    println!("PRESSED KEYS: {:?}", normalized_pressed_keys);

    // if pressed_keys matches any of the chords, trigger the callback
    for hook in HOOKS.read().unwrap().iter() {
        if hook
            .vk_codes
            .iter()
            .all(|&chord_key| normalized_pressed_keys.contains(&chord_key))
        {
            println!("CHORD MATCH DETECTED: {:?}", hook.vk_codes);
            let should_block = handle_chord_pressed(&hook);
            if should_block {
                return LRESULT(1);
            }
        }
    }

    CallNextHookEx(None, n_code, w_param, l_param)
}

pub struct KeyListener {
    parser: KeyChordParser,
}

impl KeyListener {
    pub fn new() -> Self {
        KeyListener {
            parser: KeyChordParser::new(),
        }
    }

    pub fn listen(&self, key_chord: &str, interval: Duration, callback: Callback) {
        if let Some(vk_codes) = self.parser.parse(key_chord) {
            unsafe {
                let hook_result = SetWindowsHookExW(
                    WH_KEYBOARD_LL,
                    Some(keyboard_hook),
                    GetModuleHandleW(None).unwrap_or_default(),
                    0,
                );

                let hook = match hook_result {
                    Ok(h) => h,
                    Err(_) => {
                        println!("Failed to set keyboard hook");
                        return;
                    }
                };

                let key_states = Arc::new(RwLock::new(HashMap::new()));
                for &code in &vk_codes {
                    key_states.write().unwrap().insert(code, false);
                }

                let mut hooks = HOOKS.write().unwrap();
                hooks.push(HookData {
                    callback,
                    vk_codes,
                    interval,
                    last_trigger: Arc::new(RwLock::new(Instant::now())),
                    hook_handle: Arc::new(RwLock::new(Some(hook.0 as isize))),
                });

                // Start a message loop in a separate thread with a timeout to prevent deadlocks
                thread::spawn(move || {
                    let mut msg = MSG::default();
                    while GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
                        // Process the message
                        // Add a small sleep to prevent high CPU usage
                        thread::sleep(Duration::from_millis(1));
                    }
                });
            }
        }
    }

    pub fn unlisten(&self) {
        let mut hooks = HOOKS.write().unwrap();
        hooks.iter_mut().for_each(|hook| {
            if let Some(hhk) = hook.hook_handle.write().unwrap().take() {
                unsafe { UnhookWindowsHookEx(HHOOK(hhk as *mut _)).unwrap() };
            }
        });
    }
}
