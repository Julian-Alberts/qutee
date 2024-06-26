#![warn(missing_docs)]
#![warn(clippy::missing_errors_doc)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::missing_errors_doc)]
#![warn(unused_unsafe)]
#![warn(clippy::suspicious)]
#![warn(clippy::perf)]
//! qutee is a small create which implements a quad tree.
//! ```
//! use qutee::*;
//! // Create a new quadtree where the area's top left corner is at -10, -10, with a width and height of 20.
//! let mut tree = QuadTree::new_with_dyn_cap(Boundary::new((-10., -10.), 20., 20.), 5);
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
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct QuadTree<C, Item, Cap = DynCap>
where
    C: Coordinate,
{
    boundary: Boundary<C>,
    quadrants: Option<Box<[QuadTree<C, Item, Cap>; 4]>>,
    items: Option<Vec<(Point<C>, Item)>>,
    capacity: Cap,
}

/// Possible errors
#[derive(PartialEq, Eq, Clone)]
pub enum QuadTreeError<C>
where
    C: Coordinate,
{
    /// Point is out of bounds
    OutOfBounds(Boundary<C>, Point<C>),
}

/// This traits allows a type to be used with `qutee::QuadTree::insert`
pub trait AsPoint<C>
where
    C: Coordinate,
{
    /// Get the position of an item
    fn as_point(&self) -> Point<C>;
}

/// A point in two dimensional space
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Point<C>
where
    C: Coordinate,
{
    /// The x coordinate
    pub x: C,
    /// The y coordinate
    pub y: C,
}

