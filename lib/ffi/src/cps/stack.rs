use crate::DEFAULT_MEMORY_ALIGNMENT;
use std::{
    alloc::{alloc, realloc, Layout},
    ptr,
};

const CAPACITY_MULTIPLIER: usize = 2;

#[repr(C)]
#[derive(Debug)]
pub struct Stack {
    base_pointer: *mut u8,
    size: usize,
    capacity: usize,
}

impl Stack {
    pub fn new(capacity: usize) -> Self {
        let layout = Layout::from_size_align(capacity, DEFAULT_MEMORY_ALIGNMENT)
            .unwrap()
            .pad_to_align();

        Self {
            capacity: layout.size(),
            base_pointer: unsafe { alloc(layout) },
            size: 0,
        }
    }

    pub fn push<T>(&mut self, value: T) {
        let pointer = self.get_pointer() as *mut T;

        self.size += Self::get_type_size::<T>();

        while self.size > self.capacity {
            let new_capacity = CAPACITY_MULTIPLIER * self.capacity;

            self.base_pointer = unsafe {
                realloc(
                    self.base_pointer,
                    Layout::from_size_align(self.capacity, DEFAULT_MEMORY_ALIGNMENT).unwrap(),
                    new_capacity,
                )
            };
            self.capacity = new_capacity;
        }

        unsafe {
            ptr::write(pointer, value);
        }
    }

    pub fn pop<T>(&mut self) -> T {
        self.size -= Self::get_type_size::<T>();

        unsafe { ptr::read(self.get_pointer() as *const T) }
    }

    fn get_pointer(&self) -> usize {
        self.base_pointer as usize + self.size
    }

    fn get_type_size<T>() -> usize {
        Layout::new::<T>()
            .align_to(DEFAULT_MEMORY_ALIGNMENT)
            .unwrap()
            .pad_to_align()
            .size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_CAPACITY: usize = 1;

    #[test]
    fn push_u8() {
        let mut stack = Stack::new(TEST_CAPACITY);

        stack.push(42u8);

        assert_eq!(stack.pop::<u8>(), 42);
    }

    #[test]
    fn push_multiple_u8() {
        let mut stack = Stack::new(TEST_CAPACITY);

        stack.push(42u8);
        stack.push(42u8);

        assert_eq!(stack.pop::<u8>(), 42);
        assert_eq!(stack.pop::<u8>(), 42);
    }

    #[test]
    fn push_f32() {
        let mut stack = Stack::new(TEST_CAPACITY);

        stack.push(42.0f32);

        assert_eq!(stack.pop::<f32>(), 42.0);
    }

    #[test]
    fn push_f64() {
        let mut stack = Stack::new(TEST_CAPACITY);

        stack.push(42.0f64);

        assert_eq!(stack.pop::<f64>(), 42.0);
    }
}
