use bunny_plugin::{GameMode, MhfoInfo};

#[derive(Clone, Copy, Debug)]
pub struct Addresses {
    pub poogie_buff_roll: usize,
    pub poogie_buff: usize,
    pub available_buffs: usize,
}

impl Addresses {
    pub fn new(mhfo_info: MhfoInfo) -> Self {
        let dll = mhfo_info.address;
        match mhfo_info.game_mode {
            GameMode::LowGrade => Self {
                poogie_buff_roll: dll + 0x7e2894,
                poogie_buff: dll + 0x5b33fb3,
                available_buffs: dll + 0x5b33efb,
            },
            GameMode::HighGrade => Self {
                poogie_buff_roll: todo!(),
                poogie_buff: todo!(),
                available_buffs: todo!(),
            },
        }
    }
}
