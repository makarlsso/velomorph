use std::borrow::Cow;
use std::hint::black_box;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use velomorph::Morph;
#[cfg(not(feature = "janitor"))]
use velomorph::TryMorph;
#[cfg(feature = "janitor")]
use velomorph::{Janitor, TryMorph};

#[derive(Clone)]
pub struct RawInput<'a> {
    pub request_id: Option<u64>,
    pub user_tag: &'a str,
    pub metadata: Option<String>,
    pub payload: Option<Vec<u8>>,
}

#[derive(Morph, Debug)]
pub struct ProcessedEvent<'a> {
    pub request_id: u64,
    pub user_tag: Cow<'a, str>,
    pub metadata: Option<String>,
}

fn manual_transform_borrowed(input: RawInput<'_>) -> ProcessedEvent<'_> {
    ProcessedEvent {
        request_id: input.request_id.unwrap_or(0),
        // Mirrors the zero-copy shape that Morph generates for &str -> Cow<'_, str>.
        user_tag: Cow::Borrowed(input.user_tag),
        metadata: input.metadata,
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    #[cfg(feature = "janitor")]
    let janitor = Janitor::new();

    let raw = RawInput {
        request_id: Some(123),
        user_tag: "performance_test",
        metadata: Some("bench-meta".to_string()),
        payload: Some(vec![0u8; 1024 * 1024]), // 1MB payload
    };

    let mut morph_group = c.benchmark_group("MorphOnly_NoPayloadClone");

    morph_group.bench_function("Velomorph", |b| {
        b.iter_batched(
            || RawInput {
                request_id: Some(123),
                user_tag: "performance_test",
                metadata: Some("bench-meta".to_string()),
                payload: None,
            },
            |raw_no_payload| {
                let src = black_box(raw_no_payload);
                #[cfg(feature = "janitor")]
                let out: ProcessedEvent = src
                    .try_morph(&janitor)
                    .expect("morph should succeed in benchmark");
                #[cfg(not(feature = "janitor"))]
                let out: ProcessedEvent =
                    src.try_morph().expect("morph should succeed in benchmark");

                let _ = black_box(out);
            },
            BatchSize::SmallInput,
        )
    });

    morph_group.bench_function("ManualBorrowed", |b| {
        b.iter_batched(
            || RawInput {
                request_id: Some(123),
                user_tag: "performance_test",
                metadata: Some("bench-meta".to_string()),
                payload: None,
            },
            |raw_no_payload| {
                let src = black_box(raw_no_payload);
                let _ = black_box(manual_transform_borrowed(src));
            },
            BatchSize::SmallInput,
        )
    });

    morph_group.finish();

    // Isolate heavy payload clone/drop cost explicitly. This avoids mixing memory
    // movement with morph logic and makes benchmark conclusions less misleading.
    let mut clone_group = c.benchmark_group("PayloadCloneDrop_1MB");

    clone_group.bench_function("CloneRawInput", |b| {
        b.iter(|| {
            let cloned = black_box(raw.clone());
            black_box(cloned);
        })
    });

    clone_group.bench_function("ManualBorrowed_afterClone", |b| {
        b.iter(|| {
            let cloned = black_box(raw.clone());
            let _ = black_box(manual_transform_borrowed(cloned));
        })
    });

    clone_group.bench_function("Velomorph_afterClone", |b| {
        b.iter(|| {
            let cloned = black_box(raw.clone());
            #[cfg(feature = "janitor")]
            let out: ProcessedEvent = cloned
                .try_morph(&janitor)
                .expect("morph should succeed in benchmark");
            #[cfg(not(feature = "janitor"))]
            let out: ProcessedEvent = cloned
                .try_morph()
                .expect("morph should succeed in benchmark");
            let _ = black_box(out);
        })
    });

    clone_group.finish();

    // Benchmark list mapping through the blanket impl: Vec<T> -> Vec<U>.
    let mut vec_group = c.benchmark_group("VecMorph_NoPayloadClone");

    vec_group.bench_function("VelomorphVec_1k", |b| {
        b.iter_batched(
            || {
                (0..1_000)
                    .map(|idx| RawInput {
                        request_id: Some(idx),
                        user_tag: "performance_test",
                        metadata: Some("bench-meta".to_string()),
                        payload: None,
                    })
                    .collect::<Vec<_>>()
            },
            |batch| {
                let src = black_box(batch);
                #[cfg(feature = "janitor")]
                let out: Vec<ProcessedEvent> = src
                    .try_morph(&janitor)
                    .expect("vec morph should succeed in benchmark");
                #[cfg(not(feature = "janitor"))]
                let out: Vec<ProcessedEvent> = src
                    .try_morph()
                    .expect("vec morph should succeed in benchmark");
                let _ = black_box(out);
            },
            BatchSize::SmallInput,
        )
    });

    vec_group.bench_function("ManualVecBorrowed_1k", |b| {
        b.iter_batched(
            || {
                (0..1_000)
                    .map(|idx| RawInput {
                        request_id: Some(idx),
                        user_tag: "performance_test",
                        metadata: Some("bench-meta".to_string()),
                        payload: None,
                    })
                    .collect::<Vec<_>>()
            },
            |batch| {
                let src = black_box(batch);
                let out = src
                    .into_iter()
                    .map(manual_transform_borrowed)
                    .collect::<Vec<_>>();
                let _ = black_box(out);
            },
            BatchSize::SmallInput,
        )
    });

    vec_group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
