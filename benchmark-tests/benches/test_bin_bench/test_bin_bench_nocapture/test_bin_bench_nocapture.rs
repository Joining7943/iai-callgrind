use std::path::PathBuf;

use iai_callgrind::{self, binary_benchmark, binary_benchmark_group, main, Pipe, Stdin, Stdio};

const ECHO: &str = env!("CARGO_BIN_EXE_echo");
const PIPE: &str = env!("CARGO_BIN_EXE_pipe");

fn setup() {
    print!("Something");
}

#[binary_benchmark]
fn bench_echo() -> iai_callgrind::Command {
    iai_callgrind::Command::new(ECHO)
        .args(["1", "2"])
        .stdout(Stdio::File(PathBuf::from("bench.out")))
        .stderr(Stdio::Null)
        .build()
}

#[binary_benchmark(setup = setup())]
fn bench_pipe() -> iai_callgrind::Command {
    iai_callgrind::Command::new(PIPE)
        .stdin(Stdin::Setup(Pipe::Stdout))
        .stdout(Stdio::Inherit)
        .stderr(Stdio::Null)
        .build()
}

binary_benchmark_group!(
    name = simple;
    benchmarks = bench_echo, bench_pipe
);

main!(binary_benchmark_groups = simple);