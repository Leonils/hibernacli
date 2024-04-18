use hibernacli::run::run;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    run(args);
}
