use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;

use fo_data::crawler::gather_paths;

fn bench_gather_paths(c: &mut Criterion) {
    let mut group = c.benchmark_group("gather_paths");
    group.warm_up_time(Duration::from_secs(5));
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(10));
    {
        group.bench_function("gather", |b| {
            b.iter(|| gather_paths(black_box("../../CL4RP")))
        });
    }
    /*{
        let modoc_lands_town_dn_path = "../../FO4RP/maps/modoc_lands_town_dn.fomap";
        let modoc_lands_town_dn_bytes =
            std::fs::read(modoc_lands_town_dn_path).expect("Can't open map file");
        let modoc_lands_town_dn_text = String::from_utf8_lossy(&modoc_lands_town_dn_bytes);
        group.bench_function("modoc_lands_town_dn", |b| {
            b.iter(|| parse_map(black_box(&modoc_lands_town_dn_text)))
        });
    }*/

    group.finish();
}

criterion_group!(benches, bench_gather_paths);
criterion_main!(benches);
