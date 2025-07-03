// this_file: svgn/benches/optimization.rs

//! Benchmarks for SVG optimization performance

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use svgn::{optimize_default, optimize_with_config, Config};

const SIMPLE_SVG: &str = r#"<svg width="100" height="100">
    <rect x="10" y="10" width="50" height="50" fill="red"/>
</svg>"#;

const COMPLEX_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200">
    <!-- This is a comment -->
    <defs>
        <linearGradient id="grad1" x1="0%" y1="0%" x2="100%" y2="0%">
            <stop offset="0%" style="stop-color:rgb(255,255,0);stop-opacity:1" />
            <stop offset="100%" style="stop-color:rgb(255,0,0);stop-opacity:1" />
        </linearGradient>
    </defs>
    <g transform="translate(10, 10)">
        <rect x="0" y="0" width="100" height="100" fill="url(#grad1)"/>
        <circle cx="50" cy="50" r="20" fill="blue"/>
        <text x="50" y="150" text-anchor="middle">Hello World</text>
    </g>
</svg>"#;

fn bench_simple_optimization(c: &mut Criterion) {
    c.bench_function("optimize simple svg", |b| {
        b.iter(|| optimize_default(black_box(SIMPLE_SVG)))
    });
}

fn bench_complex_optimization(c: &mut Criterion) {
    c.bench_function("optimize complex svg", |b| {
        b.iter(|| optimize_default(black_box(COMPLEX_SVG)))
    });
}

fn bench_with_pretty_printing(c: &mut Criterion) {
    let mut config = Config::with_default_preset();
    config.js2svg.pretty = true;

    c.bench_function("optimize with pretty printing", |b| {
        b.iter(|| optimize_with_config(black_box(SIMPLE_SVG), black_box(config.clone())))
    });
}

fn bench_multipass_optimization(c: &mut Criterion) {
    let mut config = Config::with_default_preset();
    config.multipass = true;

    c.bench_function("multipass optimization", |b| {
        b.iter(|| optimize_with_config(black_box(COMPLEX_SVG), black_box(config.clone())))
    });
}

criterion_group!(
    benches,
    bench_simple_optimization,
    bench_complex_optimization,
    bench_with_pretty_printing,
    bench_multipass_optimization
);
criterion_main!(benches);
