mod dispatch;
mod page_table;
mod frame;


use core::fmt::Debug;

pub use dispatch::*;
pub use page_table::*;
pub use frame::*;

pub trait Environment: Sized + Debug {
    type PageTable: PageTable + Debug;
    type Frame: Frame + Clone + Debug;
    type Dispatch: Dispatch<Self::Frame>;
    fn wfi_program() -> &'static [u8];
}
