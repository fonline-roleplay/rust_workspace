use euclid::{Point2D, Rect, Size2D};

struct PlaneSpace;
type PlaneScalar = f64;
type PlaneRect = Rect<PlaneScalar, PlaneSpace>;
type PlaneSize = Size2D<PlaneScalar, PlaneSpace>;
type PlanePoint = Point2D<PlaneScalar, PlaneSpace>;

#[derive(Debug)]
struct MultiPlane {
    planes: Vec<Plane>,
}

// as in "plane of existence"
#[derive(Debug)]
struct Plane {
    name: String,
    size: PlaneSize,
    patches: Vec<Patch>,
}

// as in "patches of land"
#[derive(Debug, Default)]
struct Location {
    proto: u16,
    id: Option<u32>,
}

impl Location {
    fn with_proto(proto: u16) -> Self {
        Self {
            proto,
            ..Default::default()
        }
    }
}

#[derive(Debug)]
struct Patch {
    geometry: Geometry,
    locations: Locations,
}

#[derive(Debug)]
enum Locations {
    Single(Location),
    Grid(Grid<Cell>),
    UniformGrid(Grid<Option<Location>>),
}

impl Locations {
    fn uniform_grid(width: u16, cells: Vec<Option<Location>>) -> Self {
        let height = (cells.len() as u16 + width - 1) / width;
        Locations::UniformGrid(Grid {
            size: (width, height).into(),
            cells,
        })
    }
}

#[derive(Debug)]
struct Geometry {
    size: OuterSize,
    position: PlanePoint,
}

#[derive(Debug)]
struct Grid<C> {
    size: Size2D<u16, ()>,
    cells: Vec<C>,
}

#[derive(Debug)]
struct Cell {
    location: Location,
    rect: Rect<u16, ()>,
}

#[derive(Debug)]
enum OuterSize {
    Scale(PlaneScalar),
    Border(PlaneSize),
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn print_debug() {
        let single = Patch {
            geometry: Geometry {
                size: OuterSize::Border((100.0, 100.0).into()),
                position: (1000.0, 1000.0).into(),
            },
            locations: Locations::Single(Location::with_proto(100)),
        };
        let locations = {
            const WIDTH: u16 = 5;
            const HEIGHT: u16 = 5;
            let mut cells = Vec::with_capacity((WIDTH * HEIGHT) as usize);
            let mut i = 1;
            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    cells.push(Cell {
                        location: Location::with_proto(i),
                        rect: Rect::new((x, y).into(), (1, 1).into()),
                    });
                    i += 1;
                }
            }
            Locations::Grid(Grid {
                size: (WIDTH, HEIGHT).into(),
                cells,
            })
        };
        let grid = Patch {
            geometry: Geometry {
                size: OuterSize::Scale(1.0),
                position: (2000.0, 1000.0).into(),
            },
            locations,
        };
        let uniform_grid = Patch {
            geometry: Geometry {
                size: OuterSize::Scale(1.0),
                position: (1000.0, 2000.0).into(),
            },
            locations: Locations::uniform_grid(
                3,
                vec![
                    None,
                    Some(Location::with_proto(200)),
                    None,
                    Some(Location::with_proto(201)),
                    Some(Location::with_proto(202)),
                    Some(Location::with_proto(203)),
                    None,
                    Some(Location::with_proto(204)),
                    None,
                ],
            ),
        };
        let world = MultiPlane {
            planes: vec![Plane {
                name: "Overworld".into(),
                size: (1_000_000.0, 1_000_000.0).into(),
                patches: vec![single, grid],
            }],
        };
        println!("{:?}", world);
    }

    #[test]
    fn uniform_height() {
        for height in 1..10 {
            for width in 1..10 {
                let cells: Vec<_> = (0..(width * height))
                    .map(|i| Some(Location::with_proto(i)))
                    .collect();
                if let Locations::UniformGrid(uniform_grid) = Locations::uniform_grid(width, cells)
                {
                    assert_eq!(uniform_grid.size.height, height);
                    assert_eq!(uniform_grid.size.width, width);
                } else {
                    unreachable!();
                }
            }
        }
    }
}

#[rustfmt::skip]
const FORP_OVERWORLD: &[u16] = &[
     0, 28,  0,  0,  0,
     1,  2,  3,  4,  5,
     6,  7,  8,  9, 10, 
    11, 12, 13, 14, 15,
    16, 17, 18, 19, 20,
    21, 22, 23, 24, 25,
];
