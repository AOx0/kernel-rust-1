#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(iter_array_chunks)]

use x86_64::structures::paging::{FrameAllocator, Mapper, Size4KiB};

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

pub fn init(
	framebuffer: &'static mut bootloader_api::info::FrameBuffer,
	mapper: &mut impl Mapper<Size4KiB>,
	frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
	let _ = frame::WRITER.set(frame::init_framebuffer(framebuffer));
	#[cfg(feature = "serial")]
	serial::SERIAL1.set(serial::serial_init()).expect("Single entry point");

	println!("{}", version::VERSION);

	println!("KEYBD...");
	let Ok(_) = interrupts::KEYBOARD.set(interrupts::init_kbd()) else {
		panic!("Failed interrupts::init_kbd")
	};

	println!("GDT...");
	gdt::TSS.set(gdt::init_tss()).unwrap();
	gdt::GDT.set(gdt::init_gdt()).unwrap();
	gdt::load_gdt();

	println!("Interrupts...");
	interrupts::IDT.set(interrupts::init_idt()).unwrap();
	interrupts::load_idt();
	interrupts::init_pics();
	x86_64::instructions::interrupts::enable();

	println!("Heap...");
	allocator::init_heap(mapper, frame_allocator).unwrap();
	allocator::init_alloc();
	println!("Done!");
}

#[inline(always)]
pub fn hlt_loop() -> ! {
	loop {
		x86_64::instructions::hlt();
	}
}