impl<T> Point<T>
where
    T: Coordinate,
{
    /// Create a new point at a given x and y
    /// # Example
    /// ```
    /// use qutee::*;
    /// let x = 10;
    /// let y = 12;
    /// let p = Point::new(x,y);
    /// assert_eq!(p.x, x);
    /// assert_eq!(p.y, y);
    /// ```
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
    /// # Example
    /// ```
    /// use qutee::*;
    /// struct MyItem;
    /// let tree = QuadTree::<_, MyItem, DynCap>::new_with_capacity(Boundary::between_points((10,10), (20,20)), DynCap::new(20));
    /// assert_eq!(tree.capacity(), 20);
    /// ```
    pub fn new_with_capacity(boundary: Boundary<C>, capacity: Cap) -> Self {
        Self {
            boundary,
            quadrants: None,
            items: None,
            capacity,
        }
    }

    /// Insert new item into the quad tree.
    /// # Errors
    /// Returns an error if the point is out of bounds.
    /// # Example
    /// ```
    /// use qutee::*;
    /// let mut tree = QuadTree::<_,_,ConstCap<2>>::new_with_const_cap(Boundary::between_points((0,0), (10,10)));
    /// assert!(tree.insert_at((5,5), ()).is_ok());
    /// assert!(tree.insert_at((11,11), ()).is_err());
    /// ```
    pub fn insert_at(
        &mut self,
        point: impl Into<Point<C>>,
        value: Item,
    ) -> Result<(), QuadTreeError<C>> {
        let point = point.into();
        if !self.boundary.contains(&point) {
            return Err(QuadTreeError::OutOfBounds(self.boundary, point));
        }
        self.insert_at_unchecked(point, value);
        Ok(())
    }

    /// Same as `insert_at` except that no bounds check is performed.
    /// # Example
    /// ```
    /// use qutee::*;
    /// let mut tree = QuadTree::<_,_,ConstCap<2>>::new_with_const_cap(Boundary::between_points((0,0), (10,10)));
    /// tree.insert_at_unchecked((5,5), ());
    /// assert_eq!(tree.iter().count(), 1);
    /// ```
    pub fn insert_at_unchecked(&mut self, point: impl Into<Point<C>>, value: Item) {
        let mut sub_tree = self;
        let point = point.into();
        loop {
            if sub_tree.items.as_ref().map(|i| i.len()).unwrap_or_default()
                < sub_tree.capacity.capacity()
            {
                sub_tree
                    .items
                    .get_or_insert_with(|| Vec::with_capacity(sub_tree.capacity.capacity()))
                    .push((point, value));
                return;
            }
            let quads = sub_tree.quadrants.get_or_insert_with(|| {
                let [b0, b1, b2, b3] = sub_tree.boundary.split();
                Box::new([
                    QuadTree::new_with_capacity(b0, sub_tree.capacity),
                    QuadTree::new_with_capacity(b1, sub_tree.capacity),
                    QuadTree::new_with_capacity(b2, sub_tree.capacity),
                    QuadTree::new_with_capacity(b3, sub_tree.capacity),
                ])
            });

            let is_in_right_half = (quads[0].boundary.p2.x < point.x) as usize;
            let is_in_bottom_half = (quads[0].boundary.p2.y < point.y) as usize;
            let index = is_in_bottom_half << 1 | is_in_right_half;
            sub_tree = &mut quads[index];
        }
    }

    /// Get all items in a given area.
    /// # Example
    /// ```
    /// use qutee::*;
    /// let mut tree = QuadTree::<_,_,ConstCap<2>>::new_with_const_cap(Boundary::between_points((0,0), (10,10)));
    /// tree.insert_at((3,5), 1);
    /// tree.insert_at((1,0), 2);
    /// tree.insert_at((7,3), 4);
    /// tree.insert_at((9,4), 5);
    /// let mut res = tree.query(Boundary::between_points((2,1), (8,9))).copied().collect::<Vec<_>>();
    /// res.sort();
    /// assert_eq!(res, vec![1,4]);
    /// ```
    pub fn query<A>(&self, area: A) -> Query<'_, C, A, Item, Cap>
    where
        A: Area<C>,
    {
        Query::new(self, area)
    }

    /// Get all items in a given area and their coordinates.
    /// # Example
    /// ```
    /// use qutee::*;
    /// let mut tree = QuadTree::<_,_,ConstCap<2>>::new_with_const_cap(Boundary::between_points((0,0), (10,10)));
    /// tree.insert_at((3,5), 1);
    /// tree.insert_at((1,0), 2);
    /// tree.insert_at((7,3), 4);
    /// tree.insert_at((9,4), 5);
    /// let mut res = tree.query_points(Boundary::between_points((2,1), (8,9))).copied().collect::<Vec<_>>();
    /// res.sort_by(|a,b| a.1.cmp(&b.1));
    /// assert_eq!(res, vec![
    ///     ((3,5).into(), 1),
    ///     ((7,3).into(), 4),
    /// ]);
    /// ```
    pub fn query_points<A>(&self, area: A) -> QueryPoints<'_, C, A, Item, Cap>
    where
        A: Area<C>,
    {
        QueryPoints::new(self, area)
    }

    /// Get an iterator over all items.
    pub fn iter(&self) -> Iter<'_, C, Item, Cap> {
        Iter::new(self)
    }

    /// Get an iterator over all items and their coordinates.
    pub fn iter_points(&self) -> IterPoints<'_, C, Item, Cap> {
        IterPoints::new(self)
    }

    /// Returns the boundary of this QuadTree
    pub fn boundary(&self) -> &Boundary<C> {
        &self.boundary
    }

    /// Returns the capacity
    pub fn capacity(&self) -> usize {
        self.capacity.capacity()
    }
}

impl<C, Item, Cap> QuadTree<C, Item, Cap>
where
    Cap: Capacity,
    C: Coordinate,
    Item: AsPoint<C>,
{
    /// Insert a new item
    /// # Errors
    /// Returns an error if the item is out of bounds.
    /// # Example
    /// ```
    /// use qutee::*;
    /// struct Item {
    ///     x: usize,
    ///     y: usize,
    /// }
    /// impl AsPoint<usize> for Item {
    ///     fn as_point(&self) -> Point<usize> {
    ///         (self.x, self.y).into()
    ///     }
    /// }
    /// let mut quad_tree = QuadTree::new_with_dyn_cap(Boundary::between_points((0,0),(10,10)), 5);
    /// assert!(quad_tree.insert(Item {
    ///     x: 5,
    ///     y: 5,
    /// }).is_ok());
    /// ```
    pub fn insert(&mut self, item: Item) -> Result<(), QuadTreeError<C>> {
        self.insert_at(item.as_point(), item)
    }

    /// Same as `insert` except that no bounds check is performed.
    pub fn insert_unchecked(&mut self, item: Item) {
        self.insert_at_unchecked(item.as_point(), item)
    }
}

