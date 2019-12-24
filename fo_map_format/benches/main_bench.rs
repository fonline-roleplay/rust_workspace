use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fo_map_format::{root, Map};
use std::time::Duration;

fn parse_map(text: &str) -> Map {
    root::<nom::error::VerboseError<&str>>(text)
        .expect("Can't parse map")
        .1
}

fn parse_maps(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");
    group.warm_up_time(Duration::from_secs(5));
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(10));
    {
        let modoc_path = "../../FO4RP/maps/modoc.fomap";
        let modoc_bytes = std::fs::read(modoc_path).expect("Can't open map file");
        let modoc_text = String::from_utf8_lossy(&modoc_bytes);
        group.bench_function("modoc", |b| b.iter(|| parse_map(black_box(&modoc_text))));
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

fn bench_hashes(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashes");
    //group.sample_size(100);
    //group.measurement_time(std::time::Duration::from_secs(6));
    {
        group.bench_function("crc32", |b| {
            let mut i = 15325235325;
            b.iter(|| {
                i += 1;
                crc::crc32::checksum_ieee(black_box(&u128::to_le_bytes(i)))
            })
        });
    }
    {
        group.bench_function("crc32fast_new", |b| {
            fn crc32fast_new(buf: &[u8]) -> u32 {
                let mut hasher = crc32fast::Hasher::new();
                hasher.update(buf);
                hasher.finalize()
            }
            let mut i = 15325235325;
            b.iter(|| {
                i += 1;
                crc32fast_new(black_box(&u128::to_le_bytes(i)))
            })
        });
    }
    {
        group.bench_function("ahash", |b| {
            fn ahash(buf: &[u8]) -> u64 {
                use std::hash::Hasher;
                let mut hasher = ahash::AHasher::new_with_keys(1241241, 32234233);
                hasher.write(buf);
                hasher.finish()
            }
            let mut i = 15325235325;
            b.iter(|| {
                i += 1;
                ahash(black_box(&u128::to_le_bytes(i)))
            })
        });
    }

    group.finish();
}

criterion_group!(benches, parse_maps, bench_hashes);
criterion_main!(benches);
