pub trait Capacity: Clone + Copy {
    fn capacity(&self) -> usize;
}

/// A Capacity known at compile time
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct CompileTimeCap<const CAP: usize>;
impl<const CAP: usize> Capacity for CompileTimeCap<CAP> {
    #[inline]
    fn capacity(&self) -> usize {
        CAP
    }
}

/// A Capacity known at runtime
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct RunTimeCap(pub(super) usize);
impl Capacity for RunTimeCap {
    #[inline]
    fn capacity(&self) -> usize {
        self.0
    }
}
