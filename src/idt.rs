use spin::LazyLock;
use x86_64::structures::idt::InterruptDescriptorTable;

// TODO: maybe we should actually use this
static IDT: LazyLock<InterruptDescriptorTable> = LazyLock::new(InterruptDescriptorTable::new);

pub fn init_idt() {
    IDT.load();
}
