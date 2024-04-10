#![no_std]
#![no_main]

extern crate alloc;

use core::panic::PanicInfo;

use bootloader_api::BootInfo;
use x86_64::{structures::paging::Translate, VirtAddr};

#[cfg(feature = "serial")]
use kernel::serial_println;
use kernel::{frame::WRITER, mem, println};

const CONFIG: bootloader_api::BootloaderConfig = {
	let mut config = bootloader_api::BootloaderConfig::new_default();
	// config.kernel_stack_size = 100 * 1024; // 100 KiB
	config.mappings.physical_memory = Some(bootloader_api::config::Mapping::Dynamic);
	config
};
bootloader_api::entry_point!(kernel_main, config = &CONFIG);

fn kernel_main(
	BootInfo { memory_regions, framebuffer, physical_memory_offset, .. }: &'static mut BootInfo,
) -> ! {
	let frameinfo = framebuffer.as_ref().unwrap().info();
	let framebuffer = framebuffer.as_mut().unwrap();

	let mem_offset = physical_memory_offset.into_option().map(VirtAddr::new).unwrap();
	let mut mapper = unsafe { mem::init(mem_offset) };
	let mut frame_allocator = unsafe { mem::BootInfoFrameAllocator::init(memory_regions) };
	kernel::init(framebuffer, &mut mapper, &mut frame_allocator);

	let mut total_size = 0;
	let mut regions = 0;
	let mut pages = 0;
	for region in frame_allocator.usable_regions() {
		let size = region.end - region.start;
		pages += size / (1024 * 4);
		regions += 1;
		total_size += size;
	}
	println!(
		"Memory Regions: {:?}, pages: {:?}, size: {:.4?} GiB",
		regions,
		pages,
		total_size as f32 / (1024 * 1024 * 1024) as f32
	);

	let (size, width) = {
		let writer = &WRITER.get().unwrap().lock();
		(writer.raster_size(), writer.raster_width())
	};
	println!("Writer: {{ size: {size}, width: {width} }}");
	println!("Frame Info: {frameinfo:#?}");
	println!("Offset: {:?}", mem_offset);

	// allocate a number on the heap
	let heap_value = alloc::boxed::Box::new(41);
	println!("heap_value at {:p}", heap_value);

	// create a dynamically sized vector
	let mut vec = alloc::vec::Vec::new();
	for i in 0..500 {
		vec.push(i);
	}
	println!("vec at {:p}", vec.as_slice());

	let reference_counted = alloc::rc::Rc::new(alloc::vec![1, 2, 3]);
	let cloned_reference = reference_counted.clone();
	println!("current reference count is {}", alloc::rc::Rc::strong_count(&cloned_reference));
	core::mem::drop(reference_counted);
	println!("reference count is {} now", alloc::rc::Rc::strong_count(&cloned_reference));

	#[cfg(feature = "serial")]
	serial_println!("Hello World{}", "!");

	kernel::hlt_loop()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	println!("{}", info);
	kernel::hlt_loop()
}
