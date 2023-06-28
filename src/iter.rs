use crate::{bounds::Capacity, Area, Coordinate, Point, QuadTree};

/// Query Iterator
pub struct Query<'a, PU, A, Item, Cap>
where
    Cap: Capacity,
    PU: Coordinate,
    A: Area<PU>,
{
    iter: InternQuery<'a, PU, A, Item, Cap>,
    area: A
}

struct InternQuery<'a, PU, A, Item, Cap>
where
    Cap: Capacity,
    PU: Coordinate,
    A: Area<PU>,
{
    quadrants: Option<&'a [QuadTree<PU, Item, Cap>]>,
    items: Option<&'a [(Point<PU>, Item)]>,
    current_sub_query: Option<Box<InternQuery<'a, PU, A, Item, Cap>>>,
    area: *const A,
}

impl<'a, PU, Item, Cap, A> Query<'a, PU, A, Item, Cap>
where
    Cap: Capacity,
    PU: Coordinate,
    A: Area<PU> + Clone,
{
    pub(super) fn new(tree: &'a QuadTree<PU, Item, Cap>, area: A) -> Self {
        let mut query = Self {
            iter: InternQuery {
                items: tree.items.as_deref(),
                quadrants: tree.quadrants.as_ref().map(|q| q.as_slice()),
                current_sub_query: None,
                area: std::ptr::null()
            },
            area,
        };
        query.iter.area = &query.area as *const _;
        query
    }
}

impl<'a, PU, A, Item, Cap> Iterator for Query<'a, PU, A, Item, Cap>
where
    Cap: Capacity,
    PU: Coordinate,
    A: Area<PU>,
{
    type Item = &'a Item;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a, PU, A, Item, Cap> InternQuery<'a, PU, A, Item, Cap>
where
    Cap: Capacity,
    PU: Coordinate,
    A: Area<PU>,
{
    fn find_next_quadrant(&mut self) -> Option<Box<InternQuery<'a, PU, A, Item, Cap>>> {
        let quadrants = self.quadrants.as_mut()?;
        if quadrants.is_empty() {
            return None;
        }
        while !quadrants.is_empty() {
            let q = &quadrants[0];
            *quadrants = &quadrants[1..];
            if unsafe {self.area.as_ref()}.unwrap().intersects(&q.boundary) {
                return Some(Box::new(InternQuery {
                    items: q.items.as_deref(),
                    quadrants: q.quadrants.as_ref().map(|q| q.as_slice()),
                    current_sub_query: None,
                    area: self.area
                }));
            }
        }
        None
    }
}

impl<'a, PU, A, Item, Cap> Iterator for InternQuery<'a, PU, A, Item, Cap>
where
    Cap: Capacity,
    PU: Coordinate,
    A: Area<PU>,
{
    type Item = &'a Item;

    fn next(&mut self) -> Option<Self::Item> {
        while self
            .items
            .map(|items| !items.is_empty())
            .unwrap_or_default()
        {
            let item = &self.items.unwrap()[0];
            self.items = self.items.map(|i| &i[1..]);
            if unsafe{ self.area.as_ref().unwrap().contains(&item.0) } {
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
    PU: Coordinate,
{
    quadrants: Option<&'a [QuadTree<PU, Item, Cap>]>,
    items: Option<&'a [(Point<PU>, Item)]>,
    current_sub_query: Option<Box<Iter<'a, PU, Item, Cap>>>,
}

impl<'a, PU, Item, Cap> Iter<'a, PU, Item, Cap>
where
    Cap: Capacity,
    PU: Coordinate,
{
    pub(super) fn new(tree: &'a QuadTree<PU, Item, Cap>) -> Self {
        Self {
            items: tree.items.as_deref(),
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
    PU: Coordinate,
{
    type Item = &'a Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.items.map(|i| !i.is_empty()).unwrap_or_default() {
            let item = &self.items.unwrap()[0].1;
            self.items = self.items.map(|i| &i[1..]);
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
