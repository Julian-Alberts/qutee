use crate::{Area, Coordinate, Point, QuadTree};

/// Query Iterator over items and their coordinates
#[derive(Clone)]
pub struct QueryPoints<'a, PU, A, Item, const CAP: usize>(QuerySharedData<'a, PU, A, Item, CAP>)
where
    PU: Coordinate,
    A: Area<PU>;

#[derive(Clone)]
struct QuerySharedData<'a, C, A, Item, const CAP: usize>
where
    A: Area<C>,
    C: Coordinate,
{
    area: A,
    stack: Vec<QueryStackItem<'a, C, Item, CAP>>,
}

impl<'a, C, A, Item, const CAP: usize> QuerySharedData<'a, C, A, Item, CAP>
where
    A: Area<C>,
    C: Coordinate,
{
    fn new(tree: &'a QuadTree<C, Item, CAP>, area: A) -> Self {
        Self {
            stack: vec![QueryStackItem::new(tree, false, &area)],
            area,
        }
    }
}

#[derive(Clone)]
struct QueryStackItem<'a, PU, Item, const CAP: usize>
where
    PU: Coordinate,
{
    is_enclosed_by_area: bool,
    quadrants: &'a [QuadTree<PU, Item, CAP>],
    items: &'a [(Point<PU>, Item)],
}

impl<'a, PU, Item, const CAP: usize, A> QueryPoints<'a, PU, A, Item, CAP>
where
    PU: Coordinate,
    A: Area<PU> + Clone,
{
    pub(super) fn new(tree: &'a QuadTree<PU, Item, CAP>, area: A) -> Self {
        Self(QuerySharedData::new(tree, area))
    }
}

impl<'a, PU, A, Item, const CAP: usize> Iterator for QueryPoints<'a, PU, A, Item, CAP>
where
    PU: Coordinate,
    A: Area<PU>,
{
    type Item = &'a (Point<PU>, Item);
    fn next(&mut self) -> Option<Self::Item> {
        query_next(&mut self.0)
    }
}

fn query_next<'a, TreeItem, C, A, const CAP: usize, RetItem>(
    QuerySharedData { area, stack }: &mut QuerySharedData<'a, C, A, TreeItem, CAP>,
) -> Option<&'a RetItem>
where
    RetItem: FromTreeItem<TreeItem, C>,
    C: Coordinate,
    A: Area<C>,
{
    'main: loop {
        let ctx = stack.last_mut()?;
        while !ctx.quadrants.is_empty() {
            let quad = &ctx.quadrants[0];
            ctx.quadrants = &ctx.quadrants[1..];
            if ctx.is_enclosed_by_area || area.intersects(&quad.boundary) {
                let int_query = QueryStackItem::new(quad, ctx.is_enclosed_by_area, area);
                stack.push(int_query);
                continue 'main;
            }
        }

        while !ctx.items.is_empty() {
            let item = &ctx.items[0];
            ctx.items = &ctx.items[1..];
            if ctx.is_enclosed_by_area || area.contains(&item.0) {
                return Some(RetItem::from_iter_type(item));
            }
        }

        stack.pop();
    }
}

impl<'a, C, Item, const CAP: usize> QueryStackItem<'a, C, Item, CAP>
where
    C: Coordinate,
{
    #[inline(always)]
    fn new<A: Area<C>>(
        tree: &'a QuadTree<C, Item, CAP>,
        parent_is_enclosed_by_area: bool,
        area: &A,
    ) -> Self {
        Self {
            is_enclosed_by_area: parent_is_enclosed_by_area || area.encloses(&tree.boundary),
            items: &tree.items[..tree.next_item_index],
            quadrants: tree.quadrants.as_ref().map(|q| q.as_slice()).unwrap_or(&[]),
        }
    }
}

/// Iterator over all items and their coordinates
#[derive(Clone)]
pub struct IterPoints<'a, PU, Item, const CAP: usize>(IterSharedData<'a, PU, Item, CAP>)
where
    PU: Coordinate;

#[derive(Clone)]
struct IterSharedData<'a, PU, Item, const CAP: usize>
where
    PU: Coordinate,
{
    stack: Vec<IterStackItem<'a, PU, Item, CAP>>,
}

impl<'a, C, Item, const CAP: usize> IterSharedData<'a, C, Item, CAP>
where
    C: Coordinate,
{
    fn new(tree: &'a QuadTree<C, Item, CAP>) -> Self {
        Self {
            stack: vec![IterStackItem {
                quadrants: tree.quadrants.as_ref().map(|q| q.as_slice()),
                items: &tree.items[..tree.next_item_index],
            }],
        }
    }
}

#[derive(Clone)]
struct IterStackItem<'a, C, Item, const CAP: usize>
where
    C: Coordinate,
{
    quadrants: Option<&'a [QuadTree<C, Item, CAP>]>,
    items: &'a [(Point<C>, Item)],
}

impl<'a, PU, Item, const CAP: usize> IterPoints<'a, PU, Item, CAP>
where
    PU: Coordinate,
{
    pub(super) fn new(tree: &'a QuadTree<PU, Item, CAP>) -> Self {
        Self(IterSharedData::new(tree))
    }
}

impl<'a, PU, Item, const CAP: usize> Iterator for IterPoints<'a, PU, Item, CAP>
where
    PU: Coordinate,
{
    type Item = &'a (Point<PU>, Item);

    fn next(&mut self) -> Option<Self::Item> {
        iter_next(&mut self.0)
    }
}

fn iter_next<'a, C, TreeItem, RetItem, const CAP: usize>(
    IterSharedData { stack }: &mut IterSharedData<'a, C, TreeItem, CAP>,
) -> Option<&'a RetItem>
where
    C: Coordinate,
    RetItem: FromTreeItem<TreeItem, C>,
{
    loop {
        let ctx = stack.last_mut()?;
        if !ctx.items.is_empty() {
            let item = &ctx.items[0];
            ctx.items = &ctx.items[1..];
            return Some(RetItem::from_iter_type(item));
        }

        if let Some(quadrants) = &mut ctx.quadrants {
            if !quadrants.is_empty() {
                let quad = &quadrants[0];
                *quadrants = &quadrants[1..];
                stack.push(IterStackItem {
                    quadrants: quad.quadrants.as_ref().map(|q| q.as_slice()),
                    items: &quad.items[..quad.next_item_index],
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
pub struct Query<'a, PU, A, Item, const CAP: usize>(QuerySharedData<'a, PU, A, Item, CAP>)
where
    PU: Coordinate,
    A: Area<PU>;

impl<'a, PU, Item, const CAP: usize, A> Query<'a, PU, A, Item, CAP>
where
    PU: Coordinate,
    A: Area<PU> + Clone,
{
    pub(super) fn new(tree: &'a QuadTree<PU, Item, CAP>, area: A) -> Self {
        Self(QuerySharedData::new(tree, area))
    }
}

impl<'a, PU, A, Item, const CAP: usize> Iterator for Query<'a, PU, A, Item, CAP>
where
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
pub struct Iter<'a, PU, Item, const CAP: usize>(IterSharedData<'a, PU, Item, CAP>)
where
    PU: Coordinate;

impl<'a, PU, Item, const CAP: usize> Iter<'a, PU, Item, CAP>
where
    PU: Coordinate,
{
    pub(super) fn new(tree: &'a QuadTree<PU, Item, CAP>) -> Self {
        Self(IterSharedData::new(tree))
    }
}

impl<'a, PU, Item, const CAP: usize> Iterator for Iter<'a, PU, Item, CAP>
where
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
