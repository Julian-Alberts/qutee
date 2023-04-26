mod boundary;

use std::fmt::Debug;

pub use boundary::*;

#[derive(PartialEq, Debug)]
pub struct QuadTree<PU, Item, const CAPACITY: usize>
where
    PU: PositionUnit,
{
    boundary: Boundary<PU>,
    quadrants: Option<Box<[QuadTree<PU, Item, CAPACITY>; 4]>>,
    items: Vec<(Point<PU>, Item)>,
}

#[derive(Debug, PartialEq)]
pub enum QuadTreeError {
    OutOfBounds,
}

#[derive(Debug, PartialEq)]
pub struct Point<T>
where
    T: PositionUnit,
{
    x: T,
    y: T,
}

pub struct Query<'a, PU, Item, const CAPACITY: usize>
where
    PU: PositionUnit,
{
    quadrants: Option<Vec<&'a QuadTree<PU, Item, CAPACITY>>>,
    items: Vec<&'a (Point<PU>, Item)>,
    current_sub_query: Option<Box<Query<'a, PU, Item, CAPACITY>>>,
    boundary: Boundary<PU>,
}

impl<PU, Item, const CAPACITY: usize> QuadTree<PU, Item, CAPACITY>
where
    PU: PositionUnit,
    Item: Debug,
{
    pub fn new(boundary: Boundary<PU>) -> Self {
        Self {
            boundary,
            quadrants: None,
            items: Vec::with_capacity(CAPACITY),
        }
    }

    pub fn insert(&mut self, point: impl Into<Point<PU>>, value: Item) -> Result<(), QuadTreeError> {
        let point = point.into();
        if !self.boundary.contains(&point) {
            return Err(QuadTreeError::OutOfBounds);
        }
        if self.quadrants.is_none() && self.items.len() >= CAPACITY {
            self.quadrants = Some(
                self.boundary
                    .split()
                    .into_iter()
                    .map(|b| QuadTree::new(b))
                    .collect::<Vec<_>>()
                    .try_into()
                    .expect("Boundary did not split into 4"),
            );
        }
        if let Some(quads) = &mut self.quadrants {
            let Some(sub_tree) = quads.iter_mut().find(|tree| tree.boundary.contains(&point)) else {
                return Err(QuadTreeError::OutOfBounds)
            };
            return sub_tree.insert(point, value);
        }
        self.items.push((point, value));
        Ok(())
    }

    pub fn query<'a>(&'a self, boundary: Boundary<PU>) -> Query<'a, PU, Item, CAPACITY> {
        Query::new(self, boundary)
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

impl<'a, PU, Item, const CAPACITY: usize> Query<'a, PU, Item, CAPACITY>
where
    PU: PositionUnit,
    Item: Debug,
{
    fn new(tree: &'a QuadTree<PU, Item, CAPACITY>, boundary: Boundary<PU>) -> Self {
        Self {
            items: tree.items.iter().collect(),
            quadrants: tree.quadrants.as_ref().map(|q| q.iter().collect()),
            current_sub_query: None,
            boundary,
        }
    }

    fn find_next_quadrant(&mut self) -> Option<Box<Query<'a, PU, Item, CAPACITY>>> {
        let quadrants = self.quadrants.as_mut()?;
        if quadrants.is_empty() {
            return None;
        }
        while !quadrants.is_empty() {
            let q = quadrants.remove(0);
            if q.boundary.overlaps(&self.boundary) {
                return Some(Box::new(q.query(self.boundary.clone())));
            }
        }
        None
    }
}

impl<'a, PU, Item, const CAPACITY: usize> Iterator for Query<'a, PU, Item, CAPACITY>
where
    PU: PositionUnit,
    Item: Debug,
{
    type Item = &'a Item;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.items.is_empty() {
            let item = self.items.remove(0);
            if self.boundary.contains(&item.0) {
                return Some(&item.1);
            }
        }
        if self.current_sub_query.is_none() {
            self.current_sub_query = self.find_next_quadrant();
        }

        let Some(current_query) = self.current_sub_query.as_mut() else {
            return None;
        };

        if let Some(item) = current_query.next() {
            return Some(item);
        }

        self.current_sub_query = None;
        self.next()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Boundary, Point, PositionUnit, QuadTree, QuadTreeError};

    #[test]
    fn create_quad_tree() {
        let boundary = Boundary::new(0, 0, 10, 10);
        let tree = QuadTree::<usize, u8, 20>::new(boundary.clone());
        assert_eq!(
            QuadTree {
                boundary,
                quadrants: None,
                items: Vec::new(),
            },
            tree
        );
        assert_eq!(20, tree.items.capacity())
    }

    #[test]
    fn insert_single() {
        let mut tree = QuadTree::<usize, u8, 20>::new(Boundary::new(0, 0, 10, 10));
        assert!(tree.insert((10, 10), 1u8).is_ok());
        assert_eq!(tree.items[0], ((10, 10).into(), 1));
    }

    #[test]
    fn insert_out_of_bounds() {
        let mut tree = QuadTree::<usize, u8, 20>::new(Boundary::new(0, 0, 10, 10));
        assert_eq!(tree.insert((20, 20), 1u8), Err(QuadTreeError::OutOfBounds));
    }

    #[test]
    fn insert_more_than_capacity() {
        let mut tree = QuadTree::<usize, u8, 1>::new(Boundary::new(0, 0, 10, 10));
        assert!(tree.quadrants.is_none());

        assert!(tree.insert((1, 1), 1).is_ok());
        assert!(tree.quadrants.is_none());
        assert_eq!(tree.items.len(), 1);

        assert!(tree.insert((2, 2), 1).is_ok());
        assert_eq!(tree.items.len(), 1);
        assert!(tree.quadrants.is_some());
        let quads = tree.quadrants.as_ref().unwrap();
        assert_eq!(quads[0].items.len(), 1);
        assert_eq!(quads[1].items.len(), 0);
        assert_eq!(quads[2].items.len(), 0);
        assert_eq!(quads[3].items.len(), 0);

        assert!(tree.insert((7, 7), 1).is_ok());
        assert!(tree.quadrants.is_some());
        let quads = tree.quadrants.as_ref().unwrap();
        assert_eq!(quads[0].items.len(), 1);
        assert_eq!(quads[1].items.len(), 0);
        assert_eq!(quads[2].items.len(), 0);
        assert_eq!(quads[3].items.len(), 1);
    }

    #[test]
    fn query() {
        let mut tree = QuadTree::<_, _, 2>::new(Boundary::new(-10, -10, 20, 20));
        let mut expected = Vec::new();
        for i in 1..10 {
            assert!(tree.insert((i, i), 0b0000_0000 | i).is_ok());
            assert!(tree.insert((-i, i), 0b1000_0000 | i).is_ok());
            assert!(tree.insert((i, -i), 0b0100_0000 | i).is_ok());
            assert!(tree.insert((-i, -i), 0b1100_0000 | i).is_ok());
            if i <= 2 {
                expected.push(0b0000_0000 | i);
                expected.push(0b1000_0000 | i);
                expected.push(0b0100_0000 | i);
                expected.push(0b1100_0000 | i);
            }
        }
        let iter = tree.query(Boundary::new(-2, -2, 4, 4));
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
