extern crate getopts;
use getopts::Options;
use std::env;
mod config;


fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("v", "verbose", "");
    opts.optopt("o", "output", "output database", "");
    opts.optopt("c", "config", "toml file describing the library files to analyze the file", "");
    opts.optopt("a", "analyzer_dir", "the directory of the analyzer.", "");
    opts.optopt("t", "target", "root directory of extracted firmware", "");
    
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { 
            panic!("{}", f);
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    if matches.opt_present("v") {
        //
    }

    let output: String = matches.opt_str("o").unwrap();
    let config: Option<String> = matches.opt_str("c");
    let analyzer_dir: Option<String> = matches.opt_str("a");
    let target: String = matches.opt_str("t").unwrap();

}
