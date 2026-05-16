use std::path::{Path, PathBuf};

/// Install ccline: copy binary to `~/.claude/ccline/` and patch
/// `~/.claude/settings.json` so Claude Code uses it as the status line.
///
/// Safe to run repeatedly. Backs up settings.json before each modification.
pub fn install() -> Result<(), Box<dyn std::error::Error>> {
    let home = dirs::home_dir().ok_or("cannot find home directory")?;
    let ccline_dir = home.join(".claude").join("ccline");
    let bin_name = if cfg!(windows) { "ccline.exe" } else { "ccline" };
    let target = ccline_dir.join(bin_name);

    std::fs::create_dir_all(&ccline_dir)?;

    let current = std::env::current_exe()?;
    let already_installed = current
        .canonicalize()
        .ok()
        .zip(target.canonicalize().ok())
        .map(|(a, b)| a == b)
        .unwrap_or(false);

    if already_installed {
        println!("✓ Binary already installed at: {}", target.display());
    } else {
        std::fs::copy(&current, &target)?;
        println!("✓ Binary installed: {}", target.display());
    }

    let settings_path = home.join(".claude").join("settings.json");
    update_settings(&settings_path)?;
    println!("✓ Settings updated: {}", settings_path.display());

    println!();
    println!("Installation complete. Restart Claude Code to activate the statusline.");
    println!("Run `ccline --doctor` anytime to re-verify the integration.");
    Ok(())
}

fn update_settings(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut settings: serde_json::Value = if path.exists() {
        let content = std::fs::read_to_string(path)?;
        let backup = path.with_extension("json.ccline-backup");
        std::fs::write(&backup, &content)?;
        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        serde_json::json!({})
    };

    if !settings.is_object() {
        settings = serde_json::json!({});
    }

    let obj = settings
        .as_object_mut()
        .expect("settings root is guaranteed object");
    obj.insert(
        "statusLine".to_string(),
        serde_json::json!({
            "type": "command",
            "command": "~/.claude/ccline/ccline",
            "padding": 0,
        }),
    );

    std::fs::write(path, serde_json::to_string_pretty(&settings)?)?;
    Ok(())
}

/// Verify the integration is healthy. Prints a checklist and exits 0 (all ok)
/// or 1 (something needs `--install`).
pub fn doctor() -> Result<bool, Box<dyn std::error::Error>> {
    let home = dirs::home_dir().ok_or("cannot find home directory")?;
    let ccline_dir = home.join(".claude").join("ccline");
    let bin_name = if cfg!(windows) { "ccline.exe" } else { "ccline" };
    let bin_path = ccline_dir.join(bin_name);
    let settings_path = home.join(".claude").join("settings.json");
    let projects_dir = home.join(".claude").join("projects");

    println!("CCometixLine Doctor");
    println!("───────────────────");

    let mut all_ok = true;

    if ccline_dir.is_dir() {
        println!("✓ ccline directory present: {}", ccline_dir.display());
    } else {
        println!("✗ ccline directory missing — run `ccline --install`");
        all_ok = false;
    }

    if bin_path.is_file() {
        let size = std::fs::metadata(&bin_path)?.len();
        println!(
            "✓ Binary present: {} ({} bytes)",
            bin_path.display(),
            size
        );
    } else {
        println!("✗ Binary missing — run `ccline --install`");
        all_ok = false;
    }

    if settings_path.is_file() {
        let content = std::fs::read_to_string(&settings_path)?;
        match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(j) => match j.get("statusLine").and_then(|v| v.get("command")) {
                Some(serde_json::Value::String(cmd)) => {
                    println!("✓ statusLine command: {}", cmd);
                }
                _ => {
                    println!(
                        "✗ statusLine entry missing in settings.json — run `ccline --install`"
                    );
                    all_ok = false;
                }
            },
            Err(e) => {
                println!("✗ settings.json invalid JSON: {}", e);
                all_ok = false;
            }
        }
    } else {
        println!("✗ settings.json missing — run `ccline --install`");
        all_ok = false;
    }

    if projects_dir.is_dir() {
        let count = std::fs::read_dir(&projects_dir)
            .map(|rd| rd.flatten().count())
            .unwrap_or(0);
        println!(
            "✓ Transcripts dir: {} ({} project folders)",
            projects_dir.display(),
            count
        );
    } else {
        println!("⚠ Transcripts dir missing — token stats will start empty");
    }

    println!();
    if all_ok {
        println!("All checks passed. statusline should be active after Claude Code restart.");
    } else {
        println!("Some checks failed. Run `ccline --install` to repair.");
    }
    Ok(all_ok)
}

/// Internal entry point: choose the right binary location regardless of platform.
#[allow(dead_code)]
pub fn target_binary_path() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let bin = if cfg!(windows) { "ccline.exe" } else { "ccline" };
    Some(home.join(".claude").join("ccline").join(bin))
}
