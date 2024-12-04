use alloc::alloc::{alloc, dealloc};
use core::{
    alloc::Layout,
    ptr::{drop_in_place, null},
    sync::atomic::{fence, AtomicIsize, Ordering},
};

const UNIQUE_COUNT: isize = 0;
const SYNCHRONIZED_UNIQUE_COUNT: isize = -1;
// The static count is also synchronized.
const STATIC_COUNT: isize = 1 << (isize::BITS as usize - 1);

#[derive(Debug)]
#[repr(C)]
pub struct ArcBlock {
    pointer: *const u8,
}

#[repr(C)]
struct ArcInner {
    count: AtomicIsize,
    payload: (),
}

impl ArcBlock {
    pub fn new(layout: Layout) -> Self {
        if layout.size() == 0 {
            Self { pointer: null() }
        } else {
            let pointer = unsafe { &mut *(alloc(Self::inner_layout(layout)) as *mut ArcInner) };

            pointer.count = AtomicIsize::new(UNIQUE_COUNT);

            Self {
                pointer: &pointer.payload as *const () as *const u8,
            }
        }
    }

    pub fn ptr(&self) -> *const u8 {
        &self.inner().payload as *const () as *const u8
    }

    pub fn ptr_mut(&mut self) -> *mut u8 {
        self.ptr() as *mut u8
    }

    pub fn get_mut(&mut self) -> Option<*mut u8> {
        if self.is_unique() {
            Some(self.ptr_mut())
        } else {
            None
        }
    }

    pub fn is_null(&self) -> bool {
        self.pointer.is_null()
    }

    fn inner(&self) -> &ArcInner {
        unsafe { &*self.inner_ptr() }
    }

    fn inner_ptr(&self) -> *const ArcInner {
        if self.is_null() {
            null()
        } else {
            (unsafe { (self.pointer as *const usize).offset(-1) }) as _
        }
    }

    fn inner_layout(layout: Layout) -> Layout {
        Layout::new::<AtomicIsize>()
            .extend(layout)
            .unwrap()
            .0
            .pad_to_align()
    }

    pub fn clone(&self) -> Self {
        if !self.is_null() {
            let count = self.inner().count.load(Ordering::Relaxed);

            if !Self::is_count_synchronized(count) {
                self.inner().count.store(count + 1, Ordering::Relaxed);
            } else if count != STATIC_COUNT {
                self.inner().count.fetch_sub(1, Ordering::Relaxed);
            }
        }

        Self {
            pointer: self.pointer,
        }
    }

    pub fn drop<T>(&mut self) {
        if self.is_null() {
            return;
        }

        let count = self.inner().count.load(Ordering::Relaxed);

        if Self::is_count_synchronized(count) {
            if count != STATIC_COUNT
                && self.inner().count.fetch_add(1, Ordering::Release) == SYNCHRONIZED_UNIQUE_COUNT
            {
                fence(Ordering::Acquire);

                self.drop_inner::<T>()
            }
        } else if count == UNIQUE_COUNT {
            self.drop_inner::<T>()
        } else {
            self.inner().count.store(count - 1, Ordering::Relaxed);
        }
    }

    fn drop_inner<T>(&mut self) {
        unsafe {
            drop_in_place(self.ptr() as *mut T);

            // The layout argument is expected not to be used.
            dealloc(
                self.inner_ptr() as *mut u8,
                Layout::from_size_align(1, 1).unwrap(),
            )
        }
    }

    pub fn synchronize(&self) {
        if self.is_null() {
            return;
        }

        let count = self.inner().count.load(Ordering::Relaxed);

        if !Self::is_count_synchronized(count) {
            self.inner()
                .count
                .store(SYNCHRONIZED_UNIQUE_COUNT - count, Ordering::Relaxed);
        }
    }

    fn is_count_synchronized(count: isize) -> bool {
        count < 0
    }

    fn is_unique(&self) -> bool {
        if self.is_null() {
            return false;
        }

        let count = self.inner().count.load(Ordering::Relaxed);

        count
            == if Self::is_count_synchronized(count) {
                fence(Ordering::Acquire);

                SYNCHRONIZED_UNIQUE_COUNT
            } else {
                UNIQUE_COUNT
            }
    }
}

unsafe impl Send for ArcBlock {}

unsafe impl Sync for ArcBlock {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        ArcBlock::new(Layout::from_size_align(1, 1).unwrap());
    }

    #[test]
    fn create_empty() {
        let block = ArcBlock::new(Layout::from_size_align(0, 1).unwrap());

        assert!(block.is_null());
    }

    #[test]
    fn clone() {
        ArcBlock::new(Layout::from_size_align(1, 1).unwrap()).clone();
    }

    #[test]
    fn drop() {
        ArcBlock::new(Layout::from_size_align(1, 1).unwrap()).drop::<u8>();
    }

    #[test]
    fn drop_twice() {
        let mut block = ArcBlock::new(Layout::from_size_align(1, 1).unwrap());
        block.clone().drop::<u8>();
        block.drop::<u8>();
    }

    #[test]
    fn get_mut() {
        let mut block = ArcBlock::new(Layout::from_size_align(1, 1).unwrap());
        assert!(block.get_mut().is_some());
    }

    #[test]
    fn fail_to_get_mut() {
        let mut block = ArcBlock::new(Layout::from_size_align(1, 1).unwrap());
        let _ = block.clone();
        assert!(block.get_mut().is_none());
    }

    mod sync {
        use super::*;

        #[test]
        fn synchronize() {
            let block = ArcBlock::new(Layout::from_size_align(1, 1).unwrap());
            block.synchronize();
        }

        #[test]
        fn synchronize_empty() {
            ArcBlock::new(Layout::from_size_align(0, 1).unwrap()).synchronize();
        }

        #[test]
        fn clone() {
            let block = ArcBlock::new(Layout::from_size_align(1, 1).unwrap());
            block.synchronize();
            block.clone();
        }

        #[test]
        fn drop() {
            let mut block = ArcBlock::new(Layout::from_size_align(1, 1).unwrap());
            block.synchronize();
            block.drop::<u8>();
        }

        #[test]
        fn drop_twice() {
            let mut block = ArcBlock::new(Layout::from_size_align(1, 1).unwrap());
            block.synchronize();
            block.clone().drop::<u8>();
            block.drop::<u8>();
        }

        #[test]
        fn get_mut() {
            let mut block = ArcBlock::new(Layout::from_size_align(1, 1).unwrap());
            block.synchronize();
            assert!(block.get_mut().is_some());
        }

        #[test]
        fn fail_to_get_mut_synchronizing_unique() {
            let mut block = ArcBlock::new(Layout::from_size_align(1, 1).unwrap());
            block.synchronize();
            let _ = block.clone();
            assert!(block.get_mut().is_none());
        }

        #[test]
        fn fail_to_get_mut_synchronizing_cloned() {
            let mut block = ArcBlock::new(Layout::from_size_align(1, 1).unwrap());
            let _ = block.clone();
            block.synchronize();
            assert!(block.get_mut().is_none());
        }
    }
}
