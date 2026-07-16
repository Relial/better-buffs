use std::time::Instant;

use anyhow::Result;
use bunny_plugin::hook_builder::{NoCbHookBuilder, NoCbHookPoint};
use ilhook::x86::{HookType, Registers};
use tracing::debug;

use crate::{
    address::Addresses,
    plugin::STATE,
    ui::{GuildFood, GuildPoogieBuff, TorePoogieItem},
};

pub unsafe extern "C" fn on_lobby_update() {
    unsafe {
        let state = STATE.get_unchecked_mut();
        if !state.applied {
            state.applied = true;
            let mut any_applied = false;
            let addresses = &state.addresses;

            if let Some(item) = &state.buffs.item {
                let ptr = addresses.poogie_item as *mut u16;
                ptr.write(item.kind);
                debug!("Applied poogie item: {}", item.kind);
                any_applied = true;
            }

            if let Some(buff) = &state.buffs.guild_buff {
                let ptr = (addresses.poogie_buff as *mut u8).wrapping_byte_add(buff.offset);
                ptr.write(buff.kind);
                debug!("Applied guild poogie buff: {}", buff.kind);
                any_applied = true;
            }

            if let Some(food) = &state.buffs.guild_food
                && let Some(mhfdat) = addresses.mhfdat()
            {
                let entry = addresses.food_skill_to_mhfdat_index(food.skill as usize);
                let entry_ptr = mhfdat.guild_food_entry(entry);
                let food_ptr = addresses.guild_food as *mut u8;
                (food_ptr as *mut u16).write(food.id);
                (food_ptr as *mut *mut u8)
                    .wrapping_byte_add(4)
                    .write(entry_ptr);
                (food_ptr as *mut u32)
                    .wrapping_byte_add(8)
                    .write(food.timestamp);
                debug!(
                    "Applied guild food id {} entry ptr {:#?} timestamp {}",
                    food.id, entry_ptr, food.timestamp
                );
                any_applied = true;
            }

            if any_applied && state.config.show_notification {
                state.notification_start = Some(Instant::now());
            }
        }
    }
}

unsafe extern "cdecl" fn on_guild_food(reg: *mut Registers, _: usize) {
    unsafe {
        let state = STATE.get_unchecked_mut();
        let timestamp = (*reg).edx;
        let skill = ((*reg).eax as *mut u16).read();
        let entry = (*reg).esi as u16;
        let guild_food = GuildFood::new(entry, skill, timestamp);
        state.buffs.guild_food = Some(guild_food);
        debug!("Saving guild food buff: {}", entry);
        state.save_buffs();
    }
}

fn hook_guild_food(addresses: &Addresses) -> Result<NoCbHookPoint> {
    let hook_address = addresses.apply_guild_food;
    let builder = NoCbHookBuilder::new(hook_address, HookType::JmpBack(on_guild_food));
    let hook_point = unsafe { builder.hook() }?;
    debug!("Hooked at {:#X}", hook_address);
    Ok(hook_point)
}

unsafe extern "cdecl" fn on_poogie_item_apply(reg: *mut Registers, _: usize) {
    unsafe {
        let state = STATE.get_unchecked_mut();
        let item_kind = ((*reg).ecx & 0xffff) as u16;
        let item = TorePoogieItem::new(item_kind);
        state.buffs.item = Some(item);
        debug!("Saving tore poogie item: {}", item_kind);
        state.save_buffs();
    }
}

fn hook_poogie_item(addresses: &Addresses) -> Result<NoCbHookPoint> {
    let hook_address = addresses.apply_tore_poogie_item;
    let builder = NoCbHookBuilder::new(hook_address, HookType::JmpBack(on_poogie_item_apply));
    let hook_point = unsafe { builder.hook() }?;
    debug!("Hooked at {:#X}", hook_address);
    Ok(hook_point)
}

unsafe extern "cdecl" fn on_poogie_buff_roll(reg: *mut Registers, _: usize) {
    unsafe {
        let state = STATE.get_unchecked_mut();
        (*reg).eflags |= 1;
        let offset = (*reg).esi as usize;
        let applied_buff = (state.addresses.available_buffs as *const u8)
            .wrapping_byte_add(offset)
            .read()
            + 1;
        let buff = GuildPoogieBuff::new(applied_buff, offset);
        state.buffs.guild_buff = Some(buff);
        debug!("Saving guild poogie buff: {}", applied_buff);
        state.save_buffs();
    }
}

fn hook_poogie_buff_roll(addresses: &Addresses) -> Result<NoCbHookPoint> {
    let hook_address = addresses.poogie_buff_roll;
    let builder = NoCbHookBuilder::new(hook_address, HookType::JmpBack(on_poogie_buff_roll));
    let hook_point = unsafe { builder.hook() }?;
    debug!("Hooked at {:#X}", hook_address);
    Ok(hook_point)
}

pub fn init(addresses: &Addresses) -> Result<Vec<NoCbHookPoint>> {
    Ok(vec![
        hook_poogie_buff_roll(addresses)?,
        hook_poogie_item(addresses)?,
        hook_guild_food(addresses)?,
    ])
}
