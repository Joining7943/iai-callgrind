use benchmark_tests::find_primes;
use iai_callgrind::{
    library_benchmark, library_benchmark_group, main, EntryPoint, LibraryBenchmarkConfig, Tool,
    ValgrindTool,
};

#[library_benchmark(
    config = LibraryBenchmarkConfig::default()
        .entry_point(EntryPoint::None)
        .raw_callgrind_args(["--fair-sched=yes"])
        .tool(Tool::new(ValgrindTool::DHAT)
            .args(["--trace-children=yes"])
            .outfile_modifier("%p"))
)]
#[bench::two(4)]
#[bench::three(3)]
fn bench_library(num: u64) {
    let mut handles = vec![];
    let mut low = 0;
    for _ in 0..num {
        let handle = std::thread::spawn(move || find_primes(low, low + 10000));
        handles.push(handle);

        low += 10000;
    }

    let mut primes = vec![];
    for handle in handles {
        let result = handle.join();
        primes.extend(result.unwrap())
    }

    println!(
        "Number of primes found in the range 0 to {low}: {}",
        primes.len()
    );
}

library_benchmark_group!(name = my_group; benchmarks = bench_library);
main!(library_benchmark_groups = my_group);