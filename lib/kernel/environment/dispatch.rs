use crate::kernel::{environment::frame::Frame, scheduler::Stack};

pub enum DispatchLevel {
    User,
    Kernel,
    Unchanged,
}

pub trait Dispatch<F: Frame> {
    unsafe fn dispatch(frame: &F) -> !;
    fn activate_irq();
    fn deactivate_irq();
    fn kernel_yield();
    fn irq_lock<T, N: FnOnce() -> T>(f: N) -> T;
}
