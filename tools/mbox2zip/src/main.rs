use anyhow::Result;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use mbox2zip::convert_mbox_to_mbxc;
use std::fs::File;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(help = "Path to the MBOX file")]
    input: String,

    #[arg(short, long, help = "Output MBXC file path")]
    output: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let input_path = PathBuf::from(&args.input);
    let output_path = match args.output {
        Some(o) => PathBuf::from(o),
        None => input_path.with_extension("mbxc"),
    };

    println!(
        "Converting {} to {}...",
        input_path.display(),
        output_path.display()
    );

    let input_file = File::open(&input_path)?;
    let file_size = input_file.metadata()?.len();
    drop(input_file);

    let pb = ProgressBar::new(file_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
        .progress_chars("#>-"));

    let pb_clone = pb.clone();
    let progress_callback = Some(Box::new(move |total_bytes, _count| {
        pb_clone.set_position(total_bytes);
    }) as Box<dyn Fn(u64, u64) + Send>);

    convert_mbox_to_mbxc(input_path, output_path, progress_callback)?;

    pb.finish_with_message("Processing complete");
    println!("\nSuccessfully processed messages.");

    Ok(())
}
