extern crate rand;
use rand::{
    Rng,
    seq::IteratorRandom};
use std::{
    env,
    fs,
    process,
    error::Error,
    cmp,
    io::{BufRead, BufReader}};

fn main() {
    let args: Vec<String> = env::args().collect(); // vec of arguments, 1st is always path to binary

    let config = Config::new(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    if let Err(e) = run(config) {
        println!("Application error: {}", e);
        process::exit(1);
    }
    
} 

fn run(config: Config) -> Result<(), Box<dyn Error>> { // runs the password generator, with a given configuration
    
    // help output, does not return anything besides help
    if config.help {
        println!("usage: xkcdpwgen [-h] [-w WORDS] [-c CAPS] [-n NUMBERS] [-s SYMBOLS]
                
                Generate a secure, memorable password using the XKCD method
                                
                optional arguments:
                    -h, --help            show this help message and exit
                    -d, --debug           include debug info in the output
                    -w WORDS, --words WORDS
                                          include WORDS words in the password (default=4)
                    -c CAPS, --caps CAPS  capitalize the first letter of CAPS random words
                                          (default=0)
                    -n NUMBERS, --numbers NUMBERS
                                          insert NUMBERS random numbers in the password
                                          (default=0)
                    -s SYMBOLS, --symbols SYMBOLS
                                          insert SYMBOLS random symbols in the password
                                          (default=0)");
        return Ok(());
    }

    // debug output
    if config.debug {
        println!("--DEBUG--");
        println!("  words: {}", config.words);
        println!("  caps: {}",  config.caps);
        println!("  nums: {}",  config.nums);
        println!("  syms: {}",  config.syms);
    }

    let total_capacity = config.words + config.nums + config.syms;
    let mut password = Vec::<String>::with_capacity(total_capacity);

    // pick words from words.txt
    for _ in 0..config.words {
        password.push(choose_word());
    }

    // capitalize c random words
    let mut c = config.caps;
    let mut c_words: Vec<bool> = vec![false;config.words];
    while c > 0 {
        let r = rand::thread_rng().gen_range(0..config.words);
        if c_words[r] { // if the word at r is already capitalized, generate a new r
            continue;
        }
        c_words[r] = true;
        password[r] = capitalize(password[r].as_str());
        c = c - 1;
    }

    // insert n random digits
    let mut n = config.nums;
    while n > 0 {
        let r = rand::thread_rng().gen_range(0..=password.len());
        password.insert(r, String::from(rand_digit()));
        n = n - 1;
    }

    // insert n random symbols
    let mut s = config.syms;
    while s > 0 {
        let r = rand::thread_rng().gen_range(0..=password.len());
        password.insert(r, String::from(rand_symbol()));
        s = s - 1;
    }

    // print final password
    println!("{}", password.join(""));
    Ok(())
}

struct Config { // stores configuration data
    words: usize,  // number of words
    caps: usize,   // number of capitalized words
    nums: usize,   // number of inserted digits
    syms: usize,   // number of inserted symbols
    help: bool,    // help option
    debug: bool,   // debug option 
}

impl Config { // config implementations
    fn new(args: &[String]) -> Result<Config, &str> {
        let mut words = 4;
        let mut caps = 0;
        let mut nums = 0;
        let mut syms = 0;
        let mut help = false;
        let mut debug = false;
        
        let n = args.len(); // number of arguments

        if n == 1 { // no extra args, default config
            return Ok(Config { words, caps, nums, syms, help, debug });
        }

        let mut i = 1; // loop variable
        while i < n { // iterate over extra args (range is not inclusive, so goes from a[1] to a[n-1])

            // words case
            if args[i] == "-w" || args[i] == "--words" { 
                if (i+1) < n { // check if we can access a[i+1], and that it is a valid argument
                    if let Ok(w) = args[i+1].parse::<usize>() {
                        words = w;
                        i = i + 2;
                    } else {
                        return Err("invalid parameter for option -w");
                    }
                } else {
                    return Err("no parameter for option -w");
                }
            }

            // caps case
            else if args[i] == "-c" || args[i] == "--caps" { 
                if (i+1) < n { // check if we can access a[i+1], and that it is a valid argument
                    if let Ok(c) = args[i+1].parse::<usize>() {
                        caps = cmp::min(words, c);
                        i = i + 2;
                    } else {
                        return Err("invalid parameter for option -c");
                    }
                } else {
                    return Err("no parameter for option -c");
                }
            }

            // nums case
            else if args[i] == "-n" || args[i] == "--numbers" { 
                if (i+1) < n { // check if we can access a[i+1], and that it is a valid argument
                    if let Ok(n_) = args[i+1].parse::<usize>() {
                        nums = n_;
                        i = i + 2;
                    } else {
                        return Err("invalid parameter for option -n");
                    }
                } else {
                    return Err("no parameter for option -n");
                }
            }

            // syms case
            else if args[i] == "-s" || args[i] == "--symbols" { 
                if (i+1) < n { // check if we can access a[i+1], and that it is a valid argument
                    if let Ok(s) = args[i+1].parse::<usize>() {
                        syms = s;
                        i = i + 2;
                    } else {
                        return Err("invalid parameter for option -s");
                    }
                } else {
                    return Err("no parameter for option -s");
                }
            }

            // help case
            else if args[i] == "-h" || args[i] == "--help" {
                help = true;
                i = i + 1;
            }

            // debug case
            else if args[i] == "-d" || args[i] == "--debug" {
                debug = true;
                i = i + 1;
            }
            
            // invalid arg case
            else {
                println!("invalid arg: {}", args[i]);
                return Err("invalid argument");
            }
        }
        
        Ok(Config { words, caps, nums, syms, help, debug })
    }
}

fn choose_word() -> String { // choose random word from words.txt
    const FILENAME: &str = "words.txt";

    let f = fs::File::open(FILENAME)
        .unwrap_or_else(|err| {
            println!("Problem reading {}: {}", FILENAME, err);
            process::exit(1);
        });
    let f = BufReader::new(f);

    let lines = f.lines().map(|l| l.expect("Couldn't read line"));

    lines
        .choose(&mut rand::thread_rng())
        .expect("File has no lines")
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn rand_digit() -> char { // returns a random digit
    char::from_digit(rand::thread_rng().gen_range(0..10), 10).unwrap()
}

fn rand_symbol() -> char { // returns a random symbol
    let symbols = String::from("~!@#$%^&*.:;");
    symbols.chars().nth(rand::thread_rng().gen_range(0..symbols.len())).unwrap()
}
