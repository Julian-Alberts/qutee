pub trait Capacity: Clone + Copy {
    fn capacity(&self) -> usize;
}

/// A Capacity known at compile time
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ConstCap<const CAP: usize>;
impl<const CAP: usize> Capacity for ConstCap<CAP> {
    #[inline]
    fn capacity(&self) -> usize {
        CAP
    }
}

/// A Capacity known at runtime
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DynCap(pub(super) usize);
impl Capacity for DynCap {
    #[inline]
    fn capacity(&self) -> usize {
        self.0
    }
}
