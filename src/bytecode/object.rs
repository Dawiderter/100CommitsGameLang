use std::{fmt::Display, mem};

use ahash::AHashMap;
use ecow::EcoString;
use slotmap::{new_key_type, SlotMap};

#[derive(Debug)]
pub struct Object {
    pub kind: ObjectKind,
}

#[derive(Debug)]
pub enum ObjectKind {
    String(EcoString),
}

impl Object {
    pub fn new(kind: ObjectKind) -> Self {
        Self { kind }
    }
}

new_key_type! { pub struct ObjectKey; }

#[derive(Debug)]
pub struct ObjectHeap {
    heap: SlotMap<ObjectKey, Object>,
    interner: AHashMap<EcoString, ObjectKey>,
    dynamic_memory_used: usize,
}

impl ObjectHeap {
    pub fn new() -> Self {
        Self {
            heap: SlotMap::with_key(),
            interner: AHashMap::new(),
            dynamic_memory_used: 0,
        }
    }

    pub fn alloc_object(&mut self, object: Object) -> ObjectKey {
        Self::inner_heap_alloc(&mut self.heap, &mut self.dynamic_memory_used, object)
    }

    pub fn intern_string(&mut self, string: EcoString) -> ObjectKey {
        *self.interner.entry(string).or_insert_with_key(|s| {
            let obj = Object::new(ObjectKind::String(s.clone()));
            Self::inner_heap_alloc(&mut self.heap, &mut self.dynamic_memory_used, obj)
        })
    }

    pub fn get_object(&self, key: ObjectKey) -> &Object {
        self.heap
            .get(key)
            .expect("Internal error: Tried to get freed object")
    }

    pub fn live_count(&self) -> usize {
        self.heap.len()
    }

    pub fn dynamic_memory_used(&self) -> usize {
        self.dynamic_memory_used
    }

    fn inner_heap_alloc(
        inner_heap: &mut SlotMap<ObjectKey, Object>,
        mem_counter: &mut usize,
        obj: Object,
    ) -> ObjectKey {
        match &obj.kind {
            ObjectKind::String(s) => *mem_counter += mem::size_of_val(s.as_bytes()),
        }
        inner_heap.insert(obj)
    }
}

impl Display for ObjectKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectKind::String(string) => write!(f, "{}", string),
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}

impl Default for ObjectHeap {
    fn default() -> Self {
        Self::new()
    }
}
