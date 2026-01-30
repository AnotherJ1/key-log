// 键盘鼠标监听器

use rdev::{listen, Event, EventType, Key, Button};
use std::sync::mpsc::Sender;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

/// 输入事件类型
#[derive(Debug, Clone)]
pub enum InputEvent {
    KeyPress { key_name: String, key_code: Option<i32> },
    MouseClick { button: String },
}

/// 全局监听器管理
static LISTENER_RUNNING: std::sync::OnceLock<Arc<AtomicBool>> = std::sync::OnceLock::new();

fn get_running_flag() -> Arc<AtomicBool> {
    LISTENER_RUNNING.get_or_init(|| Arc::new(AtomicBool::new(false))).clone()
}

/// 启动全局监听
pub fn start_global_listener(tx: Sender<InputEvent>) -> Result<(), String> {
    let running = get_running_flag();
    
    if running.load(Ordering::SeqCst) {
        return Err("Listener already running".to_string());
    }
    
    running.store(true, Ordering::SeqCst);
    
    running.store(true, Ordering::SeqCst);
    
    thread::spawn(move || {
        log::info!("Input listener thread started");
        
        let tx = tx;
        if let Err(e) = listen(move |event: Event| {
            if let Some(input_event) = process_event(&event) {
                if tx.send(input_event).is_err() {
                    // Channel closed, stop listening
                }
            }
        }) {
            log::error!("Error in input listener: {:?}", e);
        }
        
        log::info!("Input listener thread ended");
    });
    
    Ok(())
}

/// 停止全局监听
pub fn stop_global_listener() {
    let running = get_running_flag();
    running.store(false, Ordering::SeqCst);
    log::info!("Input listener stop requested");
}

/// 检查监听器是否运行
pub fn is_listener_running() -> bool {
    get_running_flag().load(Ordering::SeqCst)
}

/// 处理输入事件
fn process_event(event: &Event) -> Option<InputEvent> {
    match event.event_type {
        EventType::KeyPress(key) => {
            let key_name = key_to_string(&key);
            let key_code = key_to_code(&key);
            Some(InputEvent::KeyPress { key_name, key_code })
        }
        EventType::ButtonPress(button) => {
            let button_name = button_to_string(&button);
            Some(InputEvent::MouseClick { button: button_name })
        }
        _ => None,
    }
}

