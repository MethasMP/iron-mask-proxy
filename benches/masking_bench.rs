use criterion::{black_box, criterion_group, criterion_main, Criterion};
use iron_mask_proxy::masker::apply_global_standard_masking;

fn benchmark_thai_id_masking(c: &mut Criterion) {
    let input = "User ID: 1103700012346 and another 1103700012347";
    c.bench_function("thai_id_masking", |b| {
        b.iter(|| apply_global_standard_masking(black_box(input)))
    });
}

fn benchmark_phone_masking(c: &mut Criterion) {
    let input = "Contact: 0812345678, 0898765432, 0855555555";
    c.bench_function("phone_masking", |b| {
        b.iter(|| apply_global_standard_masking(black_box(input)))
    });
}

fn benchmark_email_masking(c: &mut Criterion) {
    let input = "Emails: user1@test.com, admin@company.co.th, info@example.org";
    c.bench_function("email_masking", |b| {
        b.iter(|| apply_global_standard_masking(black_box(input)))
    });
}

fn benchmark_mixed_pii(c: &mut Criterion) {
    let input = "User: สมชาย เข็มกลัด, ID: 1103700012346, Phone: 0812345678, Email: test@gmail.com";
    c.bench_function("mixed_pii_masking", |b| {
        b.iter(|| apply_global_standard_masking(black_box(input)))
    });
}

fn benchmark_large_payload(c: &mut Criterion) {
    // 10KB payload
    let input = "User: สมชาย, ID: 1103700012346, Phone: 0812345678 | ".repeat(200);
    c.bench_function("large_payload_10kb", |b| {
        b.iter(|| apply_global_standard_masking(black_box(&input)))
    });
}

criterion_group!(
    benches,
    benchmark_thai_id_masking,
    benchmark_phone_masking,
    benchmark_email_masking,
    benchmark_mixed_pii,
    benchmark_large_payload
);
criterion_main!(benches);
