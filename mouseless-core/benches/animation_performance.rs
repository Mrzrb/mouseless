use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use mouseless_core::{
    animation::{AnimationInterpolator, EasingFunctions},
    models::{AnimationType, MovementSpeed, Position},
};
use std::time::Instant;

fn benchmark_easing_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("easing_functions");

    group.bench_function("linear", |b| {
        b.iter(|| {
            for i in 0..100 {
                let t = i as f32 / 100.0;
                black_box(EasingFunctions::linear(t));
            }
        })
    });

    group.bench_function("ease_out_cubic", |b| {
        b.iter(|| {
            for i in 0..100 {
                let t = i as f32 / 100.0;
                black_box(EasingFunctions::ease_out_cubic(t));
            }
        })
    });

    group.bench_function("ease_in_out_cubic", |b| {
        b.iter(|| {
            for i in 0..100 {
                let t = i as f32 / 100.0;
                black_box(EasingFunctions::ease_in_out_cubic(t));
            }
        })
    });

    group.bench_function("ease_out_bounce", |b| {
        b.iter(|| {
            for i in 0..100 {
                let t = i as f32 / 100.0;
                black_box(EasingFunctions::ease_out_bounce(t));
            }
        })
    });

    group.finish();
}

fn benchmark_animation_interpolation(c: &mut Criterion) {
    let mut group = c.benchmark_group("animation_interpolation");

    let start = Position::new(0, 0);
    let end = Position::new(1000, 1000);

    for speed in [
        MovementSpeed::Slow,
        MovementSpeed::Normal,
        MovementSpeed::Fast,
    ] {
        for animation_type in [
            AnimationType::Linear,
            AnimationType::Smooth,
            AnimationType::Bounce,
        ] {
            group.bench_with_input(
                BenchmarkId::new(format!("{:?}_{:?}", speed, animation_type), "interpolation"),
                &(speed, animation_type),
                |b, &(speed, animation_type)| {
                    b.iter(|| {
                        let interpolator =
                            AnimationInterpolator::new(start, end, speed, animation_type);

                        // Generate all animation positions
                        black_box(interpolator.get_animation_sequence());
                    })
                },
            );
        }
    }

    group.finish();
}

fn benchmark_single_step_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_step_performance");

    let start = Position::new(100, 100);
    let end = Position::new(500, 500);

    // Test that single animation steps complete within 10ms requirement
    group.bench_function("single_step_fast_smooth", |b| {
        b.iter(|| {
            let interpolator =
                AnimationInterpolator::new(start, end, MovementSpeed::Fast, AnimationType::Smooth);

            let step_start = Instant::now();
            let _position = black_box(interpolator.next_position(0));
            let step_duration = step_start.elapsed();

            // Ensure step calculation is well under 10ms
            assert!(
                step_duration.as_millis() < 1,
                "Step calculation took too long: {}ms",
                step_duration.as_millis()
            );
        })
    });

    group.bench_function("single_step_normal_bounce", |b| {
        b.iter(|| {
            let interpolator = AnimationInterpolator::new(
                start,
                end,
                MovementSpeed::Normal,
                AnimationType::Bounce,
            );

            let step_start = Instant::now();
            let _position = black_box(interpolator.next_position(5));
            let step_duration = step_start.elapsed();

            // Ensure step calculation is well under 10ms
            assert!(
                step_duration.as_millis() < 1,
                "Step calculation took too long: {}ms",
                step_duration.as_millis()
            );
        })
    });

    group.finish();
}

fn benchmark_full_animation_sequence(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_animation_sequence");

    let start = Position::new(0, 0);
    let end = Position::new(800, 600);

    group.bench_function("fast_linear_full_sequence", |b| {
        b.iter(|| {
            let interpolator =
                AnimationInterpolator::new(start, end, MovementSpeed::Fast, AnimationType::Linear);

            let sequence_start = Instant::now();
            let positions = black_box(interpolator.get_animation_sequence());
            let sequence_duration = sequence_start.elapsed();

            // Ensure full sequence generation is fast
            assert!(!positions.is_empty());
            assert!(
                sequence_duration.as_millis() < 5,
                "Full sequence generation took too long: {}ms",
                sequence_duration.as_millis()
            );
        })
    });

    group.bench_function("normal_smooth_full_sequence", |b| {
        b.iter(|| {
            let interpolator = AnimationInterpolator::new(
                start,
                end,
                MovementSpeed::Normal,
                AnimationType::Smooth,
            );

            let sequence_start = Instant::now();
            let positions = black_box(interpolator.get_animation_sequence());
            let sequence_duration = sequence_start.elapsed();

            // Ensure full sequence generation is fast
            assert!(!positions.is_empty());
            assert!(
                sequence_duration.as_millis() < 5,
                "Full sequence generation took too long: {}ms",
                sequence_duration.as_millis()
            );
        })
    });

    group.finish();
}

fn benchmark_performance_requirements(c: &mut Criterion) {
    let mut group = c.benchmark_group("performance_requirements");

    // Test the critical path: single animation step calculation and execution
    group.bench_function("critical_path_sub_10ms", |b| {
        b.iter(|| {
            let start = Position::new(200, 200);
            let end = Position::new(400, 300);

            let critical_start = Instant::now();

            // This simulates the critical path of animation step processing
            let interpolator =
                AnimationInterpolator::new(start, end, MovementSpeed::Fast, AnimationType::Smooth);

            let position = interpolator.next_position(0).unwrap();

            // Simulate the time it would take to move cursor (this is the bottleneck)
            // In real implementation, this would be the enigo.move_mouse call
            black_box(position);

            let critical_duration = critical_start.elapsed();

            // This is the key requirement: each step must be under 10ms
            assert!(
                critical_duration.as_millis() < 1,
                "Critical path took {}ms, must be under 10ms",
                critical_duration.as_millis()
            );
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_easing_functions,
    benchmark_animation_interpolation,
    benchmark_single_step_performance,
    benchmark_full_animation_sequence,
    benchmark_performance_requirements
);
criterion_main!(benches);
