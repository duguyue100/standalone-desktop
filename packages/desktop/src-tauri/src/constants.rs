use tauri_plugin_window_state::StateFlags;

pub const SETTINGS_STORE: &str = "alfalfa.settings.dat";
pub const DEFAULT_SERVER_URL_KEY: &str = "defaultServerUrl";
pub const WSL_ENABLED_KEY: &str = "wslEnabled";
pub const UPDATER_ENABLED: bool = match option_env!("TAURI_SIGNING_PRIVATE_KEY") {
    Some(key) => !key.is_empty(),
    None => false,
};

pub fn window_state_flags() -> StateFlags {
    StateFlags::all() - StateFlags::DECORATIONS - StateFlags::VISIBLE
}
