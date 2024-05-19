use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use qutee::{AsPoint, Boundary, ConstCap, QuadTree};
use rand::{Rng as _, SeedableRng as _};

const ITEMS: usize = 1_000_000;

pub struct QuadTreeEntry {
    x: usize,
    y: usize,
    _value: usize,
}

impl AsPoint<usize> for &QuadTreeEntry {
    fn as_point(&self) -> qutee::Point<usize> {
        (self.x, self.y).into()
    }
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

fn criterion_benchmark(c: &mut Criterion) {
    let data = parse_data().collect::<Vec<_>>();

    let mut group = c.benchmark_group("insert");
    for i in [1_000, 10_000, 100_000, 1_000_000] {
        group.throughput(criterion::Throughput::Elements(i));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{} Elements", i)),
            &data[0..i as usize],
            |b, d| {
                b.iter(|| {
                    let mut tree: QuadTree<_, _, ConstCap<16>> = QuadTree::new_with_const_cap(
                        Boundary::between_points((0, 0), (32_767, 32_767)),
                    );
                    d.iter().for_each(|item| tree.insert(item).unwrap())
                });
            },
        );
    }
    group.finish();

    let mut group = c.benchmark_group("insert_unchecked");
    for i in [1_000, 10_000, 100_000, 1_000_000] {
        group.throughput(criterion::Throughput::Elements(i));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{} Elements", i)),
            &data[0..i as usize],
            |b, d| {
                b.iter(|| {
                    let mut tree: QuadTree<_, _, ConstCap<16>> = QuadTree::new_with_const_cap(
                        Boundary::between_points((0, 0), (32_767, 32_767)),
                    );
                    d.iter().for_each(|item| tree.insert_unchecked(item))
                });
            },
        );
    }
    group.finish();

    let mut tree: QuadTree<_, _, ConstCap<16>> =
        QuadTree::new_with_const_cap(Boundary::between_points((0, 0), (32_767, 32_767)));
    data.iter().for_each(|item| tree.insert(item).unwrap());
    let mut group = c.benchmark_group("query");
    for i in [
        ((0, 0), (32_767, 32_767)),
        ((500, 500), (25_000, 25_000)),
        ((15_000, 15_000), (20_000, 20_000)),
        ((500, 500), (510, 510)),
    ] {
        let area = (i.1 .0 - i.0 .0) * (i.1 .1 - i.0 .1);
        let query = Boundary::between_points(i.0, i.1);
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{query} Area: {area}")),
            &query,
            |b, q| b.iter(|| tree.query(q.clone()).count()),
        );
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