/// 将按键转换为字符串名称
fn key_to_string(key: &Key) -> String {
    match key {
        // 字母键
        Key::KeyA => "A".to_string(),
        Key::KeyB => "B".to_string(),
        Key::KeyC => "C".to_string(),
        Key::KeyD => "D".to_string(),
        Key::KeyE => "E".to_string(),
        Key::KeyF => "F".to_string(),
        Key::KeyG => "G".to_string(),
        Key::KeyH => "H".to_string(),
        Key::KeyI => "I".to_string(),
        Key::KeyJ => "J".to_string(),
        Key::KeyK => "K".to_string(),
        Key::KeyL => "L".to_string(),
        Key::KeyM => "M".to_string(),
        Key::KeyN => "N".to_string(),
        Key::KeyO => "O".to_string(),
        Key::KeyP => "P".to_string(),
        Key::KeyQ => "Q".to_string(),
        Key::KeyR => "R".to_string(),
        Key::KeyS => "S".to_string(),
        Key::KeyT => "T".to_string(),
        Key::KeyU => "U".to_string(),
        Key::KeyV => "V".to_string(),
        Key::KeyW => "W".to_string(),
        Key::KeyX => "X".to_string(),
        Key::KeyY => "Y".to_string(),
        Key::KeyZ => "Z".to_string(),
        
        // 数字键
        Key::Num0 => "0".to_string(),
        Key::Num1 => "1".to_string(),
        Key::Num2 => "2".to_string(),
        Key::Num3 => "3".to_string(),
        Key::Num4 => "4".to_string(),
        Key::Num5 => "5".to_string(),
        Key::Num6 => "6".to_string(),
        Key::Num7 => "7".to_string(),
        Key::Num8 => "8".to_string(),
        Key::Num9 => "9".to_string(),
        
        // 功能键
        Key::F1 => "F1".to_string(),
        Key::F2 => "F2".to_string(),
        Key::F3 => "F3".to_string(),
        Key::F4 => "F4".to_string(),
        Key::F5 => "F5".to_string(),
        Key::F6 => "F6".to_string(),
        Key::F7 => "F7".to_string(),
        Key::F8 => "F8".to_string(),
        Key::F9 => "F9".to_string(),
        Key::F10 => "F10".to_string(),
        Key::F11 => "F11".to_string(),
        Key::F12 => "F12".to_string(),
        
        // 特殊键
        Key::Space => "Space".to_string(),
        Key::Return => "Enter".to_string(),
        Key::Tab => "Tab".to_string(),
        Key::Backspace => "Backspace".to_string(),
        Key::Escape => "Escape".to_string(),
        Key::Delete => "Delete".to_string(),
        Key::CapsLock => "CapsLock".to_string(),
        
        // 修饰键
        Key::ShiftLeft => "Shift".to_string(),
        Key::ShiftRight => "Shift".to_string(),
        Key::ControlLeft => "Ctrl".to_string(),
        Key::ControlRight => "Ctrl".to_string(),
        Key::Alt => "Alt".to_string(),
        Key::AltGr => "AltGr".to_string(),
        Key::MetaLeft => "Win".to_string(),
        Key::MetaRight => "Win".to_string(),
        
        // 方向键
        Key::UpArrow => "Up".to_string(),
        Key::DownArrow => "Down".to_string(),
        Key::LeftArrow => "Left".to_string(),
        Key::RightArrow => "Right".to_string(),
        
        // 其他
        Key::Home => "Home".to_string(),
        Key::End => "End".to_string(),
        Key::PageUp => "PageUp".to_string(),
        Key::PageDown => "PageDown".to_string(),
        Key::Insert => "Insert".to_string(),
        Key::PrintScreen => "PrintScreen".to_string(),
        Key::ScrollLock => "ScrollLock".to_string(),
        Key::Pause => "Pause".to_string(),
        
        // 小键盘
        Key::Kp0 => "Num0".to_string(),
        Key::Kp1 => "Num1".to_string(),
        Key::Kp2 => "Num2".to_string(),
        Key::Kp3 => "Num3".to_string(),
        Key::Kp4 => "Num4".to_string(),
        Key::Kp5 => "Num5".to_string(),
        Key::Kp6 => "Num6".to_string(),
        Key::Kp7 => "Num7".to_string(),
        Key::Kp8 => "Num8".to_string(),
        Key::Kp9 => "Num9".to_string(),
        Key::KpMinus => "NumMinus".to_string(),
        Key::KpPlus => "NumPlus".to_string(),
        Key::KpMultiply => "NumMultiply".to_string(),
        Key::KpDivide => "NumDivide".to_string(),
        Key::KpReturn => "NumEnter".to_string(),
        Key::KpDelete => "NumDecimal".to_string(),
        Key::NumLock => "NumLock".to_string(),
        
        // 符号键
        Key::Minus => "Minus".to_string(),
        Key::Equal => "Equal".to_string(),
        Key::LeftBracket => "LeftBracket".to_string(),
        Key::RightBracket => "RightBracket".to_string(),
        Key::BackSlash => "Backslash".to_string(),
        Key::SemiColon => "Semicolon".to_string(),
        Key::Quote => "Quote".to_string(),
        Key::Comma => "Comma".to_string(),
        Key::Dot => "Period".to_string(),
        Key::Slash => "Slash".to_string(),
        Key::BackQuote => "Backquote".to_string(),
        
        // 未知按键
        Key::Unknown(code) => format!("Unknown({})", code),
        _ => format!("{:?}", key),
    }
}

/// 将按键转换为键码
fn key_to_code(key: &Key) -> Option<i32> {
    match key {
        Key::Unknown(code) => Some(*code as i32),
        _ => None,
    }
}

/// 将鼠标按钮转换为字符串
fn button_to_string(button: &Button) -> String {
    match button {
        Button::Left => "Left".to_string(),
        Button::Right => "Right".to_string(),
        Button::Middle => "Middle".to_string(),
        Button::Unknown(code) => format!("Button{}", code),
    }
}