impl<C, Item> QuadTree<C, Item, DynCap>
where
    C: Coordinate,
{
    /// Create a new QuadTree
    pub fn new_with_dyn_cap(boundary: Boundary<C>, cap: usize) -> Self {
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
                items: None,
                capacity: ConstCap,
            },
            tree
        );
        assert_eq!(None, tree.items)
    }

    #[test]
    fn insert_single() {
        let mut tree = QuadTree::new_with_dyn_cap(Boundary::new((0, 0), 10, 10), 10);
        assert!(tree.insert_at((10, 10), 1u8).is_ok());
        assert_eq!(tree.items.unwrap()[0], ((10, 10).into(), 1));
    }

    #[test]
    fn insert_out_of_bounds() {
        let mut tree = QuadTree::new_with_dyn_cap(Boundary::new((0, 0), 10, 10), 10);
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
        let mut tree = QuadTree::new_with_dyn_cap(Boundary::new((0, 0), 10, 10), 1);
        assert!(tree.quadrants.is_none());

        assert!(tree.insert_at((1, 1), 1).is_ok());
        assert!(tree.quadrants.is_none());
        assert_eq!(tree.items.as_ref().unwrap().len(), 1);

        assert!(tree.insert_at((2, 2), 1).is_ok());
        assert_eq!(tree.items.as_ref().unwrap().len(), 1);
        assert!(tree.quadrants.is_some());
        let quads = tree.quadrants.as_ref().unwrap();
        assert_eq!(quads[0].items.as_ref().unwrap().len(), 1);
        assert_eq!(quads[1].items, None);
        assert_eq!(quads[2].items, None);
        assert_eq!(quads[3].items, None);

        assert!(tree.insert_at((7, 7), 1).is_ok());
        assert!(tree.quadrants.is_some());
        let quads = tree.quadrants.as_ref().unwrap();
        assert_eq!(quads[0].items.as_ref().unwrap().len(), 1);
        assert_eq!(quads[1].items, None);
        assert_eq!(quads[2].items, None);
        assert_eq!(quads[3].items.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn query() {
        let mut tree = QuadTree::new_with_dyn_cap(Boundary::new((-10, -10), 20, 20), 2);
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
        let mut tree = QuadTree::new_with_dyn_cap(Boundary::new((-10, -10), 20, 20), 2);
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
        let e = super::QuadTreeError::OutOfBounds(
            Boundary::between_points((1, 2), (2, 3)),
            (10, 20).into(),
        );
        assert_eq!(
            "point (10,20) is outside of area (1,2),(2,3)",
            format!("{:#?}", e)
        );
    }

    #[test]
    fn format_display_error() {
        let e = super::QuadTreeError::OutOfBounds(
            Boundary::between_points((1, 2), (2, 3)),
            (10, 20).into(),
        );
        assert_eq!(
            "point (10,20) is outside of area (1,2),(2,3)",
            format!("{}", e)
        );
    }

    #[test]
    fn insert_item() {
        struct TmpItem {
            x: usize,
            y: usize,
            content: &'static str,
        }
        impl super::AsPoint<usize> for TmpItem {
            fn as_point(&self) -> Point<usize> {
                (self.x, self.y).into()
            }
        }
        let mut qt =
            super::QuadTree::new_with_dyn_cap(Boundary::between_points((0, 0), (10, 10)), 5);
        assert!(qt
            .insert(TmpItem {
                x: 5,
                y: 5,
                content: "test"
            })
            .is_ok());
        let mut query = qt.query(Boundary::new((4, 4), 2, 2));
        assert_eq!(query.next().unwrap().content, "test");
    }
}
