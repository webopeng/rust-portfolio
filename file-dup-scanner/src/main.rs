use blake3::Hasher;
use clap::Parser;
use rayon::prelude::*;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};
use walkdir::WalkDir;
use itertools::Itertools;

const MIN_SIZE: u64 = 11 * 1024 * 1024; // 11 MB

#[derive(Debug, Clone)]
struct FileEntry {
    path: PathBuf,
    size: u64,
}

#[derive(Parser, Debug)]
#[command(
    name = "dupe-scan",
    version,
    about = "Finds duplicate files across all active Windows drives"
)]

struct Args {
    #[arg(long, default_value = "11MB")]
    min_size: String,

    #[arg(long)]
    json: bool,
    
    #[arg(long)]
    limit: Option<usize>,
}

fn get_windows_drives() -> Vec<String> {
    let mut drives = Vec::new();
    for letter in b'A'..=b'Z' {
        let drive = format!("{}:\\", letter as char);
        if PathBuf::from(&drive).exists() {
            drives.push(drive);
        }
    }
    drives
}

fn hash_file(path: &PathBuf) -> Option<String> {
    let file = File::open(path).ok()?;
    let mut reader = BufReader::new(file);
    let mut hasher = Hasher::new();

    let mut buf = vec![0u8; 4 * 1024 * 1024]; // 4 MB chunks
    loop {
        let n = reader.read(&mut buf).ok()?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    Some(hasher.finalize().to_hex().to_string())
}

fn parse_size(s: &str) -> u64 {
    let s = s.to_uppercase();
    if let Some(v) = s.strip_suffix("MB") {
        return v.trim().parse::<u64>().unwrap() * 1024 * 1024;
    }
    if let Some(v) = s.strip_suffix("GB") {
        return v.trim().parse::<u64>().unwrap() * 1024 * 1024 * 1024;
    }
    s.parse::<u64>().unwrap()
}

fn main() {
    let args = Args::parse();
    let min_size_bytes = parse_size(&args.min_size);
    let limit = args.limit;

    let drives = get_windows_drives();
    println!("Scanning drives: {:?}", drives);

    let mut files: Vec<FileEntry> = drives
        .par_iter()
        .flat_map(|drive| {
            let mut count = 0usize;
            WalkDir::new(drive)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
                .take_while(|entry| {
                    // Only increment when we see a file
                    if entry.file_type().is_file() {
                        if let Some(max) = limit {
                            if count >= max {
                                return false; // stop iterating this drive
                            }
                        } 
                        count += 1;
                    }
                    true
                })
                .filter(|e| e.file_type().is_file())
                .filter_map(|entry| {
                    let md = entry.metadata().ok()?;
                    if md.len() >= min_size_bytes {
                        

                        Some(FileEntry {
                            path: entry.path().to_path_buf(),
                            size: md.len(),
                        })
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>() // each thread returns its own Vec
        })
        .collect(); // flattened into one Vec

    // Apply smoke test limit if specified
    if let Some(limit) = args.limit {
        files.truncate(limit);
        println!("Limiting to first {} files for testing", limit);
    }
    // 2. Hash in parallel

    let map: HashMap<String, Vec<FileEntry>> = files
        .par_iter()
        .fold(
            || HashMap::new(),
            |mut local_map, f| {
                if let Some(h) = hash_file(&f.path) {
                    local_map.entry(h).or_insert_with(Vec::new).push(f.clone());
                }
                local_map
            },
        )
        .reduce(
            || HashMap::new(),
            |mut a, mut b| {
                for (hash, mut vec_b) in b.drain() {
                    a.entry(hash).or_insert_with(Vec::new).append(&mut vec_b);
                }
                a
            },
        );

    // 3. Compute duplicates + output
    let mut total_dupe_size: u64 = 0;
    let mut dupe_list = String::new();
    let mut delete_script = String::new();

    for (_hash, group) in map.iter() {
        if group.len() > 1 {
            let keeper = &group[0];

            dupe_list.push_str(&format!("KEEP: {}\n", keeper.path.display()));

            for dup in group.iter().skip(1) {
                total_dupe_size += dup.size;
                dupe_list.push_str(&format!("DUP : {}\n", dup.path.display()));
                delete_script.push_str(&format!(
                    "Remove-Item -LiteralPath \"{}\" -Force\n",
                    dup.path.display()
                ));
            }

            dupe_list.push('\n');
        }
    }

    std::fs::write("dupe_list.txt", dupe_list).unwrap();
    std::fs::write("delete_dupes.ps1", delete_script).unwrap();

    println!("Total duplicate size: {} bytes", total_dupe_size);
}
