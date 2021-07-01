use std::{
    alloc::{alloc, dealloc, Layout},
    ptr::{drop_in_place, null},
    sync::atomic::{fence, AtomicUsize, Ordering},
};

const INITIAL_COUNT: usize = 0;

#[derive(Debug)]
#[repr(C)]
pub struct ArcBlock {
    pointer: *const u8,
}

#[repr(C)]
struct ArcInner {
    count: AtomicUsize,
    payload: (),
}

impl ArcBlock {
    pub fn new(layout: Layout) -> Self {
        if layout.size() == 0 {
            Self::null()
        } else {
            let pointer = unsafe { &mut *(alloc(Self::inner_layout(layout)) as *mut ArcInner) };

            pointer.count = AtomicUsize::new(INITIAL_COUNT);

            Self {
                pointer: &pointer.payload as *const () as *const u8,
            }
        }
    }

    pub fn null() -> Self {
        Self { pointer: null() }
    }

    pub fn ptr(&self) -> *const u8 {
        &self.inner().payload as *const () as *const u8
    }

    pub fn ptr_mut(&mut self) -> *mut u8 {
        &self.inner().payload as *const () as *mut u8
    }

    pub fn is_null(&self) -> bool {
        self.pointer.is_null()
    }

    fn is_static(&self) -> bool {
        self.pointer as usize & 1 == 1
    }

    fn inner(&self) -> &ArcInner {
        unsafe { &*self.inner_pointer() }
    }

    fn inner_pointer(&self) -> *const ArcInner {
        (unsafe { (self.pointer as *const usize).offset(-1) } as usize & !1) as *const ArcInner
    }

    fn inner_layout(layout: Layout) -> Layout {
        Layout::new::<AtomicUsize>()
            .extend(layout)
            .unwrap()
            .0
            .pad_to_align()
    }

    pub fn clone(&self) -> Self {
        if !self.pointer.is_null() && !self.is_static() {
            self.inner().count.fetch_add(1, Ordering::Relaxed);
        }

        Self {
            pointer: self.pointer,
        }
    }

    pub fn drop<T>(&mut self) {
        if self.pointer.is_null() || self.is_static() {
            return;
        }

        if self.inner().count.fetch_sub(1, Ordering::Release) == INITIAL_COUNT {
            fence(Ordering::Acquire);

            unsafe {
                drop_in_place(self.inner_pointer() as *mut T);

                // This layout is expected not to be used.
                dealloc(
                    self.inner_pointer() as *mut u8,
                    Layout::from_size_align(1, 1).unwrap(),
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        ArcBlock::new(Layout::from_size_align(1, 1).unwrap());
    }

    #[test]
    fn create_null() {
        ArcBlock::null();
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
