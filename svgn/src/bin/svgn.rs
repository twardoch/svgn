// this_file: svgn/src/bin/svgn.rs

//! SVGN command-line interface
//!
//! This is the CLI binary for SVGN, providing SVGO-compatible command-line
//! options for SVG optimization.

use clap::{Arg, ArgAction, Command};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use svgn::{optimize_with_config, Config, VERSION};

fn main() {
    let matches = Command::new("svgn")
        .version(VERSION)
        .about("A high-performance Rust port of SVGO (SVG Optimizer)")
        // Allow positional arguments for input files
        .arg(
            Arg::new("INPUT")
                .help("Input files, \"-\" for STDIN")
                .value_name("INPUT")
                .num_args(1..)
                .conflicts_with_all(&["input", "string", "folder"]),
        )
        .arg(
            Arg::new("input")
                .help("Input files, \"-\" for STDIN")
                .short('i')
                .long("input")
                .value_name("INPUT")
                .num_args(1..)
                .conflicts_with_all(&["INPUT", "string", "folder"]),
        )
        .arg(
            Arg::new("string")
                .help("Input SVG data string")
                .short('s')
                .long("string")
                .value_name("STRING")
                .conflicts_with_all(&["INPUT", "input", "folder"]),
        )
        .arg(
            Arg::new("folder")
                .help("Input folder, optimize and rewrite all *.svg files")
                .short('f')
                .long("folder")
                .value_name("FOLDER")
                .conflicts_with_all(&["INPUT", "input", "string"]),
        )
        .arg(
            Arg::new("output")
                .help("Output file or folder (by default the same as the input), \"-\" for STDOUT")
                .short('o')
                .long("output")
                .value_name("OUTPUT")
                .num_args(1..),
        )
        .arg(
            Arg::new("precision")
                .help("Set number of digits in the fractional part, overrides plugins params")
                .short('p')
                .long("precision")
                .value_name("INTEGER")
                .value_parser(clap::value_parser!(u8)),
        )
        .arg(
            Arg::new("pretty")
                .help("Make SVG pretty printed")
                .long("pretty")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("indent")
                .help("Indent number when pretty printing SVGs")
                .long("indent")
                .value_name("INTEGER")
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(
            Arg::new("eol")
                .help("Line break to use when outputting SVG: lf, crlf. If unspecified, uses platform default.")
                .long("eol")
                .value_name("EOL")
                .value_parser(["lf", "crlf"]),
        )
        .arg(
            Arg::new("final-newline")
                .help("Ensure SVG ends with a line break")
                .long("final-newline")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("config")
                .help("Custom config file, only .js is supported")
                .long("config")
                .value_name("CONFIG"),
        )
        .arg(
            Arg::new("disable")
                .help("Disable a plugin by name")
                .long("disable")
                .value_name("PLUGIN")
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("enable")
                .help("Enable a plugin by name")
                .long("enable")
                .value_name("PLUGIN")
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("datauri")
                .help("Output as Data URI string (base64), URI encoded (enc) or unencoded (unenc)")
                .long("datauri")
                .value_name("FORMAT")
                .value_parser(["base64", "enc", "unenc"]),
        )
        .arg(
            Arg::new("multipass")
                .help("Pass over SVGs multiple times to ensure all optimizations are applied")
                .long("multipass")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("recursive")
                .help("Use with '--folder'. Optimizes *.svg files in folders recursively.")
                .short('r')
                .long("recursive")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("exclude")
                .help("Use with '--folder'. Exclude files matching regular expression pattern.")
                .long("exclude")
                .value_name("PATTERN")
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("quiet")
                .help("Only output error messages, not regular status messages")
                .short('q')
                .long("quiet")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("show-plugins")
                .help("Show available plugins and exit")
                .long("show-plugins")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no-color")
                .help("Output plain text without color")
                .long("no-color")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    // Check if we should show plugins and exit
    if matches.get_flag("show-plugins") {
        show_plugins();
        std::process::exit(0);
    }

    let result = run_cli(matches);

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run_cli(matches: clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let quiet = matches.get_flag("quiet");
    let no_color = matches.get_flag("no-color") || std::env::var("NO_COLOR").is_ok();

    // Load configuration
    let mut config = if let Some(config_path) = matches.get_one::<String>("config") {
        Config::from_file(config_path)?
    } else {
        // Try to load from current directory
        svgn::config::load_config_from_directory(".")?.unwrap_or_else(Config::with_default_preset)
    };

    // Apply CLI overrides
    if matches.get_flag("pretty") {
        config.js2svg.pretty = true;
    }

    if let Some(indent) = matches.get_one::<usize>("indent") {
        config.js2svg.indent = *indent;
    }
    
    if let Some(eol) = matches.get_one::<String>("eol") {
        use svgn::config::LineEnding;
        config.js2svg.eol = match eol.as_str() {
            "lf" => LineEnding::Lf,
            "crlf" => LineEnding::Crlf,
            _ => unreachable!(), // Clap validates this
        };
    }
    
    if matches.get_flag("final-newline") {
        config.js2svg.final_newline = true;
    }

    if matches.get_flag("multipass") {
        config.multipass = true;
    }

    // Apply precision override
    if let Some(precision) = matches.get_one::<u8>("precision") {
        apply_precision_override(&mut config, *precision);
    }

    if let Some(datauri_format) = matches.get_one::<String>("datauri") {
        use svgn::config::DataUriFormat;
        config.datauri = Some(match datauri_format.as_str() {
            "base64" => DataUriFormat::Base64,
            "enc" => DataUriFormat::Enc,
            "unenc" => DataUriFormat::Unenc,
            _ => unreachable!(), // Clap validates this
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

    // Determine input mode
    let (input_mode, output_mode) = determine_io_mode(&matches)?;

    match input_mode {
        InputMode::String(svg_string) => {
            process_string(&svg_string, output_mode, &config, quiet)?;
        }
        InputMode::Stdin => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            process_string(&buffer, output_mode, &config, quiet)?;
        }
        InputMode::Files(files) => {
            process_files(&files, output_mode, &config, quiet, no_color)?;
        }
        InputMode::Folder(folder, recursive) => {
            let exclude_patterns = matches
                .get_many::<String>("exclude")
                .map(|v| v.map(|s| s.as_str()).collect::<Vec<_>>())
                .unwrap_or_default();
            process_folder(&folder, &config, quiet, recursive, &exclude_patterns)?;
        }
    }

    Ok(())
}

enum InputMode {
    String(String),
    Stdin,
    Files(Vec<String>),
    Folder(String, bool), // path, recursive
}

enum OutputMode {
    Stdout,
    File(String),
    Directory(String),
    InPlace,
}

fn determine_io_mode(
    matches: &clap::ArgMatches,
) -> Result<(InputMode, OutputMode), Box<dyn std::error::Error>> {
    // Determine input mode
    let input_mode = if let Some(svg_string) = matches.get_one::<String>("string") {
        InputMode::String(svg_string.clone())
    } else if let Some(folder) = matches.get_one::<String>("folder") {
        let recursive = matches.get_flag("recursive");
        InputMode::Folder(folder.clone(), recursive)
    } else if let Some(input_files) = matches.get_many::<String>("input") {
        let files: Vec<String> = input_files.cloned().collect();
        if files.len() == 1 && files[0] == "-" {
            InputMode::Stdin
        } else {
            InputMode::Files(files)
        }
    } else if let Some(positional_files) = matches.get_many::<String>("INPUT") {
        let files: Vec<String> = positional_files.cloned().collect();
        if files.len() == 1 && files[0] == "-" {
            InputMode::Stdin
        } else {
            InputMode::Files(files)
        }
    } else {
        // No input specified, default to stdin
        InputMode::Stdin
    };

    // Determine output mode
    let output_mode = match matches.get_many::<String>("output") {
        Some(outputs) => {
            let outputs: Vec<String> = outputs.cloned().collect();
            if outputs.len() == 1 {
                if outputs[0] == "-" {
                    OutputMode::Stdout
                } else {
                    let path = Path::new(&outputs[0]);
                    if path.is_dir() || outputs[0].ends_with('/') || outputs[0].ends_with('\\') {
                        OutputMode::Directory(outputs[0].clone())
                    } else {
                        OutputMode::File(outputs[0].clone())
                    }
                }
            } else {
                return Err("Multiple output files not supported".into());
            }
        }
        None => {
            // Default behavior
            match &input_mode {
                InputMode::Stdin | InputMode::String(_) => OutputMode::Stdout,
                _ => OutputMode::InPlace,
            }
        }
    };

    Ok((input_mode, output_mode))
}

fn process_string(
    content: &str,
    output_mode: OutputMode,
    config: &Config,
    quiet: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let result = optimize_with_config(content, config.clone())?;

    match output_mode {
        OutputMode::Stdout => {
            print!("{}", result.data);
            io::stdout().flush()?;
        }
        OutputMode::File(path) => {
            fs::write(&path, &result.data)?;
            if !quiet {
                eprintln!(
                    "Optimized: {} → {} ({:.1}% reduction)",
                    format_bytes(result.info.original_size),
                    format_bytes(result.info.optimized_size),
                    result.info.compression_percentage()
                );
            }
        }
        _ => return Err("Invalid output mode for string input".into()),
    }

    Ok(())
}

fn process_files(
    files: &[String],
    output_mode: OutputMode,
    config: &Config,
    quiet: bool,
    _no_color: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match output_mode {
        OutputMode::Stdout => {
            if files.len() > 1 {
                return Err("Cannot output multiple files to stdout".into());
            }
            let content = fs::read_to_string(&files[0])?;
            let mut file_config = config.clone();
            file_config.path = Some(files[0].clone());
            let result = optimize_with_config(&content, file_config)?;
            print!("{}", result.data);
            io::stdout().flush()?;
        }
        OutputMode::File(output_path) => {
            if files.len() > 1 {
                return Err("Cannot output multiple files to a single file".into());
            }
            let content = fs::read_to_string(&files[0])?;
            let mut file_config = config.clone();
            file_config.path = Some(files[0].clone());
            let result = optimize_with_config(&content, file_config)?;
            fs::write(&output_path, &result.data)?;
            if !quiet {
                eprintln!(
                    "Optimized: {} → {} ({:.1}% reduction)",
                    format_bytes(result.info.original_size),
                    format_bytes(result.info.optimized_size),
                    result.info.compression_percentage()
                );
            }
        }
        OutputMode::Directory(output_dir) => {
            for input_file in files {
                let content = fs::read_to_string(input_file)?;
                let mut file_config = config.clone();
                file_config.path = Some(input_file.clone());
                let result = optimize_with_config(&content, file_config)?;

                let input_path = Path::new(input_file);
                let file_name = input_path
                    .file_name()
                    .ok_or("Invalid input file path")?;
                let output_path = Path::new(&output_dir).join(file_name);

                fs::write(&output_path, &result.data)?;
                if !quiet {
                    eprintln!(
                        "Optimized: {} → {} ({:.1}% reduction)",
                        input_path.display(),
                        output_path.display(),
                        result.info.compression_percentage()
                    );
                }
            }
        }
        OutputMode::InPlace => {
            for input_file in files {
                let content = fs::read_to_string(input_file)?;
                let mut file_config = config.clone();
                file_config.path = Some(input_file.clone());
                let result = optimize_with_config(&content, file_config)?;
                fs::write(input_file, &result.data)?;
                if !quiet {
                    eprintln!(
                        "Optimized: {} ({:.1}% reduction)",
                        input_file,
                        result.info.compression_percentage()
                    );
                }
            }
        }
    }

    Ok(())
}

fn process_folder(
    folder_path: &str,
    config: &Config,
    quiet: bool,
    recursive: bool,
    exclude_patterns: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    let folder = PathBuf::from(folder_path);

    if !folder.is_dir() {
        return Err(format!("{} is not a directory", folder_path).into());
    }

    let svg_files = if recursive {
        find_svg_files_recursive(&folder, exclude_patterns)?
    } else {
        find_svg_files(&folder, exclude_patterns)?
    };

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
                    println!(
                        "Optimized: {} ({:.1}% reduction)",
                        svg_file.display(),
                        result.info.compression_percentage()
                    );
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
        eprintln!(
            "Size: {} → {} ({:.1}% reduction)",
            format_bytes(total_original),
            format_bytes(total_optimized),
            total_reduction
        );
    }

    Ok(())
}

fn find_svg_files(
    dir: &Path,
    exclude_patterns: &[&str],
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut svg_files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && is_svg_file(&path) && !is_excluded(&path, exclude_patterns)? {
            svg_files.push(path);
        }
    }

    svg_files.sort();
    Ok(svg_files)
}

fn find_svg_files_recursive(
    dir: &Path,
    exclude_patterns: &[&str],
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut svg_files = Vec::new();
    let mut dirs_to_process = vec![dir.to_path_buf()];

    while let Some(current_dir) = dirs_to_process.pop() {
        for entry in fs::read_dir(&current_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                dirs_to_process.push(path);
            } else if path.is_file() && is_svg_file(&path) && !is_excluded(&path, exclude_patterns)? {
                svg_files.push(path);
            }
        }
    }

    svg_files.sort();
    Ok(svg_files)
}

fn is_svg_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("svg"))
        .unwrap_or(false)
}

