//! small utility to initialize a static variable only once with a function

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::{spin_loop_hint, AtomicUsize, Ordering};

pub struct InitOnce<T> {
    status: AtomicUsize,
    content: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T: Send + Sync> Sync for InitOnce<T> {}
unsafe impl<T: Send> Send for InitOnce<T> {}

const STATUS_UNINIT: usize = 0;
const STATUS_INITING: usize = 1;
const STATUS_DONE: usize = 2;

impl<T> InitOnce<T> {
    pub const fn init() -> Self {
        InitOnce {
            status: AtomicUsize::new(STATUS_UNINIT),
            content: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    /// Load a value from the structure. if the structure is not initialized, then
    /// f is called once to compute the value and initialize the structure.
    pub fn load<F>(&self, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        let status = self
            .status
            .compare_and_swap(STATUS_UNINIT, STATUS_INITING, Ordering::SeqCst);
        if status == STATUS_UNINIT {
            // call F to write to cell and set the status to done
            let value = f();
            let cp = &self.content as *const UnsafeCell<MaybeUninit<T>> as *const MaybeUninit<T>
                as *mut MaybeUninit<T>;
            // write to the cell
            unsafe {
                let cp_ref = &mut *cp;
                cp_ref.as_mut_ptr().write(value)
            }
            self.status.store(STATUS_DONE, Ordering::SeqCst);
        } else if status == STATUS_INITING {
            // wait to be done
            while self.status.load(Ordering::SeqCst) != STATUS_DONE {
                spin_loop_hint()
            }
        }

        // read the cell
        unsafe {
            let mu = &*(self.content.get());
            &*(mu.as_ptr())
        }
    }
}
