use std::fs::File;
use std::io::Read;

fn main() {
    let mut args = std::env::args_os().skip(1);
    let file = args.next().unwrap();
    let expected = args.next().unwrap();

    let mut file = File::open(file).unwrap();
    let mut actual = String::new();
    file.read_to_string(&mut actual).unwrap();

    assert_eq!(actual, expected.to_string_lossy());
}
