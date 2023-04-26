use std::{
    fmt::Debug,
    ops::{Add, Div},
};

use crate::Point;

#[derive(Debug, Clone, PartialEq)]
pub struct Boundary<T = usize>
where
    T: PositionUnit,
{
    x: T,
    y: T,
    width: T,
    height: T,
}

pub trait PositionUnit:
    Div<Output = Self> + Add<Output = Self> + Copy + Clone + Sized + PartialEq + PartialOrd + Debug
{
    fn convert(value: usize) -> Self;
}

impl<T> Boundary<T>
where
    T: PositionUnit,
{
    pub fn new(x: T, y: T, width: T, height: T) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn split(&self) -> [Boundary<T>; 4] {
        let half_width = self.width / T::convert(2);
        let half_height = self.height / T::convert(2);
        let x = self.x;
        let y = self.y;
        [
            Boundary::new(x, y, half_width, half_height),
            Boundary::new(x + half_width, y, half_width, half_height),
            Boundary::new(x, y + half_height, half_width, half_height),
            Boundary::new(x + half_width, y + half_height, half_width, half_height),
        ]
    }

    pub fn contains(&self, point: &Point<T>) -> bool {
        !(point.x < self.x
            || point.x > self.x + self.width
            || point.y < self.y
            || point.y > self.y + self.height)
    }

    pub(crate) fn overlaps(
        &self,
        Boundary {
            x,
            y,
            width,
            height,
        }: &Boundary<T>,
    ) -> bool {
        !(*x + *width < self.x
            || *x > self.x + self.width
            || *y + *height < self.y
            || *y > self.y + self.height)
    }
}

impl PositionUnit for usize {
    fn convert(value: usize) -> Self {
        value
    }
}
impl PositionUnit for isize {
    fn convert(value: usize) -> Self {
        value as isize
    }
}
impl PositionUnit for u8 {
    fn convert(value: usize) -> Self {
        value as u8
    }
}
impl PositionUnit for u16 {
    fn convert(value: usize) -> Self {
        value as u16
    }
}
impl PositionUnit for u32 {
    fn convert(value: usize) -> Self {
        value as u32
    }
}
impl PositionUnit for u64 {
    fn convert(value: usize) -> Self {
        value as u64
    }
}
impl PositionUnit for i8 {
    fn convert(value: usize) -> Self {
        value as i8
    }
}
impl PositionUnit for i16 {
    fn convert(value: usize) -> Self {
        value as i16
    }
}
impl PositionUnit for i32 {
    fn convert(value: usize) -> Self {
        value as i32
    }
}
impl PositionUnit for i64 {
    fn convert(value: usize) -> Self {
        value as i64
    }
}
impl PositionUnit for f32 {
    fn convert(value: usize) -> Self {
        value as f32
    }
}
impl PositionUnit for f64 {
    fn convert(value: usize) -> Self {
        value as f64
    }
}

#[cfg(test)]
mod tests {
    use crate::{Boundary, Point};
    use test_case::test_case;

    #[test]
    fn split_boundary_equal() {
        let b = Boundary::new(0, 0, 10, 10);
        let split = b.split();
        assert_eq!(
            split[0],
            Boundary {
                x: 0,
                y: 0,
                width: 5,
                height: 5
            }
        );
        assert_eq!(
            split[1],
            Boundary {
                x: 5,
                y: 0,
                width: 5,
                height: 5
            }
        );
        assert_eq!(
            split[2],
            Boundary {
                x: 0,
                y: 5,
                width: 5,
                height: 5
            }
        );
        assert_eq!(
            split[3],
            Boundary {
                x: 5,
                y: 5,
                width: 5,
                height: 5
            }
        );
    }

    #[test_case(3,3 => true; "Contains point")]
    #[test_case(2,2 => true; "Contains point on border")]
    #[test_case(1,3 => false; "Point above")]
    #[test_case(5,3 => false; "Point below")]
    #[test_case(3,1 => false; "Point left")]
    #[test_case(3,5 => false; "Point right")]
    fn boundary_contains_point(x: usize, y: usize) -> bool {
        let b = Boundary::new(2usize, 2, 2, 2);
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
        let a = Boundary::new(1,1,4,4);
        let b = Boundary::new(x, y, width, height);
        a.overlaps(&b)
    }
}
