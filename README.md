# Qutee

Qutee is a simple implementation of a quad tree.
Qutee allows you to choose which primitive number type should be used for coordinates.
Items of the quad tree do not require any trait bounds.

## Boundary
A boundary can be constructed with `Boundary::new` or `Boundary::between_points`.
`Boundary::new` takes a `Point` as its first argument followed by a width and height.
`Boundary::between_points` takes two `Point` Objects.

## Point
A point in 2D space.
A point can be constructed with `Point::new`. This function takes an `x` and `y` argument.
Most functions do not require a `Point` directly but take `impl Into<Point>` as an argument.
This allows for an tuple to be used where the first item is `x` and the second `y`.

## QuadTree
QuadTree provides the actual quad tree implementation. QuadTree has two required and one optional generic parameter.
The first two arguments are the coordinate and item type. The third parameter defines how the max capacity for each level is defined.
By default this argument is set to `DynCapacity` you can change this to `ConstCapacity` if you know the size at compile time.

### Create
To create a `QuadTree` you can use one of three methods
1. new_with_capacity takes a `Boundary` and parameter of type `Capacity`.
2. new_with_dyn_cap takes a `Boundary` and a capacity of type usize. This function is only available if the capacity is dynamic.
3. new_with_dyn_cap takes a `Boundary`. This function is only available if the capacity is known at compile time.

### Insert
A item can be inserted using the `insert_at` function. The first parameter is the point and the second the item.

### Query
`query` takes a `Boundary` and returns an Iterator of type `Query`

### Iter
`iter` returns an Iterator of type `Iter` containing all items in the tree.

## Example
```rust
use qutee::*;
// Create a new quadtree where the area's top left corner is at -10, -10, with a width and height of 20.
let mut tree = QuadTree::new_with_dyn_cap(Boundary::new((-10., -10.), 20., 20.), 5);
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
