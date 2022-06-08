use alloc::alloc::{alloc, dealloc};
use core::{
    alloc::Layout,
    ptr::{drop_in_place, null},
    sync::atomic::{fence, AtomicI32, Ordering},
};

const INITIAL_COUNT: i32 = 0;
const EMPTY_TAG: u32 = 0;

#[derive(Debug)]
#[repr(C)]
pub struct ArcBlock {
    pointer: *const u8,
}

#[derive(Debug)]
#[repr(C)]
struct ArcHeader {
    count: AtomicI32,
    tag: u32,
}

#[repr(C)]
struct ArcInner {
    header: ArcHeader,
    payload: (),
}

impl ArcBlock {
    pub fn new(layout: Layout) -> Self {
        if layout.size() == 0 {
            Self { pointer: null() }
        } else {
            let pointer = unsafe { &mut *(alloc(Self::inner_layout(layout)) as *mut ArcInner) };

            pointer.header = ArcHeader {
                count: AtomicI32::new(INITIAL_COUNT),
                tag: EMPTY_TAG,
            };

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
        if !self.is_static() && self.inner().header.count.load(Ordering::Acquire) == INITIAL_COUNT {
            Some(self.ptr_mut())
        } else {
            None
        }
    }

    pub fn is_null(&self) -> bool {
        self.pointer.is_null()
    }

    fn is_static(&self) -> bool {
        self.pointer.is_null() || self.pointer as usize & 1 == 1
    }

    fn inner(&self) -> &ArcInner {
        unsafe { &*self.inner_ptr() }
    }

    fn inner_ptr(&self) -> *const ArcInner {
        let pointer = self.pointer as usize & !1;

        (unsafe { (pointer as *const usize).offset(-1) }) as *const ArcInner
    }

    fn inner_layout(layout: Layout) -> Layout {
        Layout::new::<ArcHeader>()
            .extend(layout)
            .unwrap()
            .0
            .pad_to_align()
    }

    pub fn clone(&self) -> Self {
        if !self.is_static() {
            let count = self.inner().header.count.load(Ordering::Relaxed);

            if Self::is_count_synchronized(count) {
                self.inner().header.count.fetch_sub(1, Ordering::Relaxed);
            } else {
                self.inner()
                    .header
                    .count
                    .store(count + 1, Ordering::Relaxed);
            }
        }

        Self {
            pointer: self.pointer,
        }
    }

    pub fn drop<T>(&mut self) {
        if self.is_static() {
            return;
        }

        let count = self.inner().header.count.load(Ordering::Relaxed);

        if Self::is_count_synchronized(count) {
            if self.inner().header.count.fetch_add(1, Ordering::Release) == INITIAL_COUNT {
                fence(Ordering::Acquire);

                self.drop_inner::<T>()
            }
        } else if count == INITIAL_COUNT {
            self.drop_inner::<T>()
        } else {
            self.inner()
                .header
                .count
                .store(count - 1, Ordering::Relaxed);
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

    fn is_count_synchronized(count: i32) -> bool {
        count < 0
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
        let mut arc = ArcBlock::new(Layout::from_size_align(1, 1).unwrap());
        arc.clone().drop::<u8>();
        arc.drop::<u8>();
    }
}
