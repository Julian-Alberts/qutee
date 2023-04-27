use std::fmt::Debug;

use crate::{PositionUnit, Point, QuadTree, Boundary};


pub struct Query<'a, PU, Item, const CAPACITY: usize>
where
    PU: PositionUnit,
{
    quadrants: Option<Vec<&'a QuadTree<PU, Item, CAPACITY>>>,
    items: Vec<&'a (Point<PU>, Item)>,
    current_sub_query: Option<Box<Query<'a, PU, Item, CAPACITY>>>,
    boundary: Boundary<PU>,
}

impl<'a, PU, Item, const CAPACITY: usize> Query<'a, PU, Item, CAPACITY>
where
    PU: PositionUnit,
    Item: Debug,
{
    pub(super) fn new(tree: &'a QuadTree<PU, Item, CAPACITY>, boundary: Boundary<PU>) -> Self {
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

pub struct Iter<'a, PU, Item, const CAPACITY: usize>
where
    PU: PositionUnit,
{
    quadrants: Option<Vec<&'a QuadTree<PU, Item, CAPACITY>>>,
    items: Vec<&'a Item>,
    current_sub_query: Option<Box<Iter<'a, PU, Item, CAPACITY>>>,
}

impl<'a, PU, Item, const CAPACITY: usize> Iter<'a, PU, Item, CAPACITY>
where
    PU: PositionUnit,
    Item: Debug,
{
    pub(super) fn new(tree: &'a QuadTree<PU, Item, CAPACITY>) -> Self {
        Self {
            items: tree.items.iter().map(|i| &i.1).collect(),
            quadrants: tree.quadrants.as_ref().map(|q| q.iter().collect()),
            current_sub_query: None,
        }
    }

    fn find_next_quadrant(&mut self) -> Option<Box<Iter<'a, PU, Item, CAPACITY>>> {
        let quadrants = self.quadrants.as_mut()?;
        if quadrants.is_empty() {
            return None;
        }
        while !quadrants.is_empty() {
            let q = quadrants.remove(0);
            return Some(Box::new(q.iter()));
        }
        None
    }
}

impl<'a, PU, Item, const CAPACITY: usize> Iterator for Iter<'a, PU, Item, CAPACITY>
where
    PU: PositionUnit,
    Item: Debug,
{
    type Item = &'a Item;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.items.is_empty() {
            let item = self.items.remove(0);
            return Some(&item);
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
