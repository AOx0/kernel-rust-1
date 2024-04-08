use core::cell::UnsafeCell;
use core::sync::atomic::{fence, AtomicBool, Ordering};

#[derive(Debug)]
pub struct Mutex<T> {
	lock: AtomicBool,
	cell: UnsafeCell<T>,
}

#[derive(Debug)]
pub struct MutexGuard<'a, T> {
	mtx: &'a Mutex<T>,
}

impl<T> Drop for MutexGuard<'_, T> {
	fn drop(&mut self) { self.mtx.lock.store(false, Ordering::Release); }
}

impl<T> core::ops::Deref for MutexGuard<'_, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target { unsafe { &*self.mtx.cell.get() } }
}

impl<T> core::ops::DerefMut for MutexGuard<'_, T> {
	fn deref_mut(&mut self) -> &mut Self::Target { unsafe { &mut *self.mtx.cell.get() } }
}

impl<T> Mutex<T> {
	pub const fn new(value: T) -> Mutex<T> {
		Mutex { lock: AtomicBool::new(false), cell: UnsafeCell::new(value) }
	}

	pub fn lock(&self) -> MutexGuard<'_, T> {
		'block: loop {
			while self.lock.load(Ordering::Relaxed) {
				core::hint::spin_loop()
			}

			let stored =
				self.lock.compare_exchange_weak(false, true, Ordering::Relaxed, Ordering::Relaxed);

			match stored {
				Ok(_) => break 'block,
				Err(_) => continue,
			}
		}

		fence(Ordering::Acquire);
		MutexGuard { mtx: self }
	}
}

/// A reference to a single mutex can be shared between threads if the inner value `T` is sendable, thus implements `Send`
unsafe impl<T> Sync for Mutex<T> where T: Send {}

/// A mutex can be sent to other threads
unsafe impl<T> Send for Mutex<T> where T: Send {}
