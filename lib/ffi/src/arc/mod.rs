mod arc_block;
mod arc_buffer;

use arc_block::*;
pub use arc_buffer::*;
use std::{alloc::Layout, marker::PhantomData, ops::Deref};

#[derive(Debug)]
#[repr(C)]
pub struct Arc<T> {
    block: ArcBlock,
    phantom: PhantomData<T>,
}

impl<T> Arc<T> {
    pub fn new(payload: T) -> Self {
        let mut block = ArcBlock::new(Layout::new::<T>());

        unsafe {
            *(block.ptr_mut() as *mut T) = payload;
        }

        Self {
            block,
            phantom: PhantomData::default(),
        }
    }
}

impl<T> From<T> for Arc<T> {
    fn from(payload: T) -> Self {
        Self::new(payload)
    }
}

impl<T> Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*(self.block.ptr() as *const T) }
    }
}

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Self {
        Self {
            block: self.block.clone(),
            phantom: PhantomData::default(),
        }
    }
}

impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        self.block.drop::<T>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drop<T>(_: T) {}

    #[test]
    fn create() {
        Arc::new(0);
    }

    #[test]
    fn clone() {
        let arc = Arc::new(0);
        drop(arc.clone());
        drop(arc);
    }

    #[test]
    fn load_payload() {
        assert_eq!(*Arc::new(42), 42);
    }

    mod zero_sized {
        use super::*;

        #[test]
        fn create() {
            Arc::new(());
        }

        #[test]
        fn clone() {
            let arc = Arc::new(());
            drop(arc.clone());
            drop(arc);
        }

        #[test]
        #[allow(clippy::unit_cmp)]
        fn load_payload() {
            assert_eq!(*Arc::new(()), ());
        }
    }
}
