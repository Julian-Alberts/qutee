#![warn(missing_docs)]
//! qutee is a small create which implements a quad tree.
//! ```
//! use qutee::*;
//! // Create a new quadtree where the area's top left corner is at -10, -10, with a width and height of 20.
//! let mut tree = QuadTree::new_with_runtime_cap(Boundary::new((-10., -10.), 20., 20.), 5);
//! assert!(tree.insert_at((0.5, 0.1), "A").is_ok());
//! assert!(tree.insert_at((-1., 1.), "B").is_ok());
//! // This point is outside the tree
//! assert!(tree.insert_at((10.1, 5.), "C").is_err());
//! // Search elements inside a boundary. A boundary can also be defined as an area between two points.
//! let mut query = tree.query(Boundary::between_points((0.,0.),(1.,1.)));
//! assert_eq!(query.next(), Some(&"A"));
//! assert!(query.next().is_none());
//! // Get an iterator over all items
//! let mut iter = tree.iter();
//! assert_eq!(iter.next(), Some(&"A"));
//! assert_eq!(iter.next(), Some(&"B"));
//! assert!(iter.next().is_none());
//! ```

mod boundary;
mod bounds;
mod iter;

use std::{
    error::Error,
    fmt::{Debug, Display},
};

pub use boundary::*;
use bounds::Capacity;
pub use bounds::{ConstCap, DynCap};
pub use iter::*;

///
/// # Parameter
/// C: The type used for coordinates
/// Item: The type to be saved
/// CAP: The maximum capacity of each level
#[derive(PartialEq, Debug)]
pub struct QuadTree<C, Item, Cap = DynCap>
where
    C: Coordinate,
{
    boundary: Boundary<C>,
    quadrants: Option<Box<[QuadTree<C, Item, Cap>; 4]>>,
    items: Vec<(Point<C>, Item)>,
    capacity: Cap,
}

/// Possible errors
#[derive(PartialEq)]
pub enum QuadTreeError<C>
where
    C: Coordinate,
{
    /// Point is out of bounds
    OutOfBounds(Boundary<C>, Point<C>),
}

/// A point in two dimensional space
#[derive(Debug, PartialEq, Clone)]
pub struct Point<C>
where
    C: Coordinate,
{
    x: C,
    y: C,
}

impl<T> Point<T>
where
    T: Coordinate,
{
    /// Create a new point
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<C, Item, Cap> QuadTree<C, Item, Cap>
where
    Cap: Capacity,
    C: Coordinate,
{
    /// Create a new quad tree for a given area where each level of the tree has a given capacity.
    pub fn new_with_capacity(boundary: Boundary<C>, capacity: Cap) -> Self {
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
        point: impl Into<Point<C>>,
        value: Item,
    ) -> Result<(), QuadTreeError<C>> {
        let point = point.into();
        if !self.boundary.contains(&point) {
            return Err(QuadTreeError::OutOfBounds(self.boundary.clone(), point));
        }
        if self.quadrants.is_none() && self.items.len() >= self.capacity.capacity() {
            let [b0, b1, b2, b3] = self.boundary.split();
            self.quadrants = Some(Box::new([
                QuadTree::new_with_capacity(b0, self.capacity),
                QuadTree::new_with_capacity(b1, self.capacity),
                QuadTree::new_with_capacity(b2, self.capacity),
                QuadTree::new_with_capacity(b3, self.capacity),
            ]));
        }
        if let Some(quads) = &mut self.quadrants {
            let sub_tree = quads
                .iter_mut()
                .find(|tree| tree.boundary.contains(&point))
                .expect("Tree did not split correctly");
            return sub_tree.insert_at(point, value);
        }
        self.items.push((point, value));
        Ok(())
    }

    /// Get all items in a given area.
    pub fn query(&self, boundary: Boundary<C>) -> Query<'_, C, Item, Cap> {
        Query::new(self, boundary)
    }

    /// Get an iterator over all items.
    pub fn iter(&self) -> Iter<'_, C, Item, Cap> {
        Iter::new(self)
    }
}

impl<C, Item> QuadTree<C, Item, DynCap>
where
    C: Coordinate,
{
    /// Create a new QuadTree
    pub fn new_with_runtime_cap(boundary: Boundary<C>, cap: usize) -> Self {
        Self::new_with_capacity(boundary, DynCap(cap))
    }
}

impl<C, Item, const CAP: usize> QuadTree<C, Item, ConstCap<CAP>>
where
    C: Coordinate,
{
    /// Create a new QuadTree with a constant capacity
    pub fn new_with_const_cap(boundary: Boundary<C>) -> Self {
        let capacity = ConstCap;
        Self::new_with_capacity(boundary, capacity)
    }
}

impl<C> From<(C, C)> for Point<C>
where
    C: Coordinate,
{
    fn from((x, y): (C, C)) -> Self {
        Point { x, y }
    }
}

impl<C> Display for Point<C>
where
    C: Coordinate,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:?},{:?})", self.x, self.y)
    }
}

impl<C> Error for QuadTreeError<C> where C: Coordinate {}

impl<C> Display for QuadTreeError<C>
where
    C: Coordinate,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

impl<C> Debug for QuadTreeError<C>
where
    C: Coordinate,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutOfBounds(boundary, point) => {
                write!(f, "point {point} is outside of area {boundary}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{bounds::ConstCap, Boundary, Coordinate, Point, QuadTree, QuadTreeError};

    #[test]
    fn create_quad_tree() {
        let boundary = Boundary::new((0, 0), 10, 10);
        let tree = QuadTree::<usize, u8, ConstCap<20>>::new_with_const_cap(boundary.clone());
        assert_eq!(
            QuadTree {
                boundary,
                quadrants: None,
                items: Vec::new(),
                capacity: ConstCap,
            },
            tree
        );
        assert_eq!(20, tree.items.capacity())
    }

    #[test]
    fn insert_single() {
        let mut tree = QuadTree::new_with_runtime_cap(Boundary::new((0, 0), 10, 10), 10);
        assert!(tree.insert_at((10, 10), 1u8).is_ok());
        assert_eq!(tree.items[0], ((10, 10).into(), 1));
    }

    #[test]
    fn insert_out_of_bounds() {
        let mut tree = QuadTree::new_with_runtime_cap(Boundary::new((0, 0), 10, 10), 10);
        assert_eq!(
            tree.insert_at((20, 20), 1u8),
            Err(QuadTreeError::OutOfBounds(
                Boundary::new((0, 0), 10, 10),
                (20, 20).into()
            ))
        );
    }

    #[test]
    fn insert_more_than_capacity() {
        let mut tree = QuadTree::new_with_runtime_cap(Boundary::new((0, 0), 10, 10), 1);
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
        let mut tree = QuadTree::new_with_runtime_cap(Boundary::new((-10, -10), 20, 20), 2);
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
        let mut tree = QuadTree::new_with_runtime_cap(Boundary::new((-10, -10), 20, 20), 2);
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
        Pu: Coordinate,
    {
        (x, y).into()
    }

    #[test]
    fn format_debug_error() {
        let e = super::QuadTreeError::OutOfBounds(Boundary::between_points((1,2), (2,3)), (10,20).into());
        assert_eq!("point (10,20) is outside of area (1,2),(2,3)", format!("{:#?}", e));
    }

    #[test]
    fn format_display_error() {
        let e = super::QuadTreeError::OutOfBounds(Boundary::between_points((1,2), (2,3)), (10,20).into());
        assert_eq!("point (10,20) is outside of area (1,2),(2,3)", format!("{}", e));
    }
}
