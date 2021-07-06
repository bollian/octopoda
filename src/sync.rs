use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, AtomicU32, Ordering};
use core::mem::MaybeUninit;
use lock_api::RawMutex;
use spin::mutex::spin::SpinMutex as Spin;
use crate::defer::defer;
use crate::bsp::mmap;
use crate::driver;

pub type SpinMutex<T> = Mutex<Spin<()>, T>;
pub type SpinMutexMut<'a, T> = MutexMut<'a, Spin<()>, T>;

pub struct Mutex<R, T>
where
    R: lock_api::RawMutex,
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
        use ufmt::uwriteln;
        let mut gpio = unsafe { crate::driver::gpio::Gpio::new(crate::bsp::mmap::GPIO_BASE) };
        let mut uart = unsafe { crate::driver::uart::PL011Uart::new(crate::bsp::mmap::PL011_UART_BASE) };
        uart.init(&mut gpio, 921_600);
        let _ = uwriteln!(uart, "Unmanaged Hello 7");

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
    filled: AtomicU32,
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
            filled: AtomicU32::new(ONCE_CELL_UNFILLED),
            data: UnsafeCell::new(MaybeUninit::uninit()),
            _no_send_sync: core::marker::PhantomData,
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.filled.load(Ordering::Acquire) == ONCE_CELL_FILLED
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
        loop {
            match self.get() {
                Some(data) => return data,
                None => {
                    use ufmt::uwriteln;

                    let mut gpio = unsafe { driver::gpio::Gpio::new(mmap::GPIO_BASE) };
                    let mut uart = unsafe { driver::uart::PL011Uart::new(mmap::PL011_UART_BASE) };
                    uart.init(&mut gpio, 921_600);

                    let _ = uwriteln!(uart, "Unmanaged Hello 1");
                    let exchange = self.filled.compare_exchange_weak(
                        ONCE_CELL_UNFILLED,
                        ONCE_CELL_FILLING,
                        Ordering::Acquire,
                        Ordering::Acquire,
                    );
                    let _ = uwriteln!(uart, "Unmanaged Hello 4");

                    if exchange.is_err() {
                        continue // retest the lock
                    }

                    // SAFETY: because of the compare-exchange check, we know we're the only one
                    // modifying the data field
                    let filled_ref = unsafe {
                        self.data.get().write(MaybeUninit::new(f()));
                        self.get_unchecked()
                    };
                    self.filled.store(ONCE_CELL_FILLED, Ordering::Release);
                    return filled_ref
                }
            }
        }
    }

    pub unsafe fn get_unchecked(&self) -> &T {
        (*self.data.get()).assume_init_ref()
    }
}
