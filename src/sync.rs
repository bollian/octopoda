//! Synchronization primitives.
//!
//! At the moment, these are wildly incorrect implementations that don't use proper atomic
//! operations. This is only because the RPi3 doesn't seem to accept the ARMv8 atomic instructions.
//! There's probably some flag that needs to be set in the CPU to enable them.
//!
//! So long as this kernel is single-threaded, this isn't actually a problem. These type instead
//! serve to get around the sharing restrictions imposed by the rust compiler. Eventually, the
//! bug(s) that prevent the use of atomics need to be fixed.

use core::cell::{Cell, UnsafeCell};
use core::sync::atomic::{AtomicUsize, AtomicU32, Ordering};
use core::mem::MaybeUninit;
use crate::defer::defer;
use crate::bsp::mmap;
use crate::driver;

pub trait RawMutex {
    /// Check and see if the lock has been acquired.
    fn is_locked(&self) -> bool;

    /// Acquire the lock if it's available.
    ///
    /// If the lock is available, this returns true. Otherwise, return false.
    fn try_lock(&self) -> bool;

    /// Acquire the lock and block if already acquired.
    fn lock(&self);

    /// Release the lock.
    ///
    /// # Safety
    ///
    /// All references to the data protected by this lock must be dropped before unlocking,
    /// otherwise there will be undefined behavior.
    ///
    /// Attempting to unlock a mutex that is already unlocked is a safe operation.
    unsafe fn unlock(&self);
}

pub type SpinMutex<T> = Mutex<Spin, T>;
pub type SpinMutexMut<'a, T> = MutexMut<'a, Spin, T>;

pub struct Spin {
    locked: UnsafeCell<bool>,
}

unsafe impl Send for Spin {}
unsafe impl Sync for Spin {}

impl Spin {
    const INIT: Self = Self { locked: UnsafeCell::new(false) };

    unsafe fn set_lock_state(&self, acquired: bool) {
        self.locked.get().write_volatile(acquired)
    }
}

impl Default for Spin {
    fn default() -> Self {
        Self::INIT
    }
}

impl RawMutex for Spin {
    fn is_locked(&self) -> bool {
        unsafe { *self.locked.get() }
    }

    fn try_lock(&self) -> bool {
        if !self.is_locked() {
            unsafe { self.set_lock_state(true) }
            true
        } else {
            false
        }
    }

    fn lock(&self) {
        while self.is_locked() {
            crate::arch::asm::nop()
        }
        unsafe { self.set_lock_state(true) }
    }

    unsafe fn unlock(&self) {
        self.set_lock_state(false)
    }
}

pub struct Mutex<R, T>
where
    R: RawMutex,
    T: ?Sized,
{
    mutex: R,
    data: UnsafeCell<T>,
}

unsafe impl<R: RawMutex + Send, T: ?Sized + Send> Send for Mutex<R, T> {}
unsafe impl<R: RawMutex + Sync, T: ?Sized + Send> Sync for Mutex<R, T> {}

impl<R, T> Mutex<R, T>
where
    R: RawMutex + Default,
{
    pub fn new(data: T) -> Self {
        Self::new_from_raw(R::default(), data)
    }
}

impl<R, T> Mutex<R, T>
where
    R: RawMutex,
{
    /// Create a new mutex given the provided raw implementation.
    pub fn new_from_raw(mutex: R, data: T) -> Self {
        Self {
            mutex,
            data: UnsafeCell::new(data),
        }
    }
}

impl<R, T> Mutex<R, T>
where
    R: RawMutex,
    T: ?Sized,
{
    pub fn with_lock<F, V>(&self, critical_section: F) -> V
    where
        F: FnOnce(&mut T) -> V
    {
        self.mutex.lock();
        // SAFETY: safe because we just acquired this lock, so we're responsible for releasing it
        let _d = defer(|| unsafe { self.mutex.unlock() });
        return critical_section(unsafe {
            &mut *self.data.get()
        });
    }

    pub fn borrow<'a, T2>(&'a self) -> MutexMut<'a, R, T2>
    where
        T: AsMut<T2>,
        T2: ?Sized,
    {
        MutexMut {
            mutex: &self.mutex,
            data: unsafe { &mut *self.data.get() }.as_mut(),
        }
    }
}

pub struct MutexMut<'a, R, T>
where
    R: RawMutex,
    T: ?Sized,
{
    mutex: &'a R,
    data: *mut T,
}

unsafe impl<R: RawMutex + Sync, T: ?Sized + Send> Send for MutexMut<'_, R, T> {}
unsafe impl<R: RawMutex + Sync, T: ?Sized + Send> Sync for MutexMut<'_, R, T> {}

