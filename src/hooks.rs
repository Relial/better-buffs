use std::time::Instant;

use anyhow::Result;
use bunny_plugin::hook_builder::{NoCbHookBuilder, NoCbHookPoint};
use ilhook::x86::{HookType, Registers};
use tracing::debug;

use crate::{
    address::Addresses,
    plugin::STATE,
    ui::{GuildPoogieBuff, TorePoogieItem},
};

pub unsafe extern "C" fn on_lobby_update() {
    unsafe {
        let state = STATE.get_unchecked_mut();
        if !state.applied {
            state.applied = true;
            let mut any_applied = false;
            if let Some(item) = &state.buffs.item {
                let ptr = state.addresses.poogie_item as *mut u16;
                ptr.write(item.kind);
                debug!("Applied poogie item: {}", item.kind);
                any_applied = true;
            }
            if let Some(buff) = &state.buffs.guild_buff {
                let ptr = (state.addresses.poogie_buff as *mut u8).wrapping_byte_add(buff.offset);
                ptr.write(buff.kind);
                debug!("Applied guild poogie buff: {}", buff.kind);
                any_applied = true;
            }
            if any_applied && state.config.show_notification {
                state.notification_start = Some(Instant::now());
            }
        }
    }
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
    ])
}
