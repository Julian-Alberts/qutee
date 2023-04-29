# QuTee

QuTee is a simple implementation of a quadtree.
The advantage of QuTee over most other implementations is
that you can choose which data type you want to use as your coordinates.
By default, all primitive number types are supported as coordinates.
Values do not require any special traits to be used.

## Example
```rust
use qutee::*;
// Create a new quadtree where the area's top left corner is at -10, -10, with a width and height of 20.
type QuadTree<Cord, Item> = qutee::QuadTree<Cord, Item, RunTimeCap>;
let mut tree = QuadTree::new(Boundary::new((-10., -10.), 20., 20.), 5);
assert!(tree.insert_at((0.5, 0.1), "A").is_ok());
assert!(tree.insert_at((-1., 1.), "B").is_ok());
// This point is outside the tree
assert_eq!(tree.insert_at((10.1, 5.), "C"), Err(QuadTreeError::OutOfBounds));
// Search elements inside a boundary. A boundary can also be defined as an area between two points.
let mut query = tree.query(Boundary::between_points((0.,0.),(1.,1.)));
assert_eq!(query.next(), Some(&"A"));
assert!(query.next().is_none());
// Get an iterator over all items
let mut iter = tree.iter();
assert_eq!(iter.next(), Some(&"A"));
assert_eq!(iter.next(), Some(&"B"));
assert!(iter.next().is_none());
```

## Important types
### QuadTree
A tree requires three important pieces of information:
1. What type do you want to use for your positions? This can be any primitive number type.
2. What type of element do you want to store in your tree?
3. How many Elements do you want to save in each area? Lower values might
increase query speeds, while larger values can improve insertion speed by reducing the number of memory allocations needed.

I recommend creating a custom type in which you warp the quadtree.
Otherwise, you might have definitions like this all over your code.
```rust
let quad_tree: qutee::QuadTree<f64, String, CompileTimeCap<16>>;
```

#### Functions
<b>new(Boundary) -> Self</b>

Create a new quadtree that accepts values inside the given area.

<b>insert_at(impl Into&lt;Point&gt;, Item) -> Result<(), QuadTreeError></b>

Insert an item at a given point. Points can be created using `Point::new(PU, PU)` or simply using a tuple `(PU, PU)`.

<b>query(Boundary) -> Query</b>

Query returns an iterator over all items inside the given boundary.

<b>iter() -> Iter</b>

Returns an iterator over all items inside the tree

### Boundary
A boundary represents an area in 2D space.

#### Functions
<b>new(Point, PU, PU) -> Self</b>

Create a new Boundary where the point is at the top left. The other two values represent width and height.

<b>between_points(Point, Point)</b>

Create a new Boundary between two points.

### Point
A single point in 2D space.

### Query
A query is an iterator over all items of a quadtree within a given `Boundary`.

### Iter
An iterator over all items inside a `QuadTree`.
