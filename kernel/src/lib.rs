#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(iter_array_chunks)]

extern crate alloc;

pub mod allocator;
pub mod apic;
pub mod frame;
pub mod gdt;
pub mod interrupts;
pub mod mem;
pub mod mutex;
pub mod once_lock;
#[cfg(feature = "serial")]
pub mod serial;
pub mod version;

pub fn init(framebuffer: &'static mut bootloader_api::info::FrameBuffer) {
	let _ = frame::WRITER.set(frame::init_framebuffer(framebuffer));
	#[cfg(feature = "serial")]
	serial::SERIAL1.set(serial::serial_init()).expect("Single entry point");

	println!("{}", version::VERSION);

	let Ok(_) = interrupts::KEYBOARD.set(interrupts::init_kbd()) else {
		panic!("Failed interrupts::init_kbd")
	};

	gdt::TSS.set(gdt::init_tss()).unwrap();
	gdt::GDT.set(gdt::init_gdt()).unwrap();
	gdt::load_gdt();

	interrupts::IDT.set(interrupts::init_idt()).unwrap();
	interrupts::load_idt();
	interrupts::init_pics();
	x86_64::instructions::interrupts::enable();
}

#[inline(always)]
pub fn hlt_loop() -> ! {
	loop {
		x86_64::instructions::hlt();
	}
}