fn is_excluded(path: &Path, patterns: &[&str]) -> Result<bool, Box<dyn std::error::Error>> {
    if patterns.is_empty() {
        return Ok(false);
    }

    let path_str = path.to_string_lossy();
    for pattern in patterns {
        let re = regex::Regex::new(pattern)?;
        if re.is_match(&path_str) {
            return Ok(true);
        }
    }

    Ok(false)
}

fn apply_precision_override(config: &mut Config, precision: u8) {
    // Apply precision to all plugins that support it
    for plugin_config in &mut config.plugins {
        match plugin_config.name.as_str() {
            "cleanupNumericValues" | "cleanupListOfValues" | "convertPathData" | "convertTransform" => {
                let params = plugin_config
                    .params
                    .get_or_insert_with(|| serde_json::json!({}));
                if let Some(obj) = params.as_object_mut() {
                    obj.insert("floatPrecision".to_string(), serde_json::json!(precision));
                }
            }
            _ => {}
        }
    }
}

fn show_plugins() {
    println!("Available plugins:");
    println!();

    let registry = svgn::plugin::create_default_registry();
    let mut plugin_names = registry.plugin_names();
    plugin_names.sort();

    for name in plugin_names {
        if let Some(plugin) = registry.get(name) {
            println!("  {} - {}", name, plugin.description());
        }
    }
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