//! Comparison of unsigned Variable Integer encodings.
//!
//! This compares the [mutiformat's unsigned-varint
//! encoding](https://github.com/multiformats/unsigned-varint) from the [unsigned-varint
//! crate](https://crates.io/crates/unsigned-varint) with [QUIC's Variable-Length Integer
//! Encoding](https://www.rfc-editor.org/rfc/rfc9000.html#name-variable-length-integer-enc).
//!
//! This benchmark limits itself to encoding and decoding u64.

use varint_bench::VarInt;

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};

pub fn encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("Encoding");
    group.bench_function("multiformat", |bencher| {
        bencher.iter_batched(
            // setup
            || (7u64, [0u8; 10]),
            // routine
            |(n, mut buf)| {
                unsigned_varint::encode::u64(black_box(n), &mut buf);
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("quic", |bencher| {
        bencher.iter_batched(
            // setup
            || (VarInt::from(7u8), [0u8; 8]),
            // routine
            |(n, mut buf)| {
                n.encode(&mut buf[..]).unwrap();
            },
            BatchSize::SmallInput,
        )
    });
}

pub fn decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("Decoding");
    group.bench_function("multiformat", |bencher| {
        bencher.iter_batched(
            // setup
            || {
                let mut buf = [0u8; 10];
                let slice = unsigned_varint::encode::u64(7, &mut buf);
                let mut buf = Vec::with_capacity(slice.len());
                buf.extend_from_slice(slice);
                buf
            },
            // routine
            |buf| {
                unsigned_varint::io::read_u64(buf.as_slice()).unwrap();
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("quic", |bencher| {
        bencher.iter_batched(
            // setup
            || {
                let mut buf = Vec::with_capacity(8);
                let n = VarInt::from(7u8);
                n.encode(&mut buf).unwrap();
                buf
            },
            // routine
            |buf| VarInt::decode(&buf[..]).unwrap(),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, encode, decode);
criterion_main!(benches);
