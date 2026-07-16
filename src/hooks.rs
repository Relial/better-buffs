use anyhow::Result;
use bunny_plugin::hook_builder::{NoCbHookBuilder, NoCbHookPoint};
use ilhook::x86::{HookType, Registers};
use tracing::{debug, error};

use crate::{address::Addresses, plugin::STATE, ui::Buff};

pub unsafe extern "C" fn on_lobby_update() {
    unsafe {
        let state = STATE.get_unchecked_mut();
        if let Some(buff) = state.buff_to_apply {
            let ptr = (state.addresses.poogie_buff as *mut u8).wrapping_byte_add(buff.offset);
            ptr.write(buff.kind);
            state.buff_to_apply = None;
            debug!("Applied guild poogie buff: {}", buff.kind);
            state.reapplied = buff.kind;
        }
    }
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
        let buff = Buff::new(applied_buff, offset);
        debug!("Saved guild poogie buff: {}", applied_buff);
        if let Err(e) = buff.save(&state.data_path) {
            error!("Save error: {:#?}", e);
        }
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
    Ok(vec![hook_poogie_buff_roll(addresses)?])
}
