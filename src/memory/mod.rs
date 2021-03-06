use std::ops::Range;

use yaxpeax_arch::Address;

pub mod repr;
pub mod reader;

use memory::repr::flat::FlatMemoryRepr;
use memory::repr::cursor::{ReadCursor, UnboundedCursor};
use memory::repr::process::ModuleInfo;

#[derive(Debug)]
pub enum LayoutError {
    AddressConflict,
    Unsupported
}

pub trait MemoryRange<A: Address> where Self: MemoryRepr<A> {
    /// a cursor to read data contained in `range`. this willfully misinterprets `std::ops::Range`
    /// to be inclusive on both ends, rather than `[inclusive, exclusive)` as the docs say. this
    /// is, for the time being, necessary because yaxpeax consistently uses ranges that are
    /// inclusive on both ends. yaxpeax must represent `[inclusive, exclusive)` ranges most clearly
    /// because this significantly simplifies expressing a basic block that ends at the end of its
    /// architecture's address space.
    fn range<'a>(&'a self, range: Range<A>) -> Option<ReadCursor<'a, A, Self>>;
    fn range_from<'a>(&'a self, start: A) -> Option<UnboundedCursor<'a, A, Self>>;
}

pub trait MemoryRepr<A: Address>: Named {
    fn module_info(&self) -> Option<&ModuleInfo>;
    fn read(&self, addr: A) -> Option<u8>;
    fn to_flat(self) -> Option<FlatMemoryRepr>;
    fn module_for(&self, addr: A) -> Option<&dyn MemoryRepr<A>>;
    fn size(&self) -> Option<u64>;
    fn start(&self) -> Option<u64> { None }
    fn end(&self) -> Option<u64> { self.start().and_then(|x| self.size().map(|y| x + y)) }
}

pub trait Named {
    fn name(&self) -> &str;
}

pub trait PatchyMemoryRepr<A: Address> {
    fn add(&mut self, data: Vec<u8>, addr: A) -> Result<(), LayoutError>;
}
