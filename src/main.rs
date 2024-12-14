use anyhow::{Context, Result};
use arrow::array::{ArrayRef, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use calamine::{open_workbook, Reader, Xlsx};
use clap::Parser;
use parquet::arrow::arrow_writer::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::properties::WriterProperties;
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;

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
        std::path::Path::new(
            &cli.excel_file
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .replace(".xlsx", ".parquet"),
        ),
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

    // Get headers and handle empty/duplicate names
    let mut headers: Vec<String> = range
        .rows()
        .skip(cli.skip_rows)
        .next()
        .context("No headers found")?
        .iter()
        .map(|cell| cell.to_string())
        .collect();

    // Handle empty headers
    for (i, header) in headers.iter_mut().enumerate() {
        if header.is_empty() {
            *header = format!("Field_{}", i);
        }
    }

    // Handle duplicate headers
    let mut seen = HashMap::new();
    for i in 0..headers.len() {
        let header = &headers[i];
        let count = seen.entry(header.clone()).or_insert(0);
        *count += 1;
        if *count > 1 {
            headers[i] = format!("{}_{}", header, count);
        }
    }

    // Create Arrow schema
    let schema = Arc::new(Schema::new(
        headers
            .iter()
            .map(|name| Field::new(name, DataType::Utf8, true))
            .collect::<Vec<Field>>()
    ));

    // Convert data to columns
    let data_rows: Vec<_> = range.rows().skip(cli.skip_rows + 1).collect();
    let num_rows = data_rows.len();
    let mut columns: Vec<ArrayRef> = Vec::new();

    // Create string arrays for each column
    for col_idx in 0..headers.len() {
        let values: Vec<Option<String>> = data_rows
            .iter()
            .map(|row| {
                row.get(col_idx)
                    .map(|cell| cell.to_string())
            })
            .collect();
        
        let string_array = StringArray::from(values);
        columns.push(Arc::new(string_array));
    }

    // Create RecordBatch
    let record_batch =
        RecordBatch::try_new(schema.clone(), columns).context("Failed to create record batch")?;

    // Set up Parquet writer properties
    let props = WriterProperties::builder()
        .set_compression(Compression::SNAPPY)
        .build();

    // Create Parquet file and writer
    let file = File::create(&output_path).context("Failed to create output file")?;
    let mut writer = ArrowWriter::try_new(file, schema, Some(props))
        .context("Failed to create parquet writer")?;

    // Write the record batch
    writer
        .write(&record_batch)
        .context("Failed to write record batch")?;

    // Close the writer
    writer.close().context("Failed to close writer")?;

    println!(
        "Successfully converted {:?} to {:?} ({} rows)",
        cli.excel_file, output_path, num_rows
    );
    Ok(())
}
