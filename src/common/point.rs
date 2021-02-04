use crate::common::{signed, Cardinal};
use auto_ops::*;
use nom::{
    character::complete::{char, space0},
    combinator::{cond, opt},
    sequence::{pair, preceded, separated_pair, tuple},
    IResult,
};
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn origin() -> Self {
        Self { x: 0, y: 0 }
    }

    pub fn from_string(input: &str) -> Self {
        Point::parser(input).expect("Invalid point format").1
    }

    pub fn parser(input: &str) -> IResult<&str, Self> {
        // Accepts the following, with whitespace allowed anywhere:
        //  123,456
        //  (123,456)
        let (input, _) = space0(input)?;
        let (input, open_paren) = opt(char('('))(input)?;
        let (input, (x, y)) = preceded(
            space0,
            separated_pair(signed, tuple((space0, char(','), space0)), signed),
        )(input)?;
        let (input, _) = cond(open_paren.is_some(), pair(space0, char(')')))(input)?;

        Ok((input, Self { x, y }))
    }

    pub fn manhattan(a: Self, b: Self) -> u32 {
        let delta = a - b;
        let distance = delta.x.abs() + delta.y.abs();
        distance as u32
    }

    pub fn cmp_x_y(a: &Self, b: &Self) -> Ordering {
        let compare = a.x.cmp(&b.x);
        if compare == Ordering::Equal {
            a.y.cmp(&b.y)
        } else {
            compare
        }
    }

    pub fn cmp_y_x(a: &Self, b: &Self) -> Ordering {
        let compare = a.y.cmp(&b.y);
        if compare == Ordering::Equal {
            a.x.cmp(&b.x)
        } else {
            compare
        }
    }

    pub fn step(&self, direction: Cardinal, count: i32) -> Self {
        match direction {
            Cardinal::North => *self + (0, -count),
            Cardinal::South => *self + (0, count),
            Cardinal::East => *self + (count, 0),
            Cardinal::West => *self + (-count, 0),
        }
    }

    const ORTHOGONALS: [(i32, i32); 4] = [(0, -1), (0, 1), (1, 0), (-1, 0)];
    pub fn orthogonals(&self) -> impl Iterator<Item = Self> + '_ {
        Self::ORTHOGONALS.iter().map(move |p| *self + *p)
    }

    const DIAGONALS: [(i32, i32); 4] = [(-1, -1), (1, -1), (1, 1), (-1, 1)];
    pub fn diagonals(&self) -> impl Iterator<Item = Self> + '_ {
        Self::DIAGONALS.iter().map(move |p| *self + *p)
    }

    pub fn adjacents(&self) -> impl Iterator<Item = Self> + '_ {
        self.orthogonals().chain(self.diagonals())
    }

    pub fn get_range<'a, I>(values: I) -> Option<((i32, i32), (i32, i32))>
    where
        I: std::iter::IntoIterator<Item = &'a Point>, // Using IntoIterator instead of Iterator allows the user to pass either an iterator or something that can be turned into one
    {
        let mut point_iter = values.into_iter(); // Note: if 'values' is an Iterator it will just return itself here
        if let Some(point) = point_iter.next() {
            let range = point_iter.fold(
                ((point.x, point.x), (point.y, point.y)),
                |(acc_x, acc_y), p| {
                    (
                        (acc_x.0.min(p.x), acc_x.1.max(p.x)),
                        (acc_y.0.min(p.y), acc_y.1.max(p.y)),
                    )
                },
            );
            Some(range)
        } else {
            None
        }
    }
}

type Tuple = (i32, i32); // TODO: remove me when auto_ops releases a new version (0.2.0?) that allows sequence types to be used in the impl macros

impl_op_ex!(+ |a: &Point, b: &Point| -> Point {
    Point {
        x: a.x + b.x,
        y: a.y + b.y,
    }
});

impl_op_ex_commutative!(+ |a: &Point, b: &Tuple| -> Point {
    Point {
        x: a.x + b.0,
        y: a.y + b.1,
    }
});

impl_op_ex!(+= |a: &mut Point, b: &Point| { *a = *a + b });
impl_op_ex!(+= |a: &mut Point, b: &Tuple| { *a = *a + b });

