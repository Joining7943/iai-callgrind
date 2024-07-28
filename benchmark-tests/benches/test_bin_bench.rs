use iai_callgrind::{binary_benchmark, binary_benchmark_group, main, BinaryBenchmarkConfig};

#[binary_benchmark]
fn just_outer_attribute() -> iai_callgrind::Command {
    iai_callgrind::Command::new().arg("me").build()
}

#[binary_benchmark(config = BinaryBenchmarkConfig::default())]
fn bench_with_config() -> iai_callgrind::Command {
    iai_callgrind::Command::new().arg("happy").build()
}

#[binary_benchmark]
#[bench::some(1)]
fn bench(first: usize) -> iai_callgrind::Command {
    iai_callgrind::Command::new().arg(first.to_string()).build()
}

#[binary_benchmark]
#[benches::multiple_list(1, 2, 3)]
#[benches::multiple_args(args = [1, 2, 3], setup = my_mod::setup_me("hello there"))]
fn benches(first: usize) -> iai_callgrind::Command {
    iai_callgrind::Command::new().arg(first.to_string()).build()
}

mod my_mod {
    pub fn setup_me<T>(arg: T)
    where
        T: AsRef<str>,
    {
        println!("{}", arg.as_ref());
    }
}

fn setup(size: usize) {
    println!("setup: {size}");
}

fn teardown(size: usize) {
    println!("teardown: {size}");
}

binary_benchmark_group!(
    name = my_group;
    setup = setup(10);
    teardown = teardown(20);
    benchmarks = just_outer_attribute, bench_with_config, bench, benches
);

main!(binary_benchmark_groups = my_group);
