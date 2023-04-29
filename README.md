# QuTee

QuTee is a simple implementation of a quad tree.
The advantage of QuTee over most other implementations is
that you can choose which data type you want to use as your coordinates.
By default all primitive number types are supported as coordinates.
Values do not require any special traits to be used.

## Example
```rust
use qutee::*;
// Create a new quad tree where the areas top left corner is at -10, -10 and with a width and height of 20.
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
The quad tree its self. A tree requires three important informations:
1. What type do you want to use for your positions? This can be any primitive number type.
2. What type of element do you want to store in your tree?
3. How many Elements do you want to save in each area? Lower values might
increase query speeds while larger values can improve insertion speed by reducing the amount of memory allocations needed.

I highly recommend that you create a custom type in which you warp the quad tree.
Otherwise you might end up with definitions like this allover your code.
```rust
let quad_tree: qutee::QuadTree<f64, String, CompileTimeCap<16>>;
```

#### Functions
<b>new(Boundary) -> Self</b>

Create a new quad tree which accepts values inside the given area.

<b>insert_at(impl Into&lt;Point&gt;, Item) -> Result<(), QuadTreeError></b>

Insert a item at a given point. Points can be created by using `Point::new(PU, PU)` or simply by using a tuple `(PU, PU)`.

<b>query(Boundary) -> Query</b>

Returns a iterator over all items inside the given boundary.

<b>iter() -> Iter</b>

Returns an iterator over all items inside the tree

### Boundary
A boundary represents an area in 2D space.

#### Functions
<b>new(Point, PU, PU) -> Self</b>

Create a new Boundary where point is the top left. The other two values represent width and height.

<b>between_points(Point, Point)</b>

Create a new Boundary between two points.

### Point
A single point in 2D space.

### Query
Query is an iterator over all items of a quad tree which are with in a given `Boundary`

### Iter
A iterator over all items inside a `QuadTree`
