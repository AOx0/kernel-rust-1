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

// static RSPD: OnceLock<AcpiTables<_>> = OnceLock::new();

// pub fn init_rspd(addr: VirtAddr) {
// 	unsafe { AcpiTables::from_rsdp(PhysicalMapping::new(addr.as_u64(), , , , ), addr.as_u64()) }
// }
