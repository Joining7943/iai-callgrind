fn main() {
    println!(
        "cargo:rustc-env=BUILD_TRIPLE={}",
        std::env::var("TARGET").unwrap()
    );
}
