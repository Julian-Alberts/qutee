pub trait Capacity: Clone + Copy {
    fn capacity(&self) -> usize;
}

/// A Capacity known at compile time
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct ConstCap<const CAP: usize>;
impl<const CAP: usize> Capacity for ConstCap<CAP> {
    #[inline]
    fn capacity(&self) -> usize {
        CAP
    }
}

/// A Capacity known at runtime
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct DynCap(pub(super) usize);

impl DynCap {
    /// Create a new DynCap
    pub fn new(cap: usize) -> Self {
        Self(cap)
    }
}

impl Capacity for DynCap {
    #[inline]
    fn capacity(&self) -> usize {
        self.0
    }
}
