use std::{
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use anyhow::Result;
use bunny_plugin::PluginContext;
use serde::{Deserialize, Serialize};

use crate::address::Addresses;

pub struct State {
    pub context: PluginContext,
    pub addresses: Addresses,
    pub data_path: PathBuf,
    pub buff_to_apply: Option<Buff>,
}

impl State {
    pub fn new(context: PluginContext, addresses: Addresses) -> Self {
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
            data_path,
            buff_to_apply,
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
