use alloc::format;
use spin::LazyLock;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::misc::fatal_fail;

static IDT: LazyLock<InterruptDescriptorTable> = LazyLock::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    // TODO: we could add a handler for breakpoints here if needed later on
    idt.page_fault.set_handler_fn(page_fault_handler);
    idt.general_protection_fault
        .set_handler_fn(general_protection_fault_handler);

    idt
});

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: x86_64::structures::idt::PageFaultErrorCode,
) {
    let cr2 = x86_64::registers::control::Cr2::read().expect("no cr2?");

    fatal_fail(&format!(
        "[red]------------------- PAGE FAULT -------------------[white]\ncr2={cr2:#X} err={error_code:#?}\n{stack_frame:#?}\n[red]--------------------------------------------------[white]"
    ));
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    fatal_fail(&format!(
        "[red]------------ GENERAL PROTECTION FAULT ------------[white]\nerr={error_code:#?}\n{stack_frame:#?}\n[red]--------------------------------------------------[white]"
    ));
}
