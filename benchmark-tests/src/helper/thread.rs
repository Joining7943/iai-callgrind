use benchmark_tests::find_primes;

fn main() {
    let mut args_iter = std::env::args().skip(1);

    let num = args_iter.next().map_or(0, |a| a.parse::<usize>().unwrap());

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
