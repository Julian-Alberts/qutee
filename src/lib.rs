#![warn(missing_docs)]
//! qutee is a small create which implements a quad tree.
//! ```
//! use qutee::*;
//! type QuadTree<Cord, Item> = qutee::QuadTree<Cord, Item, RunTimeCap>;
//! let mut tree = QuadTree::new(Boundary::new((-10., -10.), 20., 20.), 5);
//! assert!(tree.insert_at((0.5, 0.1), "A").is_ok());
//! assert!(tree.insert_at((-1., 1.), "B").is_ok());
//! assert_eq!(tree.insert_at((10.1, 5.), "C"), Err(QuadTreeError::OutOfBounds));
//! let mut query = tree.query(Boundary::between_points((0.,0.),(1.,1.)));
//! assert_eq!(query.next(), Some(&"A"));
//! assert!(query.next().is_none());
//! let mut iter = tree.iter();
//! assert_eq!(iter.next(), Some(&"A"));
//! assert_eq!(iter.next(), Some(&"B"));
//! assert!(iter.next().is_none());
//! ```

mod boundary;
mod bounds;
mod iter;

use std::fmt::Debug;

pub use boundary::*;
use bounds::Capacity;
pub use bounds::{CompileTimeCap, RunTimeCap};
pub use iter::*;

///
/// # Parameter
/// PU: The type used for coordinates
/// Item: The type to be saved
/// CAPACITY: The maximum capacity of each level
#[derive(PartialEq, Debug)]
pub struct QuadTree<PU, Item, Cap = RunTimeCap>
where
    PU: PositionUnit,
{
    boundary: Boundary<PU>,
    quadrants: Option<Box<[QuadTree<PU, Item, Cap>; 4]>>,
    items: Vec<(Point<PU>, Item)>,
    capacity: Cap,
}

/// Possible errors
#[derive(Debug, PartialEq)]
pub enum QuadTreeError {
    /// Point is out of bounds
    OutOfBounds,
}

/// A point in two dimensional space
#[derive(Debug, PartialEq, Clone)]
pub struct Point<T>
where
    T: PositionUnit,
{
    x: T,
    y: T,
}

impl<PU, Item, Cap> QuadTree<PU, Item, Cap>
where
    Cap: Capacity,
    PU: PositionUnit,
{
    /// Create a new quad tree for a given area where each level of the tree has a given capacity.
    pub fn new_with_capacity(boundary: Boundary<PU>, capacity: Cap) -> Self {
        Self {
            boundary,
            quadrants: None,
            items: Vec::with_capacity(capacity.capacity()),
            capacity,
        }
    }

    /// Insert new item into the quad tree.
    pub fn insert_at(
        &mut self,
        point: impl Into<Point<PU>>,
        value: Item,
    ) -> Result<(), QuadTreeError> {
        let point = point.into();
        if !self.boundary.contains(&point) {
            return Err(QuadTreeError::OutOfBounds);
        }
        if self.quadrants.is_none() && self.items.len() >= self.capacity.capacity() {
            let Ok(quadrants) = self.boundary
                .split()
                .into_iter()
                .map(|b| QuadTree::new_with_capacity(b, self.capacity))
                .collect::<Vec<_>>()
                .try_into() else {
                    unreachable!("Boundary did not split into 4")
                };
            self.quadrants = Some(quadrants);
        }
        if let Some(quads) = &mut self.quadrants {
            let Some(sub_tree) = quads.iter_mut().find(|tree| tree.boundary.contains(&point)) else {
                return Err(QuadTreeError::OutOfBounds)
            };
            return sub_tree.insert_at(point, value);
        }
        self.items.push((point, value));
        Ok(())
    }

    /// Get all items in a given area.
    pub fn query(&self, boundary: Boundary<PU>) -> Query<'_, PU, Item, Cap> {
        Query::new(self, boundary)
    }

    /// Get an iterator over all items.
    pub fn iter(&self) -> Iter<'_, PU, Item, Cap> {
        Iter::new(self)
    }
}

impl<PU, Item> QuadTree<PU, Item, RunTimeCap>
where
    PU: PositionUnit,
{
    /// Create a new QuadTree
    pub fn new(boundary: Boundary<PU>, cap: usize) -> Self {
        let capacity = RunTimeCap(cap);
        Self {
            boundary,
            quadrants: None,
            items: Vec::with_capacity(capacity.capacity()),
            capacity,
        }
    }
}

impl<PU, Item, const CAP: usize> QuadTree<PU, Item, CompileTimeCap<CAP>>
where
    PU: PositionUnit,
{
    /// Create a new QuadTree with a constant capacity
    pub fn new_with_const_cap(boundary: Boundary<PU>) -> Self {
        let capacity = CompileTimeCap;
        Self::new_with_capacity(boundary, capacity)
    }
}

