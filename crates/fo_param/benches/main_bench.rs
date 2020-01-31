use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;

#[derive(Clone, Copy, Debug)]
struct Time {
    hour: u16,
    minute: u16,
    second: u16,
}

fn is_night_time_old(Time { hour, minute, .. }: Time) -> bool {
    if hour < 6 || hour > 18 {
        true
    } else if hour == 6 && minute == 0 {
        true
    } else if hour == 18 && minute > 0 {
        true
    } else {
        false
    }
}

fn is_night_time_new(time: Time) -> bool {
    let full_minute = (time.hour * 60 + time.minute) * 60;
    full_minute <= 6 * 3600 || 18 * 3600 < full_minute
}
/*
fn is_night_time_new2(Time { hour, minute }: Time) -> bool {
    let full_minute = hour * 60 + minute;
    let full_minute = full_minute.overflowing_sub(6 * 60 + 1).0;
    !(full_minute < (18 * 60 - 6 * 60))
}
*/
fn time_generator() -> impl FnMut() -> Time {
    let mut i: u16 = 0xABCD;
    move || {
        let hour = (i / 60) % 24;
        let minute = i % 60;
        i = i.overflowing_add(0x0101).0;
        //i = i ^ 0x1010;
        //i = i.overflowing_add(1).0;
        Time {
            hour,
            minute,
            second: i % 60,
        }
    }
}

fn test_night_time() {
    let mut generator = time_generator();
    for _ in 0..=24 * 60 {
        let time = generator();
        let old = is_night_time_old(time);
        let new = is_night_time_new(time);
        if old != new {
            panic!("test_night_time: {:?}; old: {}; new: {}", time, old, new);
        }
    }
    println!("test_night_time passed");
}

fn bench_night_time(c: &mut Criterion) {
    //test_night_time();
    let mut group = c.benchmark_group("night_time");
    //group.warm_up_time(Duration::from_secs(5));
    //group.sample_size(10);
    //group.measurement_time(Duration::from_secs(10));
    {
        let mut generator = time_generator();
        group.bench_function("night_time_old", |b| {
            b.iter(|| is_night_time_old(generator()))
        });
    }
    {
        let mut generator = time_generator();
        group.bench_function("night_time_new", |b| {
            b.iter(|| is_night_time_new(generator()))
        });
    }

    group.finish();
}

criterion_group!(benches, bench_night_time);
criterion_main!(benches);
