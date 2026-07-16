use bunny_plugin::{GameMode, MhfoInfo};

use crate::mhfdat::Mhfdat;

#[derive(Clone, Copy, Debug)]
pub struct Addresses {
    pub poogie_buff_roll: usize,
    pub apply_tore_poogie_item: usize,
    pub apply_guild_food: usize,
    pub poogie_buff: usize,
    pub available_buffs: usize,
    pub poogie_item: usize,
    mhfdat: usize,
    pub guild_food: usize,
    pub guild_food_entries: usize,
}

impl Addresses {
    pub fn new(mhfo_info: MhfoInfo) -> Self {
        let dll = mhfo_info.address;
        match mhfo_info.game_mode {
            GameMode::LowGrade => Self {
                poogie_buff_roll: dll + 0x7e2894,
                apply_tore_poogie_item: dll + 0x616c7d,
                apply_guild_food: dll + 0x7da0af,
                poogie_buff: dll + 0x5b33fb3,
                available_buffs: dll + 0x5b33efb,
                poogie_item: dll + 0x61540f8,
                mhfdat: dll + 0x5b4609c,
                guild_food: dll + 0x5bc70d8,
                guild_food_entries: dll + 0x1b96f78,
            },
            GameMode::HighGrade => Self {
                poogie_buff_roll: dll + 0x7fd104,
                apply_tore_poogie_item: dll + 0x63119d,
                apply_guild_food: dll + 0x7f497f,
                poogie_buff: dll + 0xe76bbdb,
                available_buffs: dll + 0xe76bb23,
                poogie_item: dll + 0xed8e898,
                mhfdat: dll + 0xe77dcc4,
                guild_food: dll + 0xe7fed00,
                guild_food_entries: dll + 0x1beeb30,
            },
        }
    }

    pub fn mhfdat(&self) -> Option<Mhfdat> {
        let ptr = unsafe { (self.mhfdat as *mut *mut u8).read() };
        (!ptr.is_null()).then(|| Mhfdat::new(ptr))
    }

    pub fn food_skill_to_mhfdat_index(&self, skill: usize) -> usize {
        let entries = self.guild_food_entries as *mut u16;
        unsafe { entries.wrapping_add(skill).read() as usize }
    }
}
