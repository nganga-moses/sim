use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::process;
use getopts::Options;

#[derive(Debug)]
struct Set {
    blocks: Vec<Block>,
}
impl Set {
    fn new(size: u32) -> Set {
        Set {
            blocks: vec![Block {
                valid: false,
                tag: 0,
                last_used: 0,
            }; size as usize],
        }
    }
}

#[derive(Debug)]
struct Cache {
    sets: Vec<Set>,
    b: u32,
    s: u32,
    hits: u32,
    misses: u32,
    evictions: u32,
}


#[derive(Debug, Clone)]
struct Block {
    valid: bool,
    tag: u64,
    last_used: u32,
}

impl Cache {
    fn new(b: u32, s: u32, e: u32) -> Cache {
        let num_sets = 2u32.pow(s);
        let sets = (0..num_sets).map(|_| Set::new(e)).collect();
        Cache {
            sets,
            b,
            s,
            hits: 0,
            misses: 0,
            evictions: 0,
        }
    }

    fn access(&mut self, address: u64, is_modify: bool) {
        let (set_index, tag) = self.extract_tag_and_set_index(address);
        let set = &mut self.sets[set_index as usize];
        let mut hit_count = 0;
        for block in set.blocks.iter_mut() {
            if block.valid && block.tag == tag {
                // hit
                hit_count += 1;
                block.last_used = self.hits;
            }
        }
        if hit_count == 0{
            //miss
            self.misses += 1;
            let (_eviction_index, eviction_block) = match set
                .blocks
                .iter_mut()
                .enumerate()
                .find(|(_,block)|!block.valid)
            {
                Some((index,block)) => (index, block),
                None => {
                    let (index, block)=set
                        .blocks
                        .iter_mut()
                        .enumerate()
                        .min_by_key(|(_,block)|block.last_used)
                        .unwrap();
                    (index, block)
                }
            };
            if eviction_block.valid {
                self.evictions +=1;
            }
            eviction_block.valid = true;
            eviction_block.tag = tag;
            eviction_block.last_used = self.hits;
        } else if hit_count == 2 && is_modify {
            //modify
            self.hits += 1;
        } else {
            // hit
            self.hits += 1;
        }

    }

    fn extract_tag_and_set_index(&self, address: u64) -> (u64, u64) {
        let mask = (1 << self.s) - 1;
        let set_index = (address >> self.b) & mask;
        let tag = address >> (self.s + self.b);
        (set_index, tag)
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where
        P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut s = 0;
    let mut b = 0;
    let mut e = 0;
    let mut trace_file = String::new();

    if args.len() < 4 {
        eprintln!("Usage: cargo run <trace_file> <s> <e> <b>");
        process::exit(1);
    }
    let mut opts = Options::new();

    opts.optopt("s", "", "Number of set index bits", "S");
    opts.optopt("E", "", "Number of lines per set", "E");
    opts.optopt("b", "", "Number of block offset bits", "B");
    opts.optopt("t", "", "Name of valgrind trace file", "FILE");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!("{}",f.to_string());
        }
    };

    if let Some(s_val) = matches.opt_str("s") {
        s = s_val.parse::<u32>().unwrap();
    }
    if let Some(b_val) = matches.opt_str("b") {
        b = b_val.parse::<u32>().unwrap();
    }
    if let Some(e_val) = matches.opt_str("E") {
        e = e_val.parse::<u32>().unwrap();
    }
    if let Some(trace_val) = matches.opt_str("t") {
        trace_file = trace_val;
    }


    let mut cache = Cache::new(b, s, e);

    if let Ok(lines) = read_lines(trace_file) {
        for line in lines {
            if let Ok(ip) = line {
                let tokens: Vec<&str> = ip.split_whitespace().flat_map(|x| x.split(',')).collect();
                if tokens.len() != 3 {
                    eprintln!("Invalid input format");
                    process::exit(1);
                }
                let access_type = tokens[0];
                let address = u64::from_str_radix(tokens[1].trim_start_matches("0x"), 16).unwrap();
                match access_type {
                    "I" => continue,
                    "L" =>{
                        cache.access(address,false);
                    },
                    "S" =>{
                        cache.access(address,false);
                    } ,
                    "M" => {
                        // data modify - treated as a load followed by a store to the same address
                        cache.access(address,true);
                        cache.access(address,true);
                    },
                    _ => {
                        eprintln!("Invalid access type");
                        process::exit(1);
                    }
                }
            }
        }
    } else {
        eprintln!("Failed to read file");
        process::exit(1);
    }
    println!("hits:{} misses:{} evictions:{}", cache.hits,cache.misses, cache.evictions);

}
