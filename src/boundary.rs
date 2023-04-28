use std::fmt::Debug;

use crate::Point;

/// A Area
#[derive(Debug, Clone, PartialEq)]
pub struct Boundary<T = usize>
where
    T: PositionUnit,
{
    p1: Point<T>,
    p2: Point<T>,
}

/// This trait is required for coordinates
pub trait PositionUnit:
    num_traits::NumOps + Sized + Clone + Copy + num_traits::NumCast + PartialOrd + Debug
{}

impl<T> Boundary<T>
where
    T: PositionUnit,
{
    /// create a new Boundary from x,y with width and height
    pub fn new(point: impl Into<Point<T>>, width: T, height: T) -> Self {
        let p1 = point.into();
        let p2 = (p1.x + width, p1.y + height).into();
        Self { p1, p2 }
    }

    /// Create a new Area between two points
    pub fn between_points(p1: impl Into<Point<T>>, p2: impl Into<Point<T>>) -> Self {
        let mut p1: Point<_> = p1.into();
        let mut p2: Point<_> = p2.into();

        if p1.x > p2.x {
            std::mem::swap(&mut p1.x, &mut p2.x)
        }
        if p1.y > p2.y {
            std::mem::swap(&mut p1.y, &mut p2.y)
        }

        Self { p1, p2 }
    }

    pub(crate) fn split(&self) -> [Boundary<T>; 4] {
        let dx = self.p2.x - self.p1.x;
        let dy = self.p2.y - self.p1.y;
        let two = T::from(2).expect("Could not convert 2 to required type");
        let half_dx = dx / two;
        let half_dy = dy / two;
        [
            Boundary::new(self.p1.clone(), half_dx, half_dy),
            Boundary::new((self.p1.x + half_dx, self.p1.y), half_dx, half_dy),
            Boundary::new((self.p1.x, self.p1.y + half_dy), half_dx, half_dy),
            Boundary::new((self.p1.x + half_dx, self.p1.y + half_dy), half_dx, half_dy),
        ]
    }

    pub(crate) fn contains(&self, point: &Point<T>) -> bool {
        !(point.x < self.p1.x || point.x > self.p2.x || point.y < self.p1.y || point.y > self.p2.y)
    }

    pub(crate) fn overlaps(&self, Boundary { p1, p2 }: &Boundary<T>) -> bool {
        !(p2.x < self.p1.x || p1.x > self.p2.x || p2.y < self.p1.y || p1.y > self.p2.y)
    }
}

impl PositionUnit for usize {}
impl PositionUnit for isize {}
impl PositionUnit for u8 {}
impl PositionUnit for u16 {}
impl PositionUnit for u32 {}
impl PositionUnit for u64 {}
impl PositionUnit for u128 {}
impl PositionUnit for i8 {}
impl PositionUnit for i16 {}
impl PositionUnit for i32 {}
impl PositionUnit for i64 {}
impl PositionUnit for i128 {}
impl PositionUnit for f32 {}
impl PositionUnit for f64 {}


#[cfg(test)]
mod tests {
    use crate::{Boundary, Point};
    use test_case::test_case;

    #[test_case(1,1,2,2 => Boundary::new((1,1),1,1); "Simple case")]
    #[test_case(2,1,1,2 => Boundary::new((1,1),1,1); "Swap x")]
    #[test_case(1,2,2,1 => Boundary::new((1,1),1,1); "Swap y")]
    #[test_case(2,2,1,1 => Boundary::new((1,1),1,1); "Swap both")]
    fn boundary_between_points(x1: usize, y1: usize, x2: usize, y2: usize) -> Boundary {
        Boundary::between_points((x1, y1), (x2, y2))
    }

    #[test]
    fn split_boundary_equal() {
        let b = Boundary::new((0, 0), 10, 10);
        let split = b.split();
        assert_eq!(
            split[0],
            Boundary::new((0,0),5,5)
        );
        assert_eq!(
            split[1],
            Boundary::new((5,0),5,5)
        );
        assert_eq!(
            split[2],
            Boundary::new((0,5),5,5)
        );
        assert_eq!(
            split[3],
            Boundary::new((5,5),5,5)
        );
    }

    #[test_case(3,3 => true; "Contains point")]
    #[test_case(2,2 => true; "Contains point on border")]
    #[test_case(1,3 => false; "Point above")]
    #[test_case(5,3 => false; "Point below")]
    #[test_case(3,1 => false; "Point left")]
    #[test_case(3,5 => false; "Point right")]
    fn boundary_contains_point(x: usize, y: usize) -> bool {
        let b = Boundary::new((2, 2), 2, 2);
        let p = Point { x, y };
        b.contains(&p)
    }

    #[test_case(2,2,1,1 => true; "b inside a")]
    #[test_case(0,0,6,6 => true; "a inside b")]
    #[test_case(0,2,3,1 => true; "left overlap")]
    #[test_case(4,2,3,1 => true; "right overlap")]
    #[test_case(2,0,1,3 => true; "top overlap")]
    #[test_case(2,4,1,3 => true; "bottom overlap")]
    #[test_case(-1, 2, 1, 1 => false; "b left of a")]
    #[test_case(6, 2, 1, 1 => false; "b right of a")]
    #[test_case(2, -1, 1, 1 => false; "b above of a")]
    #[test_case(2, 6, 1, 1 => false; "b under of a")]
    #[test_case(0,2,1,1 => true; "on left border")]
    #[test_case(5,2,1,1 => true; "on right border")]
    #[test_case(2,0,1,1 => true; "on top border")]
    #[test_case(2,5,1,1 => true; "on bottom border")]
    fn boundary_overlaps(x: isize, y: isize, width: isize, height: isize) -> bool {
        let a = Boundary::new((1, 1), 4, 4);
        let b = Boundary::new((x, y), width, height);
        a.overlaps(&b)
    }
}
