use std::process::Command;

// Get selected text from primary clipboard (Linux/X11)
pub fn get_selected_text() -> Option<String> {
    // Try Wayland first
    if let Ok(output) = Command::new("wl-paste")
        .args(&["--primary", "--no-newline"])
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    // Fallback to X11
    if let Ok(output) = Command::new("xsel")
        .args(&["-o", "-p"])
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    // Try xclip as another fallback
    if let Ok(output) = Command::new("xclip")
        .args(&["-o", "-selection", "primary"])
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_selected_text() {
        // This test would require actual clipboard content
        // In a real environment, you might set up test scenarios
        let result = get_selected_text();
        // We can't assert specific values since clipboard content varies
        println!("Selected text: {:?}", result);
    }
}