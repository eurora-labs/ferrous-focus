use ferrous_focus::subscribe_focus_changes;
use std::time::Duration;

fn main() {
    // Subscribe to focus changes
    let receiver = subscribe_focus_changes().unwrap();

    // Listen for focus events
    loop {
        match receiver.recv_timeout(Duration::from_millis(100)) {
            Ok(focused_window) => {
                println!("{:?}", focused_window);
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // Continue waiting
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                println!("Focus tracking channel disconnected");
                break;
            }
        }
    }
}
