pub trait Capacity: Clone + Copy {
    fn capacity(&self) -> usize;
}

/// A Capacity known at compile time
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct CompiletimeCap<const CAP: usize>;
impl<const CAP: usize> Capacity for CompiletimeCap<CAP> {
    #[inline]
    fn capacity(&self) -> usize {
        CAP
    }
}

/// A Capacity known at runtime
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct RuntimeCap(pub(super) usize);
impl Capacity for RuntimeCap {
    #[inline]
    fn capacity(&self) -> usize {
        self.0
    }
}
