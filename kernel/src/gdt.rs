use core::ptr::addr_of;

use x86_64::instructions::tables::load_tss;
use x86_64::registers::segmentation::{Segment, CS, SS};
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

use crate::once_lock::OnceLock;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub static GDT: OnceLock<(GlobalDescriptorTable, Selectors)> = OnceLock::new();
pub static TSS: OnceLock<TaskStateSegment> = OnceLock::new();

#[derive(Debug)]
pub struct Selectors {
	code_selector: SegmentSelector,
	tss_selector: SegmentSelector,
	data_selector: SegmentSelector,
}

pub fn init_tss() -> TaskStateSegment {
	let mut tss = TaskStateSegment::new();
	tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
		const STACK_SIZE: usize = 4096 * 5;
		static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

		let stack_start = VirtAddr::from_ptr(unsafe { addr_of!(STACK) });
		stack_start + STACK_SIZE.try_into().unwrap()
	};
	tss
}

pub fn init_gdt() -> (GlobalDescriptorTable, Selectors) {
	let mut gdt = GlobalDescriptorTable::new();
	let code_selector = gdt.append(Descriptor::kernel_code_segment());
	let tss_selector = gdt.append(Descriptor::tss_segment(TSS.get().unwrap()));
	let data_selector = gdt.append(Descriptor::kernel_data_segment());
	(gdt, Selectors { code_selector, tss_selector, data_selector })
}

pub fn load_gdt() {
	let gdt = GDT.get().unwrap();
	gdt.0.load();
	unsafe {
		CS::set_reg(gdt.1.code_selector);
		SS::set_reg(gdt.1.data_selector);
		load_tss(gdt.1.tss_selector);
	};
}
