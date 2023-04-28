# QuTree

QuTree is a simple implementation of a quad tree.
The advantage of QuTree over most over implementations is 
that you can choose which data type you want to use as your coordinates.
By default all basic number types are supported as coordinates. 
Values do not require any special traits to be used.

#Example
```rust
use qutree::*;
// Create a new quad tree where the areas top left corner is at -10, -10 and with a width and height of 20.
let mut tree: QuadTree<f64, &str, 5> = QuadTree::new(Boundary::new((-10., -10.), 20., 20.));
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
