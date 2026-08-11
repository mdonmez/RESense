#[cfg(feature = "dev-tools")]
fn main() {
    if let Err(error) = resense::developer::run(std::env::args().skip(1)) {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

#[cfg(not(feature = "dev-tools"))]
fn main() {
    eprintln!("the probe binary requires the dev-tools feature");
    std::process::exit(2);
}
