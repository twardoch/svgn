// this_file: svgn/src/bin/svgn.rs

//! SVGN command-line interface
//!
//! This is the CLI binary for SVGN, providing SVGO-compatible command-line
//! options for SVG optimization.

use clap::{Arg, ArgAction, Command};
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use svgn::{optimize_with_config, Config, VERSION};

fn main() {
    let matches = Command::new("svgn")
        .version(VERSION)
        .about("A high-performance Rust port of SVGO (SVG Optimizer)")
        .arg(
            Arg::new("input")
                .help("Input file or directory")
                .short('i')
                .long("input")
                .value_name("FILE/DIR")
        )
        .arg(
            Arg::new("output")
                .help("Output file or directory")
                .short('o')
                .long("output")
                .value_name("FILE/DIR")
        )
        .arg(
            Arg::new("folder")
                .help("Input folder, optimize and rewrite all *.svg files")
                .short('f')
                .long("folder")
                .value_name("DIR")
        )
        .arg(
            Arg::new("pretty")
                .help("Make SVG pretty printed")
                .short('p')
                .long("pretty")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("config")
                .help("Custom config file")
                .long("config")
                .value_name("FILE")
        )
        .arg(
            Arg::new("disable")
                .help("Disable a plugin by name")
                .long("disable")
                .value_name("PLUGIN")
                .action(ArgAction::Append)
        )
        .arg(
            Arg::new("enable")
                .help("Enable a plugin by name")
                .long("enable")
                .value_name("PLUGIN")
                .action(ArgAction::Append)
        )
        .arg(
            Arg::new("datauri")
                .help("Output as Data URI string (base64, enc, unenc)")
                .long("datauri")
                .value_name("FORMAT")
        )
        .arg(
            Arg::new("multipass")
                .help("Optimize SVG multiple times")
                .long("multipass")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("quiet")
                .help("Only output error messages")
                .short('q')
                .long("quiet")
                .action(ArgAction::SetTrue)
        )
        .get_matches();

    let result = run_cli(matches);
    
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run_cli(matches: clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let quiet = matches.get_flag("quiet");

    // Load configuration
    let mut config = if let Some(config_path) = matches.get_one::<String>("config") {
        Config::from_file(config_path)?
    } else {
        // Try to load from current directory
        svgn::config::load_config_from_directory(".")?.unwrap_or_else(|| Config::with_default_preset())
    };

    // Apply CLI overrides
    if matches.get_flag("pretty") {
        config.js2svg.pretty = true;
    }

    if matches.get_flag("multipass") {
        config.multipass = true;
    }

    if let Some(datauri_format) = matches.get_one::<String>("datauri") {
        use svgn::config::DataUriFormat;
        config.datauri = Some(match datauri_format.as_str() {
            "base64" => DataUriFormat::Base64,
            "enc" => DataUriFormat::Enc,
            "unenc" => DataUriFormat::Unenc,
            _ => return Err(format!("Unknown datauri format: {}", datauri_format).into()),
        });
    }

    // Handle plugin enable/disable
    if let Some(disabled_plugins) = matches.get_many::<String>("disable") {
        for plugin_name in disabled_plugins {
            config.set_plugin_enabled(plugin_name, false);
        }
    }

    if let Some(enabled_plugins) = matches.get_many::<String>("enable") {
        for plugin_name in enabled_plugins {
            // Add plugin if it doesn't exist
            if config.get_plugin(plugin_name).is_none() {
                config.add_plugin(svgn::plugin::PluginConfig::new(plugin_name.clone()));
            } else {
                config.set_plugin_enabled(plugin_name, true);
            }
        }
    }

    // Handle folder processing
    if let Some(folder_path) = matches.get_one::<String>("folder") {
        return process_folder(folder_path, &config, quiet);
    }

    // Handle single file or stdin/stdout
    let input_path = matches.get_one::<String>("input");
    let output_path = matches.get_one::<String>("output");

    let input_content = if let Some(input_file) = input_path {
        fs::read_to_string(input_file)?
    } else {
        // Read from stdin
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    };

    // Set config path for context
    if let Some(path) = input_path {
        config.path = Some(path.clone());
    }

    // Optimize the SVG
    let result = optimize_with_config(&input_content, config)?;

    // Write output
    if let Some(output_file) = output_path {
        fs::write(output_file, &result.data)?;
        if !quiet {
            eprintln!("Optimized: {} → {} ({:.1}% reduction)", 
                     format_bytes(result.info.original_size),
                     format_bytes(result.info.optimized_size),
                     result.info.compression_percentage());
        }
    } else {
        // Write to stdout
        print!("{}", result.data);
    }

    Ok(())
}

fn process_folder(folder_path: &str, config: &Config, quiet: bool) -> Result<(), Box<dyn std::error::Error>> {
    let folder = PathBuf::from(folder_path);
    
    if !folder.is_dir() {
        return Err(format!("{} is not a directory", folder_path).into());
    }

    let svg_files = find_svg_files(&folder)?;
    
    if svg_files.is_empty() {
        if !quiet {
            eprintln!("No SVG files found in {}", folder_path);
        }
        return Ok(());
    }

    let mut total_original = 0;
    let mut total_optimized = 0;
    let mut processed_count = 0;

    for svg_file in svg_files {
        let input_content = fs::read_to_string(&svg_file)?;
        
        let mut file_config = config.clone();
        file_config.path = Some(svg_file.to_string_lossy().to_string());

        match optimize_with_config(&input_content, file_config) {
            Ok(result) => {
                fs::write(&svg_file, &result.data)?;
                
                total_original += result.info.original_size;
                total_optimized += result.info.optimized_size;
                processed_count += 1;

                if !quiet {
                    println!("Optimized: {} ({:.1}% reduction)",
                            svg_file.display(),
                            result.info.compression_percentage());
                }
            }
            Err(e) => {
                eprintln!("Error processing {}: {}", svg_file.display(), e);
            }
        }
    }

    if !quiet && processed_count > 0 {
        let total_reduction = if total_original > 0 {
            ((total_original - total_optimized) as f64 / total_original as f64) * 100.0
        } else {
            0.0
        };

        eprintln!("\nTotal: {} files processed", processed_count);
        eprintln!("Size: {} → {} ({:.1}% reduction)",
                 format_bytes(total_original),
                 format_bytes(total_optimized),
                 total_reduction);
    }

    Ok(())
}

fn find_svg_files(dir: &PathBuf) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut svg_files = Vec::new();
    
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension.to_string_lossy().to_lowercase() == "svg" {
                    svg_files.push(path);
                }
            }
        }
    }
    
    svg_files.sort();
    Ok(svg_files)
}

fn format_bytes(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    
    if bytes == 0 {
        return "0 B".to_string();
    }
    
    let bytes_f = bytes as f64;
    let i = (bytes_f.log10() / 3.0).floor() as usize;
    let i = i.min(UNITS.len() - 1);
    
    let size = bytes_f / (1000.0_f64.powi(i as i32));
    
    if i == 0 {
        format!("{} {}", bytes, UNITS[i])
    } else {
        format!("{:.1} {}", size, UNITS[i])
    }
}