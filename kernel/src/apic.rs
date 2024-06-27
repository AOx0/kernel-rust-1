// use acpi::{AcpiTables, PhysicalMapping};
// use x86_64::VirtAddr;

// use crate::once_lock::OnceLock;

// #[repr(C)]
// enum Rspd {
// 	Version1(RsdpV1),
// 	Version2(RsdpV2),
// }

// #[repr(C)]
// #[repr(packed)]
// struct RsdpV1 {
// 	signature: [u8; 8],
// 	checksumm: u8,
// 	oemid: [u8; 6],
// 	revision: u8,
// 	rsdt_address: u32,
// }

// #[repr(C)]
// struct RsdpV2 {
// 	signature: [u8; 8],
// 	checksumm: u8,
// 	oemid: [u8; 6],
// 	revision: u8,
// 	rsdt_address: u32,
// 	length: u32,
// 	xsdt_address: u64,
// 	extended_checksum: u8,
// 	reserved: [u8; 3],
// }

use core::{alloc::Layout, ops::DerefMut};

use acpi::AcpiTables;
use alloc::sync::Arc;
use x86_64::structures::paging::{FrameAllocator, Mapper, OffsetPageTable, Page, PageTableFlags, Size4KiB};

use crate::{allocator::ALLOCATOR, mutex::Mutex, once_lock::OnceLock};

#[derive(Clone)]
struct AcpiHandler<M, FA>
where FA: FrameAllocator<Size4KiB>,
    M: Mapper<Size4KiB>
  {
    offset: &'static OffsetPageTable<'static>,
    mapper: Arc<Mutex<M>>,
    frame_allocator: Arc<Mutex<FA>>  
}

impl<M: Mapper<Size4KiB> + Clone, FA: FrameAllocator<Size4KiB> + Clone> acpi::AcpiHandler for AcpiHandler<M, FA> {
    unsafe fn map_physical_region<T>(&self, physical_address: usize, size: usize) -> acpi::PhysicalMapping<Self, T> {
        let address = self.offset.phys_offset() + physical_address as u64;
        let page = Page::containing_address(address);
		let frame = self.frame_allocator.lock().allocate_frame().unwrap();
		let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
		unsafe { self.mapper.lock().map_to(page, frame, flags, self.frame_allocator.lock().deref_mut()).unwrap().flush() };

		acpi::PhysicalMapping::new(physical_address, address, Size4KiB, 1,  )
    }

    fn unmap_physical_region<T>(region: &acpi::PhysicalMapping<Self, T>) {
        todo!()
    }
}

static RSPD: OnceLock<AcpiTables<AcpiHandler>> = OnceLock::new();

pub fn init_rspd(addr: VirtAddr) {
	unsafe { AcpiTables::from_rsdp(PhysicalMapping::new(addr.as_u64(), , , , ), addr.as_u64()) }
}
