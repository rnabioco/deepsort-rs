use flate2::read::MultiGzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::time::Instant;

fn main() -> io::Result<()> {
    let start_time = Instant::now();

    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: {} <barcode_file> <fastq_folder> <output_folder> [prefix]", args[0]);
        std::process::exit(1);
    }

    let barcode_file = &args[1];
    let fastq_folder = &args[2];
    let output_folder = &args[3];
    let prefix = args.get(4).map(|s| s.as_str()).unwrap_or("");

    let mut barcode_map: HashMap<String, String> = HashMap::new();

    let barcode_reader = BufReader::new(File::open(barcode_file)?);
    for line in barcode_reader.lines().skip(1) {
        let line = line?;
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 3 {
            barcode_map.insert(parts[1].to_string(), parts[2].to_string());
        }
    }

    println!("Number of read IDs loaded from demultiplex file: {}", barcode_map.len());

    let mut processed_reads: HashSet<String> = HashSet::new();
    let mut non_unique_read_ids: HashSet<String> = HashSet::new();
    let mut reads_without_barcode = 0;
    let mut file_encoders: HashMap<String, GzEncoder<File>> = HashMap::new();
    let mut count_map: HashMap<String, usize> = HashMap::new();
    let mut total_count = 0;

    for entry in fs::read_dir(Path::new(fastq_folder))? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("gz") {
            println!("Processing {}", path.display());

            let file = File::open(&path)?;
            let decoder = MultiGzDecoder::new(file);
            let reader = BufReader::new(decoder);

            for chunk in reader.lines().collect::<Result<Vec<_>, _>>()?.chunks_exact(4) {
                if chunk.len() != 4 {
                    eprintln!("Incomplete read found: {:?}", chunk);
                    continue;
                }

                let header = &chunk[0];
                let read_id = header.split(' ').next().unwrap().trim_start_matches('@');

                if !processed_reads.insert(read_id.to_string()) {
                    non_unique_read_ids.insert(read_id.to_string());
                }

                if !barcode_map.contains_key(read_id) {
                    reads_without_barcode += 1;
                    continue;
                }

                total_count += 1;
                let seq = &chunk[1];
                let plus = &chunk[2];
                let qual = &chunk[3];

                let barcode = barcode_map.get(read_id).map(|s| s.as_str()).unwrap_or_else(|| "unknown");
                let output_filename = format!("{}{}.fastq.gz", prefix, barcode);
                let output_path = Path::new(output_folder).join(output_filename);

                let encoder = file_encoders.entry(barcode.to_string())
                    .or_insert_with(|| {
                        GzEncoder::new(
                            OpenOptions::new()
                                .create(true)
                                .write(true)
                                .truncate(true)
                                .open(&output_path)
                                .expect("Failed to open output file"),
                            Compression::default())
                    });

                *count_map.entry(barcode.to_string()).or_insert(0) += 1;
                writeln!(encoder, "{}\n{}\n{}\n{}", header, seq, plus, qual)?;
            }
        }
    }

    let duration = start_time.elapsed();

    println!("Sorting complete. Time taken: {:?}", duration);
    println!("Total number of reads: {}", total_count);
    println!("Number of non-unique read IDs in FASTQ files: {}", non_unique_read_ids.len());
    println!("Number of reads without a corresponding barcode: {}", reads_without_barcode);
    println!("Summary:");
    for (barcode, count) in count_map.iter() {
        let percentage = (*count as f64) / (total_count as f64) * 100.0;
        println!("{}: {:.2}% ({} reads)", barcode, percentage, count);
    }

    Ok(())
}
