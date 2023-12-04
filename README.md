
# deepsort-rs

## Introduction

`deepsort` is a Rust-based command-line tool to quickly sort gzipped FASTQ files. It takes the demultiplexed calls from the [deeplexicon](https://github.com/Psy-Fer/deeplexicon) package and quickly sorts the reads.  

`deepsort` reads a barcode file, maps read IDs to barcodes, and then sorts and writes reads from gzipped FASTQ files into new gzipped files based on their barcodes.

## Installation

To install `deepsort`, you need to have Rust and Cargo installed on your system. If you don't have Rust installed, follow the instructions on [Rust's official website](https://www.rust-lang.org/tools/install).

```bash
git clone https://github.com/rnabioco/deepsort-rs

cd deepsort-rs

cargo install --path .
```

## Usage

To use deepsort, navigate to the cloned directory and run the following command:

```bash
deepsort <barcode_file> <fastq_folder> [output_folder]
```

- `<barcode_file>`: The path to the barcode demux file output from deeplexicon.
- `<fastq_folder>`: The directory containing gzipped FASTQ files.
- `[output_folder]`: (Optional) The directory where the output files will be saved. If not specified, files will be saved in the current directory.