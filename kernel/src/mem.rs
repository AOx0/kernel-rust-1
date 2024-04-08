use bootloader_api::info::{MemoryRegion, MemoryRegionKind, MemoryRegions};
pub use x86_64::structures::paging::Page;
use x86_64::{
	registers::control::Cr3,
	structures::paging::{
		FrameAllocator, Mapper, OffsetPageTable, PageSize, PageTable, PhysFrame, Size4KiB,
	},
	PhysAddr, VirtAddr,
};

/// Return the VirtAddr for the Paging Table N. 4
///
/// # Safety
///
/// The caller must guarantee that the memory offset is valid
unsafe fn active_level_4_table(offset: VirtAddr) -> &'static mut PageTable {
	let (level_4_page_table, _) = Cr3::read();
	let ptr = offset + level_4_page_table.start_address().as_u64();
	&mut *ptr.as_mut_ptr()
}

/// .
///
/// # Safety
///
/// .
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
	let level_4_table = active_level_4_table(physical_memory_offset);
	OffsetPageTable::new(level_4_table, physical_memory_offset)
}

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator {
	memory_map: &'static MemoryRegions,
	next: usize,
}

impl BootInfoFrameAllocator {
	/// Create a FrameAllocator from the passed memory map.
	///
	/// # Safety
	///
	/// This function is unsafe because the caller must guarantee that the passed
	/// memory map is valid. The main requirement is that all frames that are marked
	/// as `USABLE` in it are really unused.
	pub unsafe fn init(memory_map: &'static MemoryRegions) -> Self {
		BootInfoFrameAllocator { memory_map, next: 0 }
	}

	pub fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
		self.usable_regions()
			.map(|region| region.start..region.end)
			.flat_map(|addr_range| addr_range.step_by(Size4KiB::SIZE as usize))
			.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
	}

	#[inline(always)]
	pub fn usable_regions(&self) -> impl Iterator<Item = MemoryRegion> {
		self.memory_map.iter().copied().filter(|region| region.kind == MemoryRegionKind::Usable)
	}
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
	fn allocate_frame(&mut self) -> Option<PhysFrame> {
		let frame = self.usable_frames().nth(self.next);
		self.next += 1;
		frame
	}
}