impl<Pu> From<(Pu, Pu)> for Point<Pu>
where
    Pu: PositionUnit,
{
    fn from((x, y): (Pu, Pu)) -> Self {
        Point { x, y }
    }
}

#[cfg(test)]
mod tests {
    use crate::{bounds::CompileTimeCap, Boundary, Point, PositionUnit, QuadTree, QuadTreeError};

    #[test]
    fn create_quad_tree() {
        let boundary = Boundary::new((0, 0), 10, 10);
        let tree = QuadTree::<usize, u8, CompileTimeCap<20>>::new_with_const_cap(boundary.clone());
        assert_eq!(
            QuadTree {
                boundary,
                quadrants: None,
                items: Vec::new(),
                capacity: CompileTimeCap
            },
            tree
        );
        assert_eq!(20, tree.items.capacity())
    }

    #[test]
    fn insert_single() {
        let mut tree = QuadTree::new(Boundary::new((0, 0), 10, 10), 10);
        assert!(tree.insert_at((10, 10), 1u8).is_ok());
        assert_eq!(tree.items[0], ((10, 10).into(), 1));
    }

    #[test]
    fn insert_out_of_bounds() {
        let mut tree = QuadTree::new(Boundary::new((0, 0), 10, 10), 10);
        assert_eq!(
            tree.insert_at((20, 20), 1u8),
            Err(QuadTreeError::OutOfBounds)
        );
    }

    #[test]
    fn insert_more_than_capacity() {
        let mut tree = QuadTree::new(Boundary::new((0, 0), 10, 10), 1);
        assert!(tree.quadrants.is_none());

        assert!(tree.insert_at((1, 1), 1).is_ok());
        assert!(tree.quadrants.is_none());
        assert_eq!(tree.items.len(), 1);

        assert!(tree.insert_at((2, 2), 1).is_ok());
        assert_eq!(tree.items.len(), 1);
        assert!(tree.quadrants.is_some());
        let quads = tree.quadrants.as_ref().unwrap();
        assert_eq!(quads[0].items.len(), 1);
        assert_eq!(quads[1].items.len(), 0);
        assert_eq!(quads[2].items.len(), 0);
        assert_eq!(quads[3].items.len(), 0);

        assert!(tree.insert_at((7, 7), 1).is_ok());
        assert!(tree.quadrants.is_some());
        let quads = tree.quadrants.as_ref().unwrap();
        assert_eq!(quads[0].items.len(), 1);
        assert_eq!(quads[1].items.len(), 0);
        assert_eq!(quads[2].items.len(), 0);
        assert_eq!(quads[3].items.len(), 1);
    }

    #[test]
    fn query() {
        let mut tree = QuadTree::new(Boundary::new((-10, -10), 20, 20), 2);
        let mut expected = Vec::new();
        for i in 1..10 {
            assert!(tree.insert_at((i, i), i).is_ok());
            assert!(tree.insert_at((-i, i), 0b1000_0000 | i).is_ok());
            assert!(tree.insert_at((i, -i), 0b0100_0000 | i).is_ok());
            assert!(tree.insert_at((-i, -i), 0b1100_0000 | i).is_ok());
            if i <= 2 {
                expected.push(i);
                expected.push(0b1000_0000 | i);
                expected.push(0b0100_0000 | i);
                expected.push(0b1100_0000 | i);
            }
        }
        let iter = tree.query(Boundary::new((-2, -2), 4, 4));
        for i in iter {
            expected.retain(|e| e != i)
        }
        assert!(expected.is_empty(), "items not found: {expected:?}")
    }

    #[test]
    fn iter() {
        let mut tree = QuadTree::new(Boundary::new((-10, -10), 20, 20), 2);
        let mut expected = Vec::new();
        for i in 1..10 {
            assert!(tree.insert_at((i, i), i).is_ok());
            assert!(tree.insert_at((-i, i), 0b1000_0000 | i).is_ok());
            assert!(tree.insert_at((i, -i), 0b0100_0000 | i).is_ok());
            assert!(tree.insert_at((-i, -i), 0b1100_0000 | i).is_ok());
            expected.push(i);
            expected.push(0b1000_0000 | i);
            expected.push(0b0100_0000 | i);
            expected.push(0b1100_0000 | i);
        }
        let iter = tree.iter();
        for i in iter {
            expected.retain(|e| e != i)
        }
        assert!(expected.is_empty(), "items not found: {expected:?}")
    }

    #[test_case::test_case(10, 10 => Point {x: 10, y: 10}; "int")]
    #[test_case::test_case(10., 10. => Point {x: 10., y: 10.}; "float")]
    fn tuple_to_point<Pu>(x: Pu, y: Pu) -> Point<Pu>
    where
        Pu: PositionUnit,
    {
        (x, y).into()
    }
}
