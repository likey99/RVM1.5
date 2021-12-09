use libvmm::msr::Msr;
use x86::{segmentation, segmentation::SegmentSelector};

use super::cpuid::CpuId;
use super::tables::{GDTStruct, GDT, IDT};

pub fn phys_id() -> usize {
    CpuId::new()
        .get_feature_info()
        .unwrap()
        .initial_local_apic_id() as usize
}

pub fn frequency() -> u16 {
    static CPU_FREQUENCY: spin::Once<u16> = spin::Once::new();
    *CPU_FREQUENCY.call_once(|| {
        const DEFAULT: u16 = 4000;
        CpuId::new()
            .get_processor_frequency_info()
            .map(|info| info.processor_base_frequency())
            .unwrap_or(DEFAULT)
            .max(DEFAULT)
    })
}

pub fn current_cycle() -> u64 {
    unsafe { core::arch::x86_64::_rdtsc() }
}

pub fn current_time_nanos() -> u64 {
    current_cycle() * 1000 / frequency() as u64
}

/// Reset CPU states for hypervisor use.
pub fn init() {
    // Setup new GDT, IDT, CS, TSS
    GDT.lock().load();
    unsafe {
        segmentation::load_cs(GDTStruct::KCODE_SELECTOR);
        segmentation::load_ds(SegmentSelector::from_raw(0));
        segmentation::load_es(SegmentSelector::from_raw(0));
        segmentation::load_ss(SegmentSelector::from_raw(0));
    }
    IDT.lock().load();
    GDT.lock().load_tss(GDTStruct::TSS_SELECTOR);

    // PAT0: WB, PAT1: WC, PAT2: UC
    unsafe { Msr::IA32_PAT.write(0x070106) };
}
