use vergen::{vergen, Config};

fn main() {
    // Generate the default 'cargo:' instruction output
    vergen(Config::default()).expect("Unable to generate build-time environment variables");
}
