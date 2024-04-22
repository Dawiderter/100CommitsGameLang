use slotmap::{new_key_type, SlotMap};

#[derive(Debug)]
pub enum Object {
    String(String)
}

new_key_type! { pub struct ObjectId; }

pub type ObjectHeap = SlotMap<ObjectId, Object>;