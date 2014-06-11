use std::u16;
use std::vec::Vec;

#[deriving(Clone, PartialEq)]
pub struct Handle {
    index: u16,
    generation: u16
}

pub struct EngineData<T> {
    data: Vec<T>,
    generation: Vec<u16>
}

impl<T> EngineData<T> {
    fn is_valid_handle(&self, handle: Handle) -> bool {
        handle.generation == *self.generation.get(handle.index as uint)
    }

    pub fn new() -> EngineData<T> {
        EngineData {
            data: Vec::with_capacity(u16::MAX as uint),
            generation: Vec::from_elem(u16::MAX as uint, 0u16)
        }
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.generation.clear();
    }

    pub fn drop(&mut self) {
        self.clear();
    }

    pub fn add(&mut self, data: T) -> Handle {
        self.data.push(data);
        let index = self.data.len() - 1;
        Handle {
            index: index as u16,
            generation: self.generation.get(index).clone()
        }
    }

    pub fn remove(&mut self, handle: Handle) {
        if self.is_valid_handle(handle) {
            self.data.swap_remove(handle.index as uint);
            *self.generation.get_mut(handle.index as uint) += 1;
        }
    }

    pub fn get<'a>(&'a self, handle: Handle) -> Option<&'a T> {
        if self.is_valid_handle(handle) {
            Some(self.data.get(handle.index as uint))
        } else {
            None
        }
    }
}
