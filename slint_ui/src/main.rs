slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let window = MainWindow::new()?;

    // Real TOTP timer: return seconds remaining in current 30-s period
    window.on_get_totp_remaining(|| {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        (30 - (secs % 30)) as i32
    });

    // Clipboard copy
    window.on_copy_to_clipboard(|text| {
        if let Ok(mut cb) = arboard::Clipboard::new() {
            cb.set_text(text.as_str()).ok();
        }
    });

    // Seed initial remaining seconds before first timer tick
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        window.set_global_remaining((30 - (secs % 30)) as i32);
    }

    window.run()
}
