use std::collections::BTreeSet;

#[derive(Default)]
pub struct KbmContext {
    pub keyboard_modifiers: u8,
    pub keyboard_keys: BTreeSet<u8>,
    pub mouse_buttons: u8,
    //
    pub mouse_id: Option<u64>,
    pub keyboard_id: Option<u64>,
}
