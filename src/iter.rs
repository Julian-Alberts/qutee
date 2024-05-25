use crate::{bounds::Capacity, Area, Coordinate, Point, QuadTree};

/// Query Iterator over items and their coordinates
#[derive(Clone)]
pub struct QueryPoints<'a, PU, A, Item, Cap>
where
    Cap: Capacity,
    PU: Coordinate,
    A: Area<PU>,
{
    area: A,
    stack: Vec<InternQuery<'a, PU, Item, Cap>>,
}

#[derive(Clone)]
struct InternQuery<'a, PU, Item, Cap>
where
    Cap: Capacity,
    PU: Coordinate,
{
    is_enclosed_by_area: bool,
    quadrants: Option<&'a [QuadTree<PU, Item, Cap>]>,
    items: Option<&'a [(Point<PU>, Item)]>,
}

impl<'a, PU, Item, Cap, A> QueryPoints<'a, PU, A, Item, Cap>
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

impl<'a, PU, A, Item, Cap> Iterator for QueryPoints<'a, PU, A, Item, Cap>
where
    Cap: Capacity,
    PU: Coordinate,
    A: Area<PU>,
{
    type Item = &'a (Point<PU>, Item);
    fn next(&mut self) -> Option<Self::Item> {
        query_next(&mut self.stack, &self.area)
    }
}

fn query_next<'a, TreeItem, C, A, Cap, RetItem>(
    stack: &mut Vec<InternQuery<'a, C, TreeItem, Cap>>,
    area: &A,
) -> Option<&'a RetItem>
where
    RetItem: FromTreeItem<TreeItem, C>,
    C: Coordinate,
    A: Area<C>,
    Cap: Capacity,
{
    'main: loop {
        let ctx = stack.last_mut()?;
        if let Some(quads) = &mut ctx.quadrants {
            while !quads.is_empty() {
                let quad = &quads[0];
                *quads = &quads[1..];
                if ctx.is_enclosed_by_area || area.intersects(&quad.boundary) {
                    let int_query = InternQuery::new(quad, ctx.is_enclosed_by_area, area);
                    stack.push(int_query);
                    continue 'main;
                }
            }
            ctx.quadrants = None
        }

        if let Some(items) = &mut ctx.items {
            while !items.is_empty() {
                let item = &items[0];
                *items = &items[1..];
                if ctx.is_enclosed_by_area || area.contains(&item.0) {
                    return Some(RetItem::from_iter_type(item));
                }
            }
            ctx.quadrants = None;
        }

        stack.pop();
    }
}

impl<'a, C, Item, Cap> InternQuery<'a, C, Item, Cap>
where
    C: Coordinate,
    Cap: Capacity,
{
    #[inline(always)]
    fn new<A: Area<C>>(
        tree: &'a QuadTree<C, Item, Cap>,
        parent_is_enclosed_by_area: bool,
        area: &A,
    ) -> Self {
        Self {
            is_enclosed_by_area: parent_is_enclosed_by_area || area.encloses(&tree.boundary),
            items: tree.items.as_deref(),
            quadrants: tree.quadrants.as_ref().map(|q| q.as_slice()),
        }
    }
}

/// Iterator over all items and their coordinates
#[derive(Clone)]
pub struct IterPoints<'a, PU, Item, Cap>
where
    Cap: Capacity,
    PU: Coordinate,
{
    stack: Vec<IterIntern<'a, PU, Item, Cap>>
}

#[derive(Clone)]
struct IterIntern<'a, C, Item, Cap>
    where C: Coordinate
{
    quadrants: Option<&'a [QuadTree<C, Item, Cap>]>,
    items: Option<&'a [(Point<C>, Item)]>,
}

impl<'a, PU, Item, Cap> IterPoints<'a, PU, Item, Cap>
where
    Cap: Capacity,
    PU: Coordinate,
{
    pub(super) fn new(tree: &'a QuadTree<PU, Item, Cap>) -> Self {
        Self { stack: vec![IterIntern {quadrants: tree.quadrants.as_ref().map(|q| q.as_slice()), items: tree.items.as_ref().map(|items| items.as_slice())}] }
    }
}

impl<'a, PU, Item, Cap> Iterator for IterPoints<'a, PU, Item, Cap>
where
    Cap: Capacity,
    PU: Coordinate,
{
    type Item = &'a (Point<PU>, Item);

    fn next(&mut self) -> Option<Self::Item> {
        iter_next(&mut self.stack)
    }
}

fn iter_next<'a, C, TreeItem, RetItem, Cap>(
    stack: &mut Vec<IterIntern<'a, C, TreeItem, Cap>>
) -> Option<&'a RetItem>
    where C: Coordinate, RetItem: FromTreeItem<TreeItem, C>, Cap: Capacity
{
    loop {
        let ctx = stack.last_mut()?;
        if let Some(items) = &mut ctx.items {
            if !items.is_empty() {
                let item = &items[0];
                *items = &items[1..];
                return Some(RetItem::from_iter_type(item))
            }
            ctx.items = None;
        }

        if let Some(quadrants) = &mut ctx.quadrants {
            if !quadrants.is_empty() {
                let quad = &quadrants[0];
                *quadrants = &quadrants[1..];
                stack.push(IterIntern { quadrants: quad.quadrants.as_ref().map(|q| q.as_slice()), items: quad.items.as_ref().map(Vec::as_slice) });
            } else {
                ctx.quadrants = None;
                stack.pop();
            }
        } else {
            stack.pop();
        }
    }
}

/// Query Iterator
#[derive(Clone)]
#[repr(transparent)]
pub struct Query<'a, PU, A, Item, Cap>(QueryPoints<'a, PU, A, Item, Cap>)
where
    Cap: Capacity,
    PU: Coordinate,
    A: Area<PU>;

impl<'a, PU, Item, Cap, A> Query<'a, PU, A, Item, Cap>
where
    Cap: Capacity,
    PU: Coordinate,
    A: Area<PU> + Clone,
{
    pub(super) fn new(tree: &'a QuadTree<PU, Item, Cap>, area: A) -> Self {
        Self(QueryPoints::new(tree, area))
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
        query_next(&mut self.0.stack, &self.0.area)
    }
}

/// Iterator over all items
#[derive(Clone)]
#[repr(transparent)]
pub struct Iter<'a, PU, Item, Cap>(IterPoints<'a, PU, Item, Cap>)
where
    Cap: Capacity,
    PU: Coordinate;

impl<'a, PU, Item, Cap> Iter<'a, PU, Item, Cap>
where
    Cap: Capacity,
    PU: Coordinate,
{
    pub(super) fn new(tree: &'a QuadTree<PU, Item, Cap>) -> Self {
        Self(IterPoints::new(tree))
    }
}

impl<'a, PU, Item, Cap> Iterator for Iter<'a, PU, Item, Cap>
where
    Cap: Capacity,
    PU: Coordinate,
{
    type Item = &'a Item;

    fn next(&mut self) -> Option<Self::Item> {
        iter_next(&mut self.0.stack)
    }
}

trait FromTreeItem<Item, C>
where
    C: Coordinate,
{
    fn from_iter_type(t: &(Point<C>, Item)) -> &Self;
}

impl<Item, C> FromTreeItem<Item, C> for (Point<C>, Item)
where
    C: Coordinate,
{
    #[inline]
    fn from_iter_type(t: &(Point<C>, Item)) -> &Self {
        &t
    }
}

impl<Item, C> FromTreeItem<Self, C> for Item
where
    C: Coordinate,
{
    #[inline]
    fn from_iter_type(t: &(Point<C>, Self)) -> &Self {
        &t.1
    }
}
