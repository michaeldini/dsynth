use crate::params::SynthParams;
use crate::preset::Preset;

/// Save a preset with a user-chosen file path
pub async fn save_preset_dialog(name: String, params: SynthParams) -> Result<(), String> {
    use rfd::AsyncFileDialog;

    let file = AsyncFileDialog::new()
        .set_title("Save Preset")
        .set_file_name(format!("{}.json", name))
        .add_filter("JSON", &["json"])
        .save_file()
        .await;

    if let Some(file) = file {
        let preset = Preset::new(name, params);
        preset
            .save(file.path())
            .map_err(|e| format!("Failed to save preset: {}", e))?;
    }

    Ok(())
}

/// Load a preset from a user-chosen file path
pub async fn load_preset_dialog() -> Result<SynthParams, String> {
    use rfd::AsyncFileDialog;

    let file = AsyncFileDialog::new()
        .set_title("Load Preset")
        .add_filter("JSON", &["json"])
        .pick_file()
        .await;

    if let Some(file) = file {
        let preset =
            Preset::load(file.path()).map_err(|e| format!("Failed to load preset: {}", e))?;
        return Ok(preset.params);
    }

    Err("No file selected".to_string())
}
