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

fn rand_u62() -> u64 {
    loop {
        let n: u64 = rand::random();
        if n <= 2u64.pow(62) {
            return n;
        }
    }
}

pub fn encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("Encoding");
    group.bench_function("multiformat", |bencher| {
        bencher.iter_batched(
            // setup
            || {
                let n = rand_u62();
                (n, [0u8; 10])
            },
            // routine
            |(n, mut buf)| {
                // returns a slice of buf: nothing to drop
                unsigned_varint::encode::u64(black_box(n), &mut buf);
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("quic", |bencher| {
        bencher.iter_batched(
            // setup
            || {
                let n = rand_u62();
                (VarInt::try_from(n).unwrap(), [0u8; 8])
            },
            // routine
            |(n, mut buf)| {
                // returns unit: nothing to drop
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
                let n = rand_u62();
                let mut buf = [0u8; 10];
                let slice = unsigned_varint::encode::u64(n, &mut buf);
                let mut buf = Vec::with_capacity(slice.len());
                buf.extend_from_slice(slice);
                buf
            },
            // routine
            |buf| {
                // returns u64 on stack: nothing to drop
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
                let n = rand_u62();
                let n = VarInt::try_from(n).unwrap();
                n.encode(&mut buf).unwrap();
                buf
            },
            // routine
            |buf| {
                // returns VarInt(u64), same layout as u64: nothing to drop
                VarInt::decode(&buf[..]).unwrap();
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, encode, decode);
criterion_main!(benches);
