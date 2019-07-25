use std::collections::HashMap;
use std::os::raw::c_uint;

use crate::command::Command;

use xcb;

pub type KeySymbol = c_uint;
pub type ModMask = c_uint;

pub enum Modifier {
    Shift,
    Lock,
    Control,
    Mod1,
    Mod2,
    Mod3,
    Mod4,
    Mod5,
}

impl Modifier {
    pub fn get_mod_mask(&self) -> ModMask {
        match self {
            Modifier::Shift => xcb::MOD_MASK_SHIFT,
            Modifier::Lock => xcb::MOD_MASK_LOCK,
            Modifier::Control => xcb::MOD_MASK_CONTROL,
            Modifier::Mod1 => xcb::MOD_MASK_1,
            Modifier::Mod2 => xcb::MOD_MASK_2,
            Modifier::Mod3 => xcb::MOD_MASK_3,
            Modifier::Mod4 => xcb::MOD_MASK_4,
            Modifier::Mod5 => xcb::MOD_MASK_5,
        }
    }
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct Key {
    pub modifier: ModMask,
    pub key: KeySymbol,
}

impl Key {
    pub fn new(modifier: &Modifier, key: KeySymbol) -> Key {
        Key {
            modifier: modifier.get_mod_mask(),
            key: key,
        }
    }
}

pub struct KeyMap {
    pub key_map: HashMap<Key, Command>,
}
