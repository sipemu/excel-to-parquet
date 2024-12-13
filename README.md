# Excel to Parquet Converter

A command-line tool written in Rust that converts Excel (XLSX) files to Parquet format. This tool is designed to be simple and efficient, making it easy to convert Excel data for use with big data tools.

## Features

- Convert XLSX files to Parquet format
- Handle empty column names (auto-generates names like Field_0, Field_1, etc.)
- Skip rows option for files with headers not in the first row
- Specify custom output directory
- Simple command-line interface
- Currently, no type inference, all data is stored as strings
- First sheet is used. Currently, no support for selecting a specific sheet.

## Installation

### From Source

Requires Rust toolchain to be installed. Visit [rustup.rs](https://rustup.rs/) to install Rust.

```bash
# Clone the repository
git clone https://github.com/yourusername/excel-to-parquet
cd excel-to-parquet

# Build and install
cargo install --path .
```

## Usage

Basic usage:
```bash
excel-to-parquet input.xlsx
```

Skip first N rows:
```bash
excel-to-parquet -s 2 input.xlsx
```

Specify output directory:
```bash
excel-to-parquet -o /path/to/output input.xlsx
```

### Command Line Options

```
USAGE:
    excel-to-parquet [OPTIONS] <EXCEL_FILE>

ARGS:
    <EXCEL_FILE>    Path to the input Excel file

OPTIONS:
    -h, --help                   Print help information
    -s, --skip-rows <N>         Number of rows to skip [default: 0]
    -o, --output-path <PATH>    Output directory [default: .]
    -V, --version               Print version information
```

## Output

The output Parquet file will:
- Have the same name as the input file (with .parquet extension)
- Be saved in the specified output directory (or current directory if not specified)
- Preserve data as strings from the Excel file

## Building from Source

```bash
# Debug build
cargo build

# Release build
cargo build --release
```

## Bash Script (Just for Linux)

A bash script for bulk converting Excel files to Parquet format is added to the repository. The executable must be in the same directory as the script.

```bash
# Convert all Excel files, skip 2 rows
./convert_excel_to_parquet.sh -s ./excel_files -t ./parquet_files -r 2

# Convert all Excel files, no rows skipped
./convert_excel_to_parquet.sh -s ./excel_files -t ./parquet_files

# Show help
./convert_excel_to_parquet.sh -h
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.