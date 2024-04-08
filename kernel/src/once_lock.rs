use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicU8, Ordering};

pub struct OnceLock<T: Send> {
	cell: UnsafeCell<MaybeUninit<T>>,
	state: AtomicU8,
}

impl<T: Send> OnceLock<T> {
	const EMPTY: u8 = 0;
	const FILLED: u8 = 2;
	const WRITING: u8 = 1;

	pub const fn new() -> OnceLock<T> {
		OnceLock { state: AtomicU8::new(Self::EMPTY), cell: UnsafeCell::new(MaybeUninit::uninit()) }
	}

	pub fn set(&self, value: T) -> Result<(), T> {
		let stored = self.state.compare_exchange(
			Self::EMPTY,
			Self::WRITING,
			Ordering::Relaxed,
			Ordering::Relaxed,
		);

		match stored {
			Ok(_) => {
				unsafe { &mut *self.cell.get() }.write(value);
				self.state.store(Self::FILLED, Ordering::Release);
				Ok(())
			}
			Err(_) => Err(value),
		}
	}

	pub fn get(&self) -> Option<&T> {
		(self.state.load(Ordering::Acquire) == Self::FILLED)
			.then(|| unsafe { { &*self.cell.get() }.assume_init_ref() })
	}
}

impl<T: Send> Default for OnceLock<T> {
	fn default() -> Self { Self::new() }
}

impl<T: Send> Drop for OnceLock<T> {
	fn drop(&mut self) {
		while self.state.load(Ordering::Relaxed) == Self::WRITING {
			core::hint::spin_loop()
		}

		if self.state.load(Ordering::Acquire) == Self::FILLED {
			drop(unsafe { { &*self.cell.get() }.assume_init_read() })
		}
	}
}

/// A OnceLock may be sent to other threads only if the inner value `T` is sendable
unsafe impl<T> Send for OnceLock<T> where T: Send {}

/// A reference to a OnceLock may be shared between threads only if the inner value T is shareable and sendable
unsafe impl<T> Sync for OnceLock<T> where T: Send + Sync {}
