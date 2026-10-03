//! The `pa3ds` executable: everything it does is `pa3ds::__main__`, where the tests reach it.

fn main() -> std::process::ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    std::process::ExitCode::from(pa3ds::__main__::main(&argv) as u8)
}
