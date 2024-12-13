#!/bin/bash

# Function to print usage
print_usage() {
    echo "Usage: $0 -s <source_dir> -t <target_dir> [-r <skip_rows>]"
    echo
    echo "Options:"
    echo "  -s <source_dir>   Directory containing Excel files"
    echo "  -t <target_dir>   Directory for output Parquet files"
    echo "  -r <skip_rows>    Number of rows to skip (default: 0)"
    echo "  -h                Display this help message"
    exit 1
}

# Initialize variables
source_dir=""
target_dir=""
skip_rows=0

# Parse command line arguments
while getopts "s:t:r:h" opt; do
    case $opt in
        s) source_dir="$OPTARG";;
        t) target_dir="$OPTARG";;
        r) skip_rows="$OPTARG";;
        h) print_usage;;
        \?) print_usage;;
    esac
done

# Validate required arguments
if [ -z "$source_dir" ] || [ -z "$target_dir" ]; then
    echo "Error: Source and target directories are required"
    print_usage
fi

# Check if source directory exists
if [ ! -d "$source_dir" ]; then
    echo "Error: Source directory does not exist: $source_dir"
    exit 1
fi

# Create target directory if it doesn't exist
mkdir -p "$target_dir"

# Check if excel-to-parquet is installed
if ! command -v ./excel-to-parquet &> /dev/null; then
    echo "Error: excel-to-parquet is not installed"
    exit 1
fi

# Initialize counters
total_files=0
converted_files=0
failed_files=0

# Process all Excel files
echo "Starting conversion process..."
echo "Source directory: $source_dir"
echo "Target directory: $target_dir"
echo "Skip rows: $skip_rows"
echo

for excel_file in "$source_dir"/*.xlsx; do
    # Check if there are any xlsx files
    if [ ! -e "$excel_file" ]; then
        echo "No Excel files found in $source_dir"
        exit 0
    fi

    ((total_files++))
    filename=$(basename "$excel_file")
    echo "Processing: $filename"

    # Convert the file
    if ./excel-to-parquet -s "$skip_rows" -o "$target_dir" "$excel_file"; then
        ((converted_files++))
        echo "Successfully converted: $filename"
    else
        ((failed_files++))
        echo "Failed to convert: $filename"
    fi
    echo
done

# Print summary
echo "Conversion complete!"
echo "Total files processed: $total_files"
echo "Successfully converted: $converted_files"
echo "Failed conversions: $failed_files"

# Exit with error if any conversions failed
if [ $failed_files -gt 0 ]; then
    exit 1
fi

exit 0 