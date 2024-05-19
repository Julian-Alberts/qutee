use qutee::{AsPoint, Boundary, ConstCap, QuadTree};

use rand::{Rng, SeedableRng};

const ITEMS: usize = 1_000_000;

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
    let mut rng = rand::rngs::StdRng::seed_from_u64(10);
    (0..ITEMS)
        .into_iter()
        .map(move |i| (rng.gen_range(0..32_767), rng.gen_range(0..32_767), i))
        .map(|(x, y, value)| QuadTreeEntry {
            x,
            y,
            _value: value,
        })
}

fn insert_data(
    qt: &mut QuadTree<usize, QuadTreeEntry, ConstCap<16>>,
    data: impl Iterator<Item = QuadTreeEntry>,
) {
    data.for_each(|item| qt.insert(item).unwrap())
}

fn query_data(qt: &mut QuadTree<usize, QuadTreeEntry, ConstCap<16>>, area: Boundary<usize>) {
    let _ = qt.query(area);
}
