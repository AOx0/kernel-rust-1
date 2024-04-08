use uart_16550::SerialPort;

use crate::mutex::Mutex;
use crate::once_lock::OnceLock;

pub struct Port {
	base: usize,
	inner: SerialPort,
}

impl core::fmt::Debug for Port {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "Port {{ inner: SerialPort {{ base: {base} }} }}", base = self.base)
	}
}

impl core::ops::Deref for Port {
	type Target = SerialPort;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl core::ops::DerefMut for Port {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.inner }
}

pub static SERIAL1: OnceLock<Mutex<Port>> = OnceLock::new();

pub fn serial_init() -> Mutex<Port> {
	let mut serial_port = unsafe { uart_16550::SerialPort::new(0x3F8) };
	serial_port.init();
	Mutex::new(Port { inner: serial_port, base: 0x3F8 })
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
	use core::fmt::Write;

	use x86_64::instructions::interrupts;

	interrupts::without_interrupts(|| {
		SERIAL1.get().unwrap().lock().write_fmt(args).expect("Printing to serial failed");
	});
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}
