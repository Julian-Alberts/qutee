use qutee::{AsPoint, Boundary, ConstCap, QuadTree};

const RAW_DATA: &str = include_str!("test.csv");

pub struct QuadTreeEntry {
    x: usize,
    y: usize,
    _value: usize,
}

impl AsPoint<usize> for QuadTreeEntry {
    fn as_point(&self) -> qutee::Point<usize> {
        (self.x, self.y).into()
    }
}

fn main() {
    let data = parse_data();
    let mut tree = QuadTree::new_with_const_cap(Boundary::between_points((0, 0), (32_767, 32_767)));
    insert_data(&mut tree, data);
    query_data(&mut tree, Boundary::new((32, 32), 10_000, 20_000));
}

fn parse_data() -> impl Iterator<Item = QuadTreeEntry> {
    RAW_DATA.lines().map(|line| {
        let mut row = line.split(',').map(|i| i.parse::<usize>());
        let x = row.next().unwrap().unwrap();
        let y = row.next().unwrap().unwrap();
        let value = row.next().unwrap().unwrap();
        QuadTreeEntry {
            x,
            y,
            _value: value,
        }
    })
}

fn insert_data(
    qt: &mut QuadTree<usize, QuadTreeEntry, ConstCap<16>>,
    data: impl Iterator<Item = QuadTreeEntry>,
) {
    data.for_each(|item| qt.insert(item).unwrap())
}

fn query_data(qt: &mut QuadTree<usize, QuadTreeEntry, ConstCap<16>>, area: Boundary<usize>) {
    assert_eq!(qt.query(area).collect::<Vec<_>>().len(), 18476)
}
