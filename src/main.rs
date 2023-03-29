use std::{env, process};
use std::fs::File;
use std::io::{BufRead, BufReader};

struct CacheLine {
    valid: bool,
    tag: u32,
    last_used: u32,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 5 {
        eprintln!("Usage: cargo run <trace_file> <s> <e> <b>");
        process::exit(1);
    }
    // Parse command-line arguments
    let s = args[1].parse::<u32>().unwrap();
    let e = args[2].parse::<u32>().unwrap();
    let b = args[3].parse::<u32>().unwrap();
    let file = &args[4];

    // Initialize cache data structures
    let num_sets = 2_u32.pow(s);
    let mut cache: Vec<Vec<CacheLine>> = Vec::with_capacity(num_sets as usize);
    for _ in 0..num_sets {
        let mut set: Vec<CacheLine> = Vec::with_capacity(e as usize);
        for _ in 0..e {
            set.push(CacheLine { valid: false, tag: 0, last_used: 0 });
        }
        cache.push(set);
    }

    // Simulate cache on memory trace file
    let file = File::open(file).unwrap();
    let reader = BufReader::new(file);
    let mut hits = 0;
    let mut misses = 0;
    let mut evictions = 0;
    let mut time = 0;
    for line in reader.lines() {
        let line = line.unwrap();
        if line.starts_with("I") {
            continue; // Ignore instruction cache accesses
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        let addr = u32::from_str_radix(parts[1], 16).unwrap();

        let tag = addr >> (s + b);
        let set_index = (addr >> b) & ((1 << s) - 1);
        let mut hit = false;
        for i in 0..e {
            let cache_line = &mut cache[set_index as usize][i as usize];
            if cache_line.valid && cache_line.tag == tag {
                hit = true;
                cache_line.last_used = time;
                hits += 1;
                break;
            }
        }
        if !hit {
            misses += 1;
            let mut evicted_line = None;
            for i in 0..e {
                let cache_line = &mut cache[set_index as usize][i as usize];
                if !cache_line.valid {
                    cache_line.valid = true;
                    cache_line.tag = tag;
                    cache_line.last_used = time;
                    break;
                } else if evicted_line == None || cache_line.last_used < cache[set_index as usize][evicted_line.unwrap() as usize].last_used {
                    evicted_line = Some(i);
                }
            }
            if evicted_line != None {
                cache[set_index as usize][evicted_line.unwrap() as usize].tag = tag;
                cache[set_index as usize][evicted_line.unwrap() as usize].last_used = time;
                evictions += 1;
            }
        }
        time += 1;
    }

    // Output results
    println!("hits:{} misses:{} evictions:{}", hits, misses, evictions);
}