impl_op_ex!(-|a: &Point, b: &Point| -> Point {
    Point {
        x: a.x - b.x,
        y: a.y - b.y,
    }
});

impl_op_ex!(-|a: &Point, b: &Tuple| -> Point {
    Point {
        x: a.x - b.0,
        y: a.y - b.1,
    }
});

impl_op_ex!(-= |a: &mut Point, b: &Point| { *a = *a - b });
impl_op_ex!(-= |a: &mut Point, b: &Tuple| { *a = *a - b });

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Point {
    fn cmp(&self, other: &Self) -> Ordering {
        Self::cmp_y_x(self, other)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_string() {
        assert_eq!(Point::from_string("123,456"), Point { x: 123, y: 456 });
        assert_eq!(Point::from_string(" 123,456"), Point { x: 123, y: 456 });
        assert_eq!(Point::from_string("123 ,456"), Point { x: 123, y: 456 });
        assert_eq!(Point::from_string("123, 456"), Point { x: 123, y: 456 });
        assert_eq!(Point::from_string("123,456 "), Point { x: 123, y: 456 });
        assert_eq!(Point::from_string(" 123 , 456 "), Point { x: 123, y: 456 });
        assert_eq!(Point::from_string("(123,456)"), Point { x: 123, y: 456 });
        assert_eq!(Point::from_string(" (123,456)"), Point { x: 123, y: 456 });
        assert_eq!(Point::from_string("( 123,456)"), Point { x: 123, y: 456 });
        assert_eq!(Point::from_string("(123 ,456)"), Point { x: 123, y: 456 });
        assert_eq!(Point::from_string("(123, 456)"), Point { x: 123, y: 456 });
        assert_eq!(Point::from_string("(123,456 )"), Point { x: 123, y: 456 });
        assert_eq!(Point::from_string("(123,456) "), Point { x: 123, y: 456 });
        assert_eq!(
            Point::from_string("( 123 , 456 )"),
            Point { x: 123, y: 456 }
        );
        assert_eq!(Point::from_string("123,456)"), Point { x: 123, y: 456 });
    }

    #[test]
    #[should_panic]
    fn test_from_string_fail() {
        Point::from_string("(123,456");
    }

    #[test]
    fn test_manhattan() {
        let a = Point { x: 0, y: 0 };
        let b = Point { x: 0, y: 0 };
        assert_eq!(Point::manhattan(a, b), 0);

        let a = Point { x: 1, y: 2 };
        let b = Point { x: 3, y: 4 };
        assert_eq!(Point::manhattan(a, b), 4);

        let a = Point { x: -1, y: -2 };
        let b = Point { x: 3, y: 4 };
        assert_eq!(Point::manhattan(a, b), 10);
    }

    #[test]
    fn test_cmp_x_y() {
        let a = Point { x: 0, y: 0 };
        let b = Point { x: 0, y: 0 };
        assert_eq!(Point::cmp_x_y(&a, &b), Ordering::Equal);

        let a = Point { x: 1, y: 2 };
        let b = Point { x: 3, y: 4 };
        assert_eq!(Point::cmp_x_y(&a, &b), Ordering::Less);

        let a = Point { x: 1, y: 2 };
        let b = Point { x: 3, y: -5 };
        assert_eq!(Point::cmp_x_y(&a, &b), Ordering::Less);

        let a = Point { x: 1, y: 2 };
        let b = Point { x: 1, y: 4 };
        assert_eq!(Point::cmp_x_y(&a, &b), Ordering::Less);

        let a = Point { x: 1, y: 2 };
        let b = Point { x: 1, y: -5 };
        assert_eq!(Point::cmp_x_y(&a, &b), Ordering::Greater);
    }

    #[test]
    fn test_cmp_y_x() {
        let a = Point { x: 0, y: 0 };
        let b = Point { x: 0, y: 0 };
        assert_eq!(Point::cmp_y_x(&a, &b), Ordering::Equal);

        let a = Point { x: 1, y: 2 };
        let b = Point { x: 3, y: 4 };
        assert_eq!(Point::cmp_y_x(&a, &b), Ordering::Less);

        let a = Point { x: 1, y: 2 };
        let b = Point { x: 3, y: -5 };
        assert_eq!(Point::cmp_y_x(&a, &b), Ordering::Greater);

        let a = Point { x: 2, y: 1 };
        let b = Point { x: 4, y: 1 };
        assert_eq!(Point::cmp_y_x(&a, &b), Ordering::Less);

        let a = Point { x: 2, y: 1 };
        let b = Point { x: -5, y: 1 };
        assert_eq!(Point::cmp_y_x(&a, &b), Ordering::Greater);
    }

    #[test]
    fn test_add() {
        let a = Point { x: 0, y: 0 };
        let b = Point { x: 0, y: 0 };
        assert_eq!(a + b, Point { x: 0, y: 0 });

        let a = Point { x: 1, y: 1 };
        let b = Point { x: 2, y: 2 };
        assert_eq!(a + b, Point { x: 3, y: 3 });

        let a = Point { x: 1, y: 1 };
        let b = Point { x: -1, y: -1 };
        assert_eq!(a + b, Point { x: 0, y: 0 });

        let a = Point {
            x: 1_000_000_000,
            y: -1_000_000_000,
        };
        let b = Point { x: -1, y: 1 };
        assert_eq!(
            a + b,
            Point {
                x: 999_999_999,
                y: -999_999_999
            }
        );
    }

    #[test]
    fn test_sub() {
        let a = Point { x: 0, y: 0 };
        let b = Point { x: 0, y: 0 };
        assert_eq!(a - b, Point { x: 0, y: 0 });

        let a = Point { x: 1, y: 1 };
        let b = Point { x: 2, y: 2 };
        assert_eq!(a - b, Point { x: -1, y: -1 });

        let a = Point { x: 1, y: 1 };
        let b = Point { x: -1, y: -1 };
        assert_eq!(a - b, Point { x: 2, y: 2 });

        let a = Point {
            x: 1_000_000_000,
            y: -1_000_000_000,
        };
        let b = Point { x: -1, y: 1 };
        assert_eq!(
            a - b,
            Point {
                x: 1_000_000_001,
                y: -1_000_000_001
            }
        );

        let a = Point {
            x: 0x7FFFFFFF,
            y: -0x7FFFFFFF,
        };
        let b = Point {
            x: 0x7FFFFFFF,
            y: -0x7FFFFFFF,
        };
        let c = Point {
            x: 0x7FFFFFFF,
            y: -0x7FFFFFFF,
        };
        assert_eq!(
            a - b - c,
            Point {
                x: -0x7FFFFFFF,
                y: 0x7FFFFFFF
            }
        );
    }

    #[test]
    fn test_get_range() {
        let points = Vec::new();
        let range = Point::get_range(&points);
        assert_eq!(range, None);

        let points = vec![Point { x: 0, y: 0 }];
        let range = Point::get_range(&points);
        assert_eq!(range, Some(((0, 0), (0, 0))));

        let points = vec![
            Point { x: -5, y: 0 },
            Point { x: 0, y: 7 },
            Point { x: 4, y: 4 },
        ];
        let range = Point::get_range(&points);
        assert_eq!(range, Some(((-5, 4), (0, 7))));

        let points = vec![
            Point { x: 24, y: -86 },
            Point { x: -80, y: 33 },
            Point { x: 16, y: -81 },
            Point { x: 59, y: 14 },
            Point { x: -97, y: -7 },
            Point { x: 73, y: -40 },
            Point { x: 16, y: -29 },
            Point { x: 5, y: 69 },
            Point { x: 2, y: 22 },
        ];
        let range = Point::get_range(&points);
        assert_eq!(range, Some(((-97, 73), (-86, 69))));

        let points = vec![
            Point {
                x: 311147,
                y: 388530,
            },
            Point {
                x: 459992,
                y: 742648,
            },
            Point {
                x: 307738,
                y: 247421,
            },
            Point {
                x: 132960,
                y: 182207,
            },
            Point {
                x: 822741,
                y: 727272,
            },
            Point {
                x: 979388,
                y: 603831,
            },
            Point {
                x: 784738,
                y: 563251,
            },
            Point {
                x: 696914,
                y: 315058,
            },
            Point {
                x: 449283,
                y: 180916,
            },
        ];
        let range = Point::get_range(&points);
        assert_eq!(range, Some(((132960, 979388), (180916, 742648))));
    }
}
