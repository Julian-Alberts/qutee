use crate::{bounds::Capacity, Area, Coordinate, Point, QuadTree};

/// Query Iterator over items and their coordinates
#[derive(Clone)]
pub struct QueryPoints<'a, PU, A, Item, Cap>(QuerySharedData<'a, PU, A, Item, Cap>)
where
    Cap: Capacity,
    PU: Coordinate,
    A: Area<PU>;

#[derive(Clone)]
struct QuerySharedData<'a, C, A, Item, Cap>
where
    A: Area<C>,
    C: Coordinate,
    Cap: Capacity,
{
    area: A,
    stack: Vec<QueryStackItem<'a, C, Item, Cap>>,
}

impl<'a, C, A, Item, Cap> QuerySharedData<'a, C, A, Item, Cap>
where
    A: Area<C>,
    C: Coordinate,
    Cap: Capacity,
{
    fn new(tree: &'a QuadTree<C, Item, Cap>, area: A) -> Self {
        Self {
            stack: vec![QueryStackItem::new(tree, false, &area)],
            area,
        }
    }
}

#[derive(Clone)]
struct QueryStackItem<'a, PU, Item, Cap>
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
        Self(QuerySharedData::new(tree, area))
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
        query_next(&mut self.0)
    }
}

fn query_next<'a, TreeItem, C, A, Cap, RetItem>(
    QuerySharedData { area, stack }: &mut QuerySharedData<'a, C, A, TreeItem, Cap>,
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
                    let int_query = QueryStackItem::new(quad, ctx.is_enclosed_by_area, area);
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

impl<'a, C, Item, Cap> QueryStackItem<'a, C, Item, Cap>
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
pub struct IterPoints<'a, PU, Item, Cap>(IterSharedData<'a, PU, Item, Cap>)
where
    Cap: Capacity,
    PU: Coordinate;

#[derive(Clone)]
struct IterSharedData<'a, PU, Item, Cap>
where
    Cap: Capacity,
    PU: Coordinate,
{
    stack: Vec<IterStackItem<'a, PU, Item, Cap>>,
}

impl<'a, C, Item, Cap> IterSharedData<'a, C, Item, Cap>
where
    Cap: Capacity,
    C: Coordinate,
{
    fn new(tree: &'a QuadTree<C, Item, Cap>) -> Self {
        Self {
            stack: vec![IterStackItem {
                quadrants: tree.quadrants.as_ref().map(|q| q.as_slice()),
                items: tree.items.as_deref(),
            }],
        }
    }
}

#[derive(Clone)]
struct IterStackItem<'a, C, Item, Cap>
where
    C: Coordinate,
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
        Self(IterSharedData::new(tree))
    }
}

impl<'a, PU, Item, Cap> Iterator for IterPoints<'a, PU, Item, Cap>
where
    Cap: Capacity,
    PU: Coordinate,
{
    type Item = &'a (Point<PU>, Item);

    fn next(&mut self) -> Option<Self::Item> {
        iter_next(&mut self.0)
    }
}

fn iter_next<'a, C, TreeItem, RetItem, Cap>(
    IterSharedData { stack }: &mut IterSharedData<'a, C, TreeItem, Cap>,
) -> Option<&'a RetItem>
where
    C: Coordinate,
    RetItem: FromTreeItem<TreeItem, C>,
    Cap: Capacity,
{
    loop {
        let ctx = stack.last_mut()?;
        if let Some(items) = &mut ctx.items {
            if !items.is_empty() {
                let item = &items[0];
                *items = &items[1..];
                return Some(RetItem::from_iter_type(item));
            }
            ctx.items = None;
        }

        if let Some(quadrants) = &mut ctx.quadrants {
            if !quadrants.is_empty() {
                let quad = &quadrants[0];
                *quadrants = &quadrants[1..];
                stack.push(IterStackItem {
                    quadrants: quad.quadrants.as_ref().map(|q| q.as_slice()),
                    items: quad.items.as_deref(),
                });
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
pub struct Query<'a, PU, A, Item, Cap>(QuerySharedData<'a, PU, A, Item, Cap>)
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
        Self(QuerySharedData::new(tree, area))
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
        query_next(&mut self.0)
    }
}

/// Iterator over all items
#[derive(Clone)]
#[repr(transparent)]
pub struct Iter<'a, PU, Item, Cap>(IterSharedData<'a, PU, Item, Cap>)
where
    Cap: Capacity,
    PU: Coordinate;

impl<'a, PU, Item, Cap> Iter<'a, PU, Item, Cap>
where
    Cap: Capacity,
    PU: Coordinate,
{
    pub(super) fn new(tree: &'a QuadTree<PU, Item, Cap>) -> Self {
        Self(IterSharedData::new(tree))
    }
}

impl<'a, PU, Item, Cap> Iterator for Iter<'a, PU, Item, Cap>
where
    Cap: Capacity,
    PU: Coordinate,
{
    type Item = &'a Item;

    fn next(&mut self) -> Option<Self::Item> {
        iter_next(&mut self.0)
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
        t
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
