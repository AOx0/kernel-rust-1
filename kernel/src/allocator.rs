use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use x86_64::{
	structures::paging::{
		mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
	},
	VirtAddr,
};

use linked_list_allocator::LockedHeap;

#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();

// pub struct Allocator;

// unsafe impl GlobalAlloc for Allocator {
// 	unsafe fn alloc(&self, _layout: Layout) -> *mut u8 { null_mut() }

// 	unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
// 		panic!("dealloc should be never called")
// 	}
// }

pub const HEAP_START: u64 = 0x4444_4444;
/// 1 MiB
pub const HEAP_SIZE: u64 = 1 * (1024u64.pow(2u32));

pub fn init_alloc() {
	unsafe {
		ALLOCATOR.lock().init(HEAP_START as *mut u8, HEAP_SIZE as usize);
	}
}

pub fn init_heap(
	mapper: &mut impl Mapper<Size4KiB>,
	frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
	let page_range = {
		let start = VirtAddr::new(HEAP_START);
		let end = VirtAddr::new(HEAP_START + HEAP_SIZE - 1);
		let start_page = Page::containing_address(start);
		let end_page = Page::containing_address(end);
		Page::range_inclusive(start_page, end_page)
	};

	// crate::println!("Range: {:?}", page_range);

	for page in page_range {
		// crate::println!("Performing {:?}", page);
		let frame = frame_allocator.allocate_frame().ok_or(MapToError::FrameAllocationFailed)?;
		let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
		unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() }
	}

	Ok(())
}
