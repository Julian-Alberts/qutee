use crate::{bounds::Capacity, Area, Coordinate, Point, QuadTree};

/// Query Iterator
pub struct Query<'a, PU, A, Item, Cap>
where
    Cap: Capacity,
    PU: Coordinate,
    A: Area<PU>,
{
    area: A,
    stack: Vec<InternQuery<'a, PU, Item, Cap>>,
}

struct InternQuery<'a, PU, Item, Cap>
where
    Cap: Capacity,
    PU: Coordinate,
{
    is_enclosed_by_area: bool,
    quadrants: Option<&'a [QuadTree<PU, Item, Cap>]>,
    items: Option<&'a [(Point<PU>, Item)]>,
}

impl<'a, PU, Item, Cap, A> Query<'a, PU, A, Item, Cap>
where
    Cap: Capacity,
    PU: Coordinate,
    A: Area<PU> + Clone,
{
    pub(super) fn new(tree: &'a QuadTree<PU, Item, Cap>, area: A) -> Self {
        Self {
            stack: vec![InternQuery::new(tree, false, &area)],
            area,
        }
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
        'main: loop {
            let ctx = self.stack.last_mut()?;
            if let Some(quads) = &mut ctx.quadrants {
                while !quads.is_empty() {
                    let quad = &quads[0];
                    *quads = &quads[1..];
                    if ctx.is_enclosed_by_area || self.area.intersects(&quad.boundary) {
                        let int_query = InternQuery::new(quad,ctx.is_enclosed_by_area, &self.area);
                        self.stack.push(int_query);
                        continue 'main;
                    }
                }
                ctx.quadrants = None
            }

            if let Some(items) = &mut ctx.items {
                while !items.is_empty() {
                    let item = &items[0];
                    *items = &items[1..];
                    if ctx.is_enclosed_by_area || self.area.contains(&item.0) {
                        return Some(&item.1);
                    }
                }
                ctx.quadrants = None;
            }

            self.stack.pop();
        }
    }
}

impl<'a, C, Item, Cap> InternQuery<'a, C, Item, Cap>
where
    C: Coordinate,
    Cap: Capacity,
{
    #[inline(always)]
    fn new<A: Area<C>>(tree: &'a QuadTree<C, Item, Cap>, parent_is_enclosed_by_area: bool, area: &A) -> Self {
        Self {
            is_enclosed_by_area: parent_is_enclosed_by_area || area.encloses(&tree.boundary),
            items: tree.items.as_deref(),
            quadrants: tree.quadrants.as_ref().map(|q| q.as_slice()),
        }
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
