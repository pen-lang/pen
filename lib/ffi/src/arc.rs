mod arc_block;
mod arc_buffer;

use arc_block::*;
pub use arc_buffer::*;
use core::{alloc::Layout, marker::PhantomData, ops::Deref, ptr::write};

#[derive(Debug)]
#[repr(C)]
pub struct Arc<T> {
    block: ArcBlock,
    phantom: PhantomData<T>,
}

impl<T> Arc<T> {
    pub fn new(payload: T) -> Self {
        let mut block = ArcBlock::new(Layout::new::<T>());

        unsafe { write(block.ptr_mut() as *mut T, payload) };

        Self {
            block,
            phantom: PhantomData::default(),
        }
    }

    pub fn get_mut(this: &mut Self) -> Option<&mut T> {
        this.block
            .get_mut()
            .map(|pointer| unsafe { &mut *(pointer as *mut T) })
    }

    pub fn synchronize(self) {
        self.block.synchronize()
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

impl<T: Default> Default for Arc<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::boxed::Box;
    use core::mem::{drop, forget};

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
    fn new_box() {
        forget(Arc::new(Box::new(0)));
    }

    #[test]
    fn clone_box() {
        let x = Arc::new(Box::new(0));
        forget(x.clone());
        forget(x);
    }

    #[test]
    fn drop_box() {
        Arc::new(Box::new(0));
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

    fn drop_send_and_sync(_: impl Send + Sync) {}

    #[test]
    fn implement_send_and_sync() {
        drop_send_and_sync(Arc::new(()));
    }

    #[test]
    fn get_mut() {
        let mut arc = Arc::new(0);

        *Arc::get_mut(&mut arc).unwrap() = 42;

        assert_eq!(*arc, 42);
    }
}
