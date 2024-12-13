use anyhow::{Context, Result};
use calamine::{open_workbook, Reader, Xlsx};
use clap::Parser;
use polars::prelude::*;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about = "Convert Excel files to Parquet format")]
struct Cli {
    /// Path to the input Excel file
    #[arg(value_name = "EXCEL_FILE")]
    excel_file: PathBuf,

    /// Skip the first n rows
    #[arg(short, long, default_value = "0")]
    skip_rows: usize,

    /// Output path, default current directory
    #[arg(short, long, default_value = ".")]
    output_path: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Verify input file exists and has xlsx extension
    if !cli.excel_file.exists() {
        anyhow::bail!("Input file does not exist");
    }
    
    if cli.excel_file.extension().and_then(|ext| ext.to_str()) != Some("xlsx") {
        anyhow::bail!("Input file must be an .xlsx file");
    }

    // Create output path replace .xlsx with .parquet
    let output_path: PathBuf = cli.output_path.join(
        std::path::Path::new(&cli.excel_file.file_name().unwrap().to_str().unwrap().replace(".xlsx", ".parquet"))
    );

    // Read Excel file
    let mut workbook: Xlsx<_> = open_workbook(&cli.excel_file)
        .with_context(|| format!("Failed to open Excel file: {:?}", cli.excel_file))?;

    // Get the first worksheet
    let sheet_name = workbook
        .sheet_names()
        .first()
        .context("No worksheets found")?
        .clone();

    let range = workbook
        .worksheet_range(&sheet_name)
        .context("Failed to get worksheet")?;

    // Convert to vectors for Polars DataFrame
    let mut headers: Vec<String> = range
        .rows()
        .skip(cli.skip_rows)
        .next()
        .context("No headers found")?
        .iter()
        .map(|cell| cell.to_string())
        .collect();

    // Test headers on empty strings, if empty set to "Field_index"
    // First pass: collect indices of empty headers
    let empty_indices: Vec<_> = headers.iter()
        .enumerate()
        .filter(|(_, h)| h.is_empty())
        .map(|(i, _)| i)
        .collect();

    // Fix empty headers
    for i in empty_indices {
        headers[i] = format!("Field_{}", i);
    }

    // Second pass: collect indices and values of duplicate headers
    let mut seen = std::collections::HashMap::new();
    let mut duplicates = Vec::new();
    
    for (i, header) in headers.iter().enumerate() {
        if seen.insert(header.clone(), i).is_some() {
            duplicates.push((i, header.clone()));
        }
    }

    // Fix duplicate headers
    for (i, header) in duplicates {
        if let Some(_) = seen.insert(header.clone(), i) {
            headers[i] = format!("{}_{}", header, i);
        }
    }

    let data: Vec<Vec<String>> = range
        .rows()
        .skip(cli.skip_rows + 1)
        .map(|row| row.iter().map(|cell| cell.to_string()).collect())
        .collect();

    // Create series for each column
    let mut columns = Vec::new();
    for (i, header) in headers.iter().enumerate() {
        let values: Vec<String> = data.iter().map(|row| row[i].clone()).collect();
        let series = Series::new(header.into(), values);
        columns.push(series.into());
    }

    // Create DataFrame and write to Parquet
    let mut df = DataFrame::new(columns)
    .with_context(|| "Failed to create DataFrame")?;

    ParquetWriter::new(std::fs::File::create(output_path.clone())?)
        .finish(&mut df)
        .with_context(|| format!("Failed to write Parquet file: {:?}", output_path))?;

    println!("Successfully converted {:?} to {:?}", cli.excel_file, output_path);
    Ok(())
}
