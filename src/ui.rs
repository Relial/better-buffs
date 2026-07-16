use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime},
};

use anyhow::Result;
use bunny_components::text::{Text, TextBackground, TextShadow};
use bunny_plugin::{
    PluginContext,
    bunny_ui::{Color32, align::Align2, paint::text::fonts::FontId, ui::BunnyUi, vec2},
};
use serde::{Deserialize, Serialize};
use strum::FromRepr;
use tracing::error;

use crate::{address::Addresses, config::Config};

const NOTIFICATION_DUR: Duration = Duration::from_secs(6);

pub struct State {
    pub context: PluginContext,
    pub addresses: Addresses,
    pub config: Config,
    config_path: PathBuf,
    pub data_path: PathBuf,
    pub buff_to_apply: Option<Buff>,
    pub reapplied: u8,
    pub notification_start: Option<Instant>,
}

impl<'a> State {
    pub fn new(context: PluginContext, addresses: Addresses) -> Self {
        let config_path = context
            .config_dir()
            .join(format!("{}.toml", env!("CARGO_PKG_NAME")));
        let config = Config::load(&config_path).unwrap_or_default();
        let data_path = context
            .config_dir()
            .join(format!("{}.bin", env!("CARGO_PKG_NAME")));
        let buff_to_apply = Buff::load(&data_path).ok().and_then(|buff| {
            const LIMIT: Duration = Duration::from_hours(12);
            if buff.time.elapsed().is_ok_and(|dur| dur < LIMIT) {
                Some(buff)
            } else {
                None
            }
        });
        Self {
            context,
            addresses,
            config,
            config_path,
            data_path,
            buff_to_apply,
            reapplied: 0,
            notification_start: None,
        }
    }

    pub fn menu(&'a mut self, ui: &mut BunnyUi<'a>) {
        ui.checkbox(
            &mut self.config.show_notification,
            "Show notification when buff is reapplied",
        );
    }

    pub fn ui(&mut self, ui: &mut BunnyUi) {
        if self.reapplied > 0 && self.config.show_notification {
            let notification_start = self.notification_start.get_or_insert_with(Instant::now);
            let t = 1.0
                - notification_start
                    .elapsed()
                    .div_duration_f32(NOTIFICATION_DUR);
            if t < 0.0 {
                self.reapplied = 0;
                self.notification_start = None;
                return;
            }
            let t = if t > 0.2 { 1.0 } else { t / 0.2 };
            let painter = ui.painter();
            let max_rect = ui.max_rect();
            let color = Color32::GREEN.gamma_multiply(t);
            let buff_text = PoogieSkill::from_repr(self.reapplied)
                .map(|kind| kind.to_string())
                .unwrap_or_else(|| self.reapplied.to_string());
            let text = format!("Reapplied guild poogie buff: {}", buff_text);
            let painted_text = Text::new(text, FontId::proportional(50.0))
                .anchor(Align2::CENTER_TOP)
                .pos(vec2(0.0, max_rect.height() / 4.0))
                .pivot(Align2::CENTER_CENTER)
                .background(TextBackground::new(
                    Color32::BLACK.gamma_multiply(t * 0.6),
                    3.0,
                ))
                .color(color);
            painted_text.paint(painter, max_rect);
        }
    }

    pub fn save_config(&self) {
        if let Err(e) = self.config.save(&self.config_path) {
            error!("Config save error: {e}");
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Buff {
    pub kind: u8,
    pub offset: usize,
    time: SystemTime,
}

impl Buff {
    pub fn new(kind: u8, offset: usize) -> Self {
        Self {
            kind,
            offset,
            time: SystemTime::now(),
        }
    }

    fn load(path: impl AsRef<Path>) -> Result<Self> {
        let bytes = std::fs::read(path)?;
        let buff = postcard::from_bytes(&bytes)?;
        Ok(buff)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let bytes = postcard::to_allocvec(self)?;
        std::fs::write(path, bytes)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, FromRepr)]
#[repr(u8)]
pub enum PoogieSkill {
    Thrift = 1,
    Discount = 2,
    Taijutsu = 3,
    Status = 4,
    Reward = 5,
    Defense = 6,
    Escape = 7,
    Transportation = 8,
    Trap = 9,
    Patience = 10,
}

impl std::fmt::Display for PoogieSkill {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            PoogieSkill::Thrift => "Thrift",
            PoogieSkill::Discount => "Discount",
            PoogieSkill::Taijutsu => "Taijutsu",
            PoogieSkill::Status => "Status",
            PoogieSkill::Reward => "Reward",
            PoogieSkill::Defense => "Defense",
            PoogieSkill::Escape => "Escape",
            PoogieSkill::Transportation => "Transportation",
            PoogieSkill::Trap => "Trap",
            PoogieSkill::Patience => "Patience",
        };
        write!(f, "{s}")
    }
}
