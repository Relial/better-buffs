use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use bunny_components::text::{Text, TextBackground};
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
    pub buffs: Buffs,
    pub applied: bool,
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
        let mut buffs = Buffs::load(&data_path).unwrap_or_default();
        buffs.remove_old();
        Self {
            context,
            addresses,
            config,
            config_path,
            data_path,
            buffs,
            applied: false,
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
        if let Some(notification_start) = self.notification_start {
            let t = 1.0
                - notification_start
                    .elapsed()
                    .div_duration_f32(NOTIFICATION_DUR);
            if t < 0.0 {
                self.notification_start = None;
                return;
            }
            let t = if t > 0.2 { 1.0 } else { t / 0.2 };
            let painter = ui.painter();
            let max_rect = ui.max_rect();
            let color = Color32::GREEN.gamma_multiply(t);

            if let Some(buff) = &self.buffs.guild_buff {
                let buff_text = PoogieSkill::from_repr(buff.kind)
                    .map(|kind| kind.to_string())
                    .unwrap_or_else(|| buff.kind.to_string());
                let text = format!("Poogie {} activated", buff_text);
                let painted_text = Text::new(text, FontId::proportional(50.0))
                    .anchor(Align2::CENTER_TOP)
                    .pos(vec2(0.0, 70.0))
                    .pivot(Align2::CENTER_CENTER)
                    .background(TextBackground::new(
                        Color32::BLACK.gamma_multiply(t * 0.6),
                        3.0,
                    ))
                    .color(color);
                painted_text.paint(painter, max_rect);
            }
            if let Some(item) = &self.buffs.item {
                let item_text = PoogieItem::from_repr(item.kind)
                    .map(|kind| kind.to_string())
                    .unwrap_or_else(|| item.kind.to_string());
                let text = format!("Poogie {} enabled", item_text);
                let painted_text = Text::new(text, FontId::proportional(50.0))
                    .anchor(Align2::CENTER_TOP)
                    .pos(vec2(0.0, 70.0 * 2.0))
                    .pivot(Align2::CENTER_CENTER)
                    .background(TextBackground::new(
                        Color32::BLACK.gamma_multiply(t * 0.6),
                        3.0,
                    ))
                    .color(color);
                painted_text.paint(painter, max_rect);
            }
            if self.buffs.guild_food.is_some() {
                let text = "Guild food activated";
                let painted_text = Text::new(text, FontId::proportional(50.0))
                    .anchor(Align2::CENTER_TOP)
                    .pos(vec2(0.0, 70.0 * 3.0))
                    .pivot(Align2::CENTER_CENTER)
                    .background(TextBackground::new(
                        Color32::BLACK.gamma_multiply(t * 0.6),
                        3.0,
                    ))
                    .color(color);
                painted_text.paint(painter, max_rect);
            }
        }
    }

    pub fn save_config(&self) {
        if let Err(e) = self.config.save(&self.config_path) {
            error!("Config save error: {e:#?}");
        }
    }

    pub fn save_buffs(&self) {
        if let Err(e) = self.buffs.save(&self.data_path) {
            error!("Buffs save error: {e:#?}")
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Buffs {
    pub item: Option<TorePoogieItem>,
    pub guild_buff: Option<GuildPoogieBuff>,
    pub guild_food: Option<GuildFood>,
}

impl Buffs {
    fn load(path: impl AsRef<Path>) -> Result<Self> {
        let bytes = std::fs::read(path)?;
        let buff = postcard::from_bytes(&bytes)?;
        Ok(buff)
    }

    fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let bytes = postcard::to_allocvec(self)?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    fn remove_old(&mut self) {
        let limit = Duration::from_hours(12);
        if let Some(item) = &self.item
            && !(item.time.elapsed().is_ok_and(|elapsed| elapsed < limit))
        {
            self.item = None;
        }
        if let Some(buff) = &self.guild_buff
            && !(buff.time.elapsed().is_ok_and(|elapsed| elapsed < limit))
        {
            self.guild_buff = None;
        }
        if let Some(food) = &self.guild_food {
            if let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) {
                let now = now.as_secs() as u32;
                if now.saturating_sub(food.timestamp) > 5400 {
                    self.guild_food = None;
                }
            } else {
                self.guild_food = None;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TorePoogieItem {
    pub kind: u16,
    time: SystemTime,
}

impl TorePoogieItem {
    pub fn new(kind: u16) -> Self {
        Self {
            kind,
            time: SystemTime::now(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GuildPoogieBuff {
    pub kind: u8,
    pub offset: usize,
    time: SystemTime,
}

impl GuildPoogieBuff {
    pub fn new(kind: u8, offset: usize) -> Self {
        Self {
            kind,
            offset,
            time: SystemTime::now(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GuildFood {
    pub id: u16,
    pub skill: u16,
    pub timestamp: u32,
}

impl GuildFood {
    pub fn new(id: u16, skill: u16, timestamp: u32) -> Self {
        Self {
            id,
            skill,
            timestamp,
        }
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

#[derive(Clone, Copy, Debug, PartialEq, FromRepr)]
#[repr(u16)]
pub enum PoogieItem {
    Potion = 0x7,
    MegaPotion = 0x8,
    Antidote = 0xB,
    DashJuice = 0xD,
    MaxPotion = 0x1B,
    EnergyDrink = 0x1E,
}

impl std::fmt::Display for PoogieItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            PoogieItem::Potion => "Potion",
            PoogieItem::MegaPotion => "Mega Potion",
            PoogieItem::Antidote => "Antidote",
            PoogieItem::DashJuice => "Dash Juice",
            PoogieItem::MaxPotion => "Max Potion",
            PoogieItem::EnergyDrink => "Energy Drink",
        };
        write!(f, "{s}")
    }
}
