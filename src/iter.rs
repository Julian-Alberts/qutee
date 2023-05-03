use crate::{bounds::Capacity, Boundary, Point, PositionUnit, QuadTree};

/// Query Iterator
pub struct Query<'a, PU, Item, Cap>
where
    Cap: Capacity,
    PU: PositionUnit,
{
    quadrants: Option<&'a [QuadTree<PU, Item, Cap>]>,
    items: &'a [(Point<PU>, Item)],
    current_sub_query: Option<Box<Query<'a, PU, Item, Cap>>>,
    boundary: Boundary<PU>,
}

impl<'a, PU, Item, Cap> Query<'a, PU, Item, Cap>
where
    Cap: Capacity,
    PU: PositionUnit,
{
    pub(super) fn new(tree: &'a QuadTree<PU, Item, Cap>, boundary: Boundary<PU>) -> Self {
        Self {
            items: tree.items.as_slice(),
            quadrants: tree.quadrants.as_ref().map(|q| q.as_slice()),
            current_sub_query: None,
            boundary,
        }
    }

    fn find_next_quadrant(&mut self) -> Option<Box<Query<'a, PU, Item, Cap>>> {
        let quadrants = self.quadrants.as_mut()?;
        if quadrants.is_empty() {
            return None;
        }
        while !quadrants.is_empty() {
            let q = &quadrants[0];
            *quadrants = &quadrants[1..];
            if q.boundary.overlaps(&self.boundary) {
                return Some(Box::new(q.query(self.boundary.clone())));
            }
        }
        None
    }
}

impl<'a, PU, Item, Cap> Iterator for Query<'a, PU, Item, Cap>
where
    Cap: Capacity,
    PU: PositionUnit,
{
    type Item = &'a Item;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.items.is_empty() {
            let item = &self.items[0];
            self.items = &self.items[1..];
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

/// Iterator over all items
pub struct Iter<'a, PU, Item, Cap>
where
    Cap: Capacity,
    PU: PositionUnit,
{
    quadrants: Option<&'a [QuadTree<PU, Item, Cap>]>,
    items: &'a [(Point<PU>, Item)],
    current_sub_query: Option<Box<Iter<'a, PU, Item, Cap>>>,
}

impl<'a, PU, Item, Cap> Iter<'a, PU, Item, Cap>
where
    Cap: Capacity,
    PU: PositionUnit,
{
    pub(super) fn new(tree: &'a QuadTree<PU, Item, Cap>) -> Self {
        Self {
            items: tree.items.as_slice(),
            quadrants: tree.quadrants.as_ref().map(|q| q.as_slice()),
            current_sub_query: None,
        }
    }

    fn find_next_quadrant(&mut self) -> Option<Box<Iter<'a, PU, Item, Cap>>> {
        let quadrants = self.quadrants.as_mut()?;
        if quadrants.is_empty() {
            return None;
        }
        let q = &quadrants[0];
        *quadrants = &quadrants[1..];
        return Some(Box::new(q.iter()));
    }
}

impl<'a, PU, Item, Cap> Iterator for Iter<'a, PU, Item, Cap>
where
    Cap: Capacity,
    PU: PositionUnit,
{
    type Item = &'a Item;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.items.is_empty() {
            let item = &self.items[0].1;
            self.items = &self.items[1..];
            return Some(item);
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