impl<R: RawMutex, T: ?Sized> MutexMut<'_, R, T> {
    pub fn with_lock<F, V>(&self, critical_section: F) -> V
    where
        F: FnOnce(&mut T) -> V,
    {
        self.mutex.lock();
        let _d = defer({
            let mutex = &self.mutex; // only capture the mutex field
            move || unsafe { mutex.unlock() }
        });
        return critical_section(unsafe {
            &mut *self.data
        });
    }
}

const ONCE_CELL_FILLED: u32 = 2;
const ONCE_CELL_FILLING: u32 = 1;
const ONCE_CELL_UNFILLED: u32 = 0;

pub struct OnceCell<T> {
    filled: UnsafeCell<u32>,
    data: UnsafeCell<MaybeUninit<T>>,
    _no_send_sync: core::marker::PhantomData<*mut T>,
}

// SAFETY: if one thread fills the cell through a shared reference
// only for another thread to observe that value later implies
// that T was sent between threads, so T must also be Send in
// addition to Sync
unsafe impl<T: Sync + Send> Sync for OnceCell<T> {}
unsafe impl<T: Send> Send for OnceCell<T> {}

impl<T> Default for OnceCell<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> OnceCell<T> {
    pub const fn new() -> Self {
        Self {
            filled: UnsafeCell::new(ONCE_CELL_UNFILLED),
            data: UnsafeCell::new(MaybeUninit::uninit()),
            _no_send_sync: core::marker::PhantomData,
        }
    }

    pub fn is_initialized(&self) -> bool {
        ONCE_CELL_FILLED == unsafe { *self.filled.get() }
    }

    pub fn get(&self) -> Option<&T> {
        if self.is_initialized() {
            // SAFETY: the above check means that the value was already initialized
            unsafe {
                return Some(self.get_unchecked())
            }
        }
        None
    }

    pub fn get_or_init<F>(&self, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        match self.get() {
            Some(data) => data,
            None => {
                unsafe {
                    self.filled.get().write_volatile(ONCE_CELL_FILLING);

                    // SAFETY: because of the compare-exchange check, we know we're the only one
                    // modifying the data field
                    self.data.get().write(MaybeUninit::new(f()));
                    let filled_ref = self.get_unchecked();

                    self.filled.get().write_volatile(ONCE_CELL_FILLED);
                    filled_ref
                }
            }
        }
    }

    pub unsafe fn get_unchecked(&self) -> &T {
        (*self.data.get()).assume_init_ref()
    }
}

/// Marker type for `Lazy`.
///
/// We set Send + Sync markers on this type separately from `Lazy` so that we don't accidentally
/// override the markers that may have been unset on the inner `OnceCell`.
struct LazyInitWrapper<F>(UnsafeCell<MaybeUninit<F>>);

/// SAFETY: Since the underlying function is only used once and never shared, the statement "this
/// object can be shared between threads" is vacuously true.
unsafe impl<F> Sync for LazyInitWrapper<F> {}

/// SAFETY: This field is adds no extra requirements for the `Send` marker. It is functionally
/// equivalent to an `Option<F>` but with the caveat that the tag is stored externally.
unsafe impl<F: Send> Send for LazyInitWrapper<F> {}

impl<F> LazyInitWrapper<F> {
    const fn new(f: F) -> Self {
        Self(UnsafeCell::new(MaybeUninit::new(f)))
    }

    unsafe fn take(&self) -> F {
        // SAFETY: get_or_init only runs this function once, so this reference is unique
        let init = &mut *self.0.get();
        let init = core::mem::replace(init, MaybeUninit::uninit());
        init.assume_init()
    }
}

pub struct Lazy<T, F = fn() -> T>
{
    once: OnceCell<T>,
    init: LazyInitWrapper<F>,
}

impl<T, F> Lazy<T, F> {
    pub const fn new(init: F) -> Self {
        Self {
            once: OnceCell::new(),
            init: LazyInitWrapper::new(init),
        }
    }
}

impl<T, F> Lazy<T, F>
where
    F: FnOnce() -> T,
{
    /// Force the initialization to happen immediately.
    pub fn force(&self) {
        self.get();
    }

    pub fn get(&self) -> &T {
        self.once.get_or_init(move || {
            // SAFETY: Lazy always starts with the init field being initialized, and this closure
            // is only ever run once.
            (unsafe { self.init.take() })()
        })
    }
}

impl<T, F> Drop for Lazy<T, F> {
    fn drop(&mut self) {
        if core::mem::needs_drop::<F>() && !self.once.is_initialized() {
            // SAFETY: Lazy always starts with the init field being initialized
            unsafe { self.init.take() };
        }
    }
}
