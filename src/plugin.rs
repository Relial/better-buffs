use bunny_plugin::{PluginContext, PluginInfo, hook_builder::NoCbHookPoint, hook_cell::HookCell};
use tracing::error;

use crate::{
    address::Addresses,
    hooks::{self, on_lobby_update},
    state::State,
};

const PLUGIN_NAME: &str = "Better Guild Poogie";
const PLUGIN_VERSION: &str = env!("CARGO_PKG_VERSION");

pub static STATE: HookCell<State> = HookCell::new();
static HOOKS: HookCell<Vec<NoCbHookPoint>> = HookCell::new();

// Called once when the plugin is loaded
#[unsafe(no_mangle)]
pub extern "C" fn init(context: PluginContext) -> PluginInfo {
    tracing_subscriber::fmt()
        .without_time()
        .with_ansi(false)
        .with_max_level(context.log_level())
        .init();

    let addresses = Addresses::new(context.mhfo_info());
    let state = State::new(context, addresses);
    let state_res = STATE.set(state);
    let mut info = PluginInfo::new(PLUGIN_NAME, PLUGIN_VERSION).with_lobby_hook(on_lobby_update);

    match hooks::init(&addresses) {
        Ok(hooks) => {
            let hooks_res = HOOKS.set(hooks);
            if state_res.is_err() || hooks_res.is_err() {
                info = info
                    .with_init_fail("State/Hooks init failed: HookCell was already initialized");
            }
            info
        }
        Err(e) => {
            let err_message = format!("Hook init error: {e:#}");
            info = info.with_init_fail(err_message.as_str());
            error!(err_message);
            info
        }
    }
}

// Called every frame when the plugin's dropdown in the manager window is open
#[unsafe(no_mangle)]
pub extern "C" fn menu(_: &mut usize) {}

// Called every frame
#[unsafe(no_mangle)]
pub extern "C" fn ui(_: &mut usize) {}

// Called once per user defined autosave interval, and when the plugin is manually disabled by the user or the game is closed
#[unsafe(no_mangle)]
pub extern "C" fn save() {}

// Called when the plugin is manually disabled by the user
pub fn unload() {
    unsafe {
        HOOKS.drop();
        STATE.drop();
    }
}
