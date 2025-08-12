use rdev::{listen, Event, EventType, Key};

fn main() {
    println!("开始监听键盘事件，请按 CapsLock 键进行测试...");
    println!("按 Ctrl+C 退出");

    if let Err(error) = listen(callback) {
        println!("监听错误: {:?}", error);
    }
}

fn callback(event: Event) {
    match event.event_type {
        EventType::KeyPress(key) => {
            println!("按键按下: {:?}", key);
            if key == Key::CapsLock {
                println!("🎯 检测到 CapsLock 按键！");
            }
        }
        EventType::KeyRelease(key) => {
            println!("按键释放: {:?}", key);
            if key == Key::CapsLock {
                println!("🎯 检测到 CapsLock 释放！");
            }
        }
        _ => {
            // 忽略其他事件
        }
    }
}