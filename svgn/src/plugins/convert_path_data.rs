// this_file: svgn/src/plugins/convert_path_data.rs

//! Convert path data to relative or absolute, optimize segments, simplify curves
//!
//! This plugin optimizes path data by:
//! - Converting between absolute and relative commands
//! - Removing redundant commands
//! - Optimizing number precision
//! - Simplifying curves where possible
//! - Collapsing repeated commands

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use serde_json::Value;

/// Default decimal precision for path coordinates
const DEFAULT_FLOAT_PRECISION: u8 = 3;

/// Default decimal precision for transform values
const DEFAULT_TRANSFORM_PRECISION: u8 = 5;

/// Configuration for path optimization
struct PathOptimizationConfig {
    float_precision: u8,
    #[allow(dead_code)]
    transform_precision: u8,
    remove_useless: bool,
    collapse_repeated: bool,
    utilize_absolute: bool,
    leading_zero: bool,
    negative_extra_space: bool,
}

/// Plugin for optimizing path data
pub struct ConvertPathDataPlugin;

impl Plugin for ConvertPathDataPlugin {
    fn name(&self) -> &'static str {
        "convertPathData"
    }

    fn description(&self) -> &'static str {
        "Converts path data to relative or absolute, optimizes segments, simplifies curves"
    }

    fn apply(
        &mut self,
        document: &mut Document,
        _plugin_info: &PluginInfo,
        params: Option<&Value>,
    ) -> PluginResult<()> {
        // Parse parameters
        let float_precision = params
            .and_then(|v| v.get("floatPrecision"))
            .and_then(|v| v.as_u64())
            .map(|p| p as u8)
            .unwrap_or(DEFAULT_FLOAT_PRECISION);

        let transform_precision = params
            .and_then(|v| v.get("transformPrecision"))
            .and_then(|v| v.as_u64())
            .map(|p| p as u8)
            .unwrap_or(DEFAULT_TRANSFORM_PRECISION);

        let remove_useless = params
            .and_then(|v| v.get("removeUseless"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let collapse_repeated = params
            .and_then(|v| v.get("collapseRepeated"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let utilize_absolute = params
            .and_then(|v| v.get("utilizeAbsolute"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let leading_zero = params
            .and_then(|v| v.get("leadingZero"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let negative_extra_space = params
            .and_then(|v| v.get("negativeExtraSpace"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        // Create config
        let config = PathOptimizationConfig {
            float_precision,
            transform_precision,
            remove_useless,
            collapse_repeated,
            utilize_absolute,
            leading_zero,
            negative_extra_space,
        };

        // Process the document
        optimize_paths_in_element(&mut document.root, &config)?;

        Ok(())
    }
}

/// Recursively optimize paths in an element and its children
fn optimize_paths_in_element(
    element: &mut Element,
    config: &PathOptimizationConfig,
) -> PluginResult<()> {
    // Process this element if it's a path
    if element.name == "path" {
        if let Some(d) = element.attr("d") {
            match optimize_path_data(d, config) {
                Ok(optimized) => {
                    element.set_attr("d".to_string(), optimized);
                }
                Err(e) => {
                    // Log error but continue processing other paths
                    eprintln!("Warning: Failed to optimize path data: {}", e);
                }
            }
        }
    }

    // Process children
    for child in &mut element.children {
        if let Node::Element(child_elem) = child {
            optimize_paths_in_element(child_elem, config)?;
        }
    }

    Ok(())
}

/// Path command types
#[derive(Debug, Clone, Copy, PartialEq)]
enum CommandType {
    MoveTo,
    LineTo,
    HorizontalLineTo,
    VerticalLineTo,
    CurveTo,
    SmoothCurveTo,
    QuadraticBezier,
    SmoothQuadraticBezier,
    Arc,
    ClosePath,
}

/// Represents a path command with its parameters
#[derive(Debug, Clone)]
struct PathCommand {
    cmd_type: CommandType,
    is_absolute: bool,
    params: Vec<f64>,
}

impl PathCommand {
    /// Get the command character
    fn get_char(&self) -> char {
        match (self.cmd_type, self.is_absolute) {
            (CommandType::MoveTo, true) => 'M',
            (CommandType::MoveTo, false) => 'm',
            (CommandType::LineTo, true) => 'L',
            (CommandType::LineTo, false) => 'l',
            (CommandType::HorizontalLineTo, true) => 'H',
            (CommandType::HorizontalLineTo, false) => 'h',
            (CommandType::VerticalLineTo, true) => 'V',
            (CommandType::VerticalLineTo, false) => 'v',
            (CommandType::CurveTo, true) => 'C',
            (CommandType::CurveTo, false) => 'c',
            (CommandType::SmoothCurveTo, true) => 'S',
            (CommandType::SmoothCurveTo, false) => 's',
            (CommandType::QuadraticBezier, true) => 'Q',
            (CommandType::QuadraticBezier, false) => 'q',
            (CommandType::SmoothQuadraticBezier, true) => 'T',
            (CommandType::SmoothQuadraticBezier, false) => 't',
            (CommandType::Arc, true) => 'A',
            (CommandType::Arc, false) => 'a',
            (CommandType::ClosePath, _) => 'z',
        }
    }
}

/// Parse a path data string into commands
fn parse_path_data(path_data: &str) -> Result<Vec<PathCommand>, String> {
    let mut commands = Vec::new();
    let mut chars = path_data.chars().peekable();
    let mut current_nums = Vec::new();
    let mut current_num = String::new();
    let mut last_cmd_type = None;
    let mut in_number = false;

    for ch in chars.by_ref() {
        match ch {
            'M' | 'm' | 'L' | 'l' | 'H' | 'h' | 'V' | 'v' | 'C' | 'c' | 'S' | 's' | 'Q' | 'q'
            | 'T' | 't' | 'A' | 'a' | 'Z' | 'z' => {
                // Finish current number if any
                if !current_num.is_empty() {
                    if let Ok(num) = current_num.parse::<f64>() {
                        current_nums.push(num);
                    }
                    current_num.clear();
                    in_number = false;
                }

                // Process accumulated numbers with previous command
                if let Some(cmd_type) = last_cmd_type {
                    process_accumulated_params(&mut commands, cmd_type, &mut current_nums)?;
                }

                // Parse new command
                let (cmd_type, is_absolute) = parse_command_char(ch)?;

                if cmd_type == CommandType::ClosePath {
                    commands.push(PathCommand {
                        cmd_type,
                        is_absolute: true,
                        params: vec![],
                    });
                    last_cmd_type = None;
                } else {
                    last_cmd_type = Some((cmd_type, is_absolute));
                }
            }
            '0'..='9' | '.' | '-' | '+' | 'e' | 'E' => {
                if ch == '-' || ch == '+' {
                    // Start new number if not at beginning of current number
                    if !current_num.is_empty() && in_number {
                        if let Ok(num) = current_num.parse::<f64>() {
                            current_nums.push(num);
                        }
                        current_num.clear();
                    }
                }
                current_num.push(ch);
                in_number = true;
            }
            ' ' | ',' | '\t' | '\n' | '\r' => {
                // Number separator
                if !current_num.is_empty() {
                    if let Ok(num) = current_num.parse::<f64>() {
                        current_nums.push(num);
                    }
                    current_num.clear();
                    in_number = false;
                }
            }
            _ => {
                // Ignore other characters
            }
        }
    }

    // Finish last number
    if !current_num.is_empty() {
        if let Ok(num) = current_num.parse::<f64>() {
            current_nums.push(num);
        }
    }

    // Process final accumulated numbers
    if let Some(cmd_type) = last_cmd_type {
        process_accumulated_params(&mut commands, cmd_type, &mut current_nums)?;
    }

    Ok(commands)
}

/// Parse a command character into its type and whether it's absolute
fn parse_command_char(ch: char) -> Result<(CommandType, bool), String> {
    match ch {
        'M' => Ok((CommandType::MoveTo, true)),
        'm' => Ok((CommandType::MoveTo, false)),
        'L' => Ok((CommandType::LineTo, true)),
        'l' => Ok((CommandType::LineTo, false)),
        'H' => Ok((CommandType::HorizontalLineTo, true)),
        'h' => Ok((CommandType::HorizontalLineTo, false)),
        'V' => Ok((CommandType::VerticalLineTo, true)),
        'v' => Ok((CommandType::VerticalLineTo, false)),
        'C' => Ok((CommandType::CurveTo, true)),
        'c' => Ok((CommandType::CurveTo, false)),
        'S' => Ok((CommandType::SmoothCurveTo, true)),
        's' => Ok((CommandType::SmoothCurveTo, false)),
        'Q' => Ok((CommandType::QuadraticBezier, true)),
        'q' => Ok((CommandType::QuadraticBezier, false)),
        'T' => Ok((CommandType::SmoothQuadraticBezier, true)),
        't' => Ok((CommandType::SmoothQuadraticBezier, false)),
        'A' => Ok((CommandType::Arc, true)),
        'a' => Ok((CommandType::Arc, false)),
        'Z' | 'z' => Ok((CommandType::ClosePath, true)),
        _ => Err(format!("Unknown command character: {}", ch)),
    }
}

/// Process accumulated parameters for a command type
fn process_accumulated_params(
    commands: &mut Vec<PathCommand>,
    (cmd_type, is_absolute): (CommandType, bool),
    params: &mut Vec<f64>,
) -> Result<(), String> {
    let expected = match cmd_type {
        CommandType::MoveTo | CommandType::LineTo => 2,
        CommandType::HorizontalLineTo | CommandType::VerticalLineTo => 1,
        CommandType::CurveTo => 6,
        CommandType::SmoothCurveTo | CommandType::QuadraticBezier => 4,
        CommandType::SmoothQuadraticBezier => 2,
        CommandType::Arc => 7,
        CommandType::ClosePath => 0,
    };

    if expected == 0 {
        return Ok(());
    }

    // Process params in chunks
    while params.len() >= expected {
        let chunk: Vec<f64> = params.drain(..expected).collect();

        // Special case: MoveTo followed by implicit LineTo
        let actual_cmd_type = if cmd_type == CommandType::MoveTo && !commands.is_empty() {
            CommandType::LineTo
        } else {
            cmd_type
        };

        commands.push(PathCommand {
            cmd_type: actual_cmd_type,
            is_absolute,
            params: chunk,
        });
    }

    if !params.is_empty() {
        // Leftover params - this is technically an error but we'll ignore them
        params.clear();
    }

    Ok(())
}

/// Optimize path data string
fn optimize_path_data(path_data: &str, config: &PathOptimizationConfig) -> Result<String, String> {
    // Parse path data
    let mut commands = parse_path_data(path_data)?;

    // Track current position for relative/absolute conversions
    let mut current_x = 0.0;
    let mut current_y = 0.0;
    let mut start_x = 0.0;
    let mut start_y = 0.0;

    // Convert to absolute coordinates for processing
    for cmd in &mut commands {
        match cmd.cmd_type {
            CommandType::MoveTo => {
                if !cmd.is_absolute && cmd.params.len() >= 2 {
                    cmd.params[0] += current_x;
                    cmd.params[1] += current_y;
                    cmd.is_absolute = true;
                }
                if cmd.params.len() >= 2 {
                    current_x = cmd.params[0];
                    current_y = cmd.params[1];
                    start_x = current_x;
                    start_y = current_y;
                }
            }
            CommandType::LineTo => {
                if !cmd.is_absolute && cmd.params.len() >= 2 {
                    cmd.params[0] += current_x;
                    cmd.params[1] += current_y;
                    cmd.is_absolute = true;
                }
                if cmd.params.len() >= 2 {
                    current_x = cmd.params[0];
                    current_y = cmd.params[1];
                }
            }
            CommandType::HorizontalLineTo => {
                if !cmd.is_absolute && !cmd.params.is_empty() {
                    cmd.params[0] += current_x;
                    cmd.is_absolute = true;
                }
                if !cmd.params.is_empty() {
                    current_x = cmd.params[0];
                }
            }
            CommandType::VerticalLineTo => {
                if !cmd.is_absolute && !cmd.params.is_empty() {
                    cmd.params[0] += current_y;
                    cmd.is_absolute = true;
                }
                if !cmd.params.is_empty() {
                    current_y = cmd.params[0];
                }
            }
            CommandType::CurveTo => {
                if !cmd.is_absolute && cmd.params.len() >= 6 {
                    cmd.params[0] += current_x;
                    cmd.params[1] += current_y;
                    cmd.params[2] += current_x;
                    cmd.params[3] += current_y;
                    cmd.params[4] += current_x;
                    cmd.params[5] += current_y;
                    cmd.is_absolute = true;
                }
                if cmd.params.len() >= 6 {
                    current_x = cmd.params[4];
                    current_y = cmd.params[5];
                }
            }
            CommandType::SmoothCurveTo => {
                if !cmd.is_absolute && cmd.params.len() >= 4 {
                    cmd.params[0] += current_x;
                    cmd.params[1] += current_y;
                    cmd.params[2] += current_x;
                    cmd.params[3] += current_y;
                    cmd.is_absolute = true;
                }
                if cmd.params.len() >= 4 {
                    current_x = cmd.params[2];
                    current_y = cmd.params[3];
                }
            }
            CommandType::QuadraticBezier => {
                if !cmd.is_absolute && cmd.params.len() >= 4 {
                    cmd.params[0] += current_x;
                    cmd.params[1] += current_y;
                    cmd.params[2] += current_x;
                    cmd.params[3] += current_y;
                    cmd.is_absolute = true;
                }
                if cmd.params.len() >= 4 {
                    current_x = cmd.params[2];
                    current_y = cmd.params[3];
                }
            }
            CommandType::SmoothQuadraticBezier => {
                if !cmd.is_absolute && cmd.params.len() >= 2 {
                    cmd.params[0] += current_x;
                    cmd.params[1] += current_y;
                    cmd.is_absolute = true;
                }
                if cmd.params.len() >= 2 {
                    current_x = cmd.params[0];
                    current_y = cmd.params[1];
                }
            }
            CommandType::Arc => {
                if !cmd.is_absolute && cmd.params.len() >= 7 {
                    cmd.params[5] += current_x;
                    cmd.params[6] += current_y;
                    cmd.is_absolute = true;
                }
                if cmd.params.len() >= 7 {
                    current_x = cmd.params[5];
                    current_y = cmd.params[6];
                }
            }
            CommandType::ClosePath => {
                current_x = start_x;
                current_y = start_y;
            }
        }
    }

    // Apply optimizations
    if config.remove_useless {
        commands = remove_useless_commands(commands);
    }

    if config.collapse_repeated {
        commands = collapse_repeated_commands(commands);
    }

    // Convert back to string
    stringify_commands(
        &commands,
        config.float_precision,
        config.utilize_absolute,
        config.leading_zero,
        config.negative_extra_space,
    )
}

/// Remove useless commands (e.g., LineTo to current position)
fn remove_useless_commands(mut commands: Vec<PathCommand>) -> Vec<PathCommand> {
    let mut result = Vec::new();
    let mut current_x = 0.0;
    let mut current_y = 0.0;

    for cmd in commands.drain(..) {
        let mut keep = true;

        match cmd.cmd_type {
            CommandType::LineTo => {
                if cmd.params.len() >= 2 {
                    // Remove LineTo that goes to current position
                    if (cmd.params[0] - current_x).abs() < f64::EPSILON
                        && (cmd.params[1] - current_y).abs() < f64::EPSILON
                    {
                        keep = false;
                    } else {
                        current_x = cmd.params[0];
                        current_y = cmd.params[1];
                    }
                }
            }
            CommandType::HorizontalLineTo => {
                if !cmd.params.is_empty() {
                    if (cmd.params[0] - current_x).abs() < f64::EPSILON {
                        keep = false;
                    } else {
                        current_x = cmd.params[0];
                    }
                }
            }
            CommandType::VerticalLineTo => {
                if !cmd.params.is_empty() {
                    if (cmd.params[0] - current_y).abs() < f64::EPSILON {
                        keep = false;
                    } else {
                        current_y = cmd.params[0];
                    }
                }
            }
            CommandType::MoveTo => {
                if cmd.params.len() >= 2 {
                    current_x = cmd.params[0];
                    current_y = cmd.params[1];
                }
            }
            CommandType::CurveTo => {
                if cmd.params.len() >= 6 {
                    current_x = cmd.params[4];
                    current_y = cmd.params[5];
                }
            }
            CommandType::SmoothCurveTo => {
                if cmd.params.len() >= 4 {
                    current_x = cmd.params[2];
                    current_y = cmd.params[3];
                }
            }
            CommandType::QuadraticBezier => {
                if cmd.params.len() >= 4 {
                    current_x = cmd.params[2];
                    current_y = cmd.params[3];
                }
            }
            CommandType::SmoothQuadraticBezier => {
                if cmd.params.len() >= 2 {
                    current_x = cmd.params[0];
                    current_y = cmd.params[1];
                }
            }
            CommandType::Arc => {
                if cmd.params.len() >= 7 {
                    current_x = cmd.params[5];
                    current_y = cmd.params[6];
                }
            }
            _ => {}
        }

        if keep {
            result.push(cmd);
        }
    }

    result
}

/// Collapse repeated commands where possible
fn collapse_repeated_commands(commands: Vec<PathCommand>) -> Vec<PathCommand> {
    // For now, just return as-is
    // TODO: Implement command collapsing
    commands
}

/// Convert commands back to string format
fn stringify_commands(
    commands: &[PathCommand],
    precision: u8,
    utilize_absolute: bool,
    leading_zero: bool,
    negative_extra_space: bool,
) -> Result<String, String> {
    let mut result = String::new();
    let mut last_cmd_char = '\0';
    let mut current_x = 0.0;
    let mut current_y = 0.0;

    for (i, cmd) in commands.iter().enumerate() {
        // Decide whether to use absolute or relative
        let mut use_absolute = cmd.is_absolute;
        if utilize_absolute && i > 0 {
            // Calculate which is shorter
            use_absolute = should_use_absolute(cmd, current_x, current_y, precision);
        }

        // Get the command character
        let cmd_char = if use_absolute {
            cmd.get_char().to_ascii_uppercase()
        } else {
            cmd.get_char().to_ascii_lowercase()
        };

        // Add command character if different from last
        if cmd_char != last_cmd_char || cmd.cmd_type == CommandType::MoveTo {
            if !result.is_empty() && cmd_char != 'Z' && cmd_char != 'z' {
                result.push(' ');
            }
            result.push(cmd_char);
            last_cmd_char = cmd_char;
        } else if !result.is_empty() {
            result.push(' ');
        }

        // Add parameters
        let params = if use_absolute {
            cmd.params.clone()
        } else {
            convert_to_relative(cmd, current_x, current_y)
        };

        for (j, &param) in params.iter().enumerate() {
            if j > 0 || (cmd_char != last_cmd_char && cmd_char != 'Z' && cmd_char != 'z') {
                // Add space before parameter unless it's negative and we're saving space
                if !negative_extra_space || param >= 0.0 || j == 0 {
                    result.push(' ');
                }
            }

            result.push_str(&format_number(param, precision, leading_zero));
        }

        // Update current position
        update_position(cmd, &mut current_x, &mut current_y);
    }

    Ok(result)
}

/// Determine if absolute coordinates are more efficient
fn should_use_absolute(
    _cmd: &PathCommand,
    _current_x: f64,
    _current_y: f64,
    _precision: u8,
) -> bool {
    // For now, always use absolute
    // TODO: Implement size comparison
    true
}

/// Convert absolute coordinates to relative
fn convert_to_relative(cmd: &PathCommand, current_x: f64, current_y: f64) -> Vec<f64> {
    let mut params = cmd.params.clone();

    match cmd.cmd_type {
        CommandType::MoveTo | CommandType::LineTo => {
            if params.len() >= 2 {
                params[0] -= current_x;
                params[1] -= current_y;
            }
        }
        CommandType::HorizontalLineTo => {
            if !params.is_empty() {
                params[0] -= current_x;
            }
        }
        CommandType::VerticalLineTo => {
            if !params.is_empty() {
                params[0] -= current_y;
            }
        }
        CommandType::CurveTo => {
            if params.len() >= 6 {
                params[0] -= current_x;
                params[1] -= current_y;
                params[2] -= current_x;
                params[3] -= current_y;
                params[4] -= current_x;
                params[5] -= current_y;
            }
        }
        CommandType::SmoothCurveTo => {
            if params.len() >= 4 {
                params[0] -= current_x;
                params[1] -= current_y;
                params[2] -= current_x;
                params[3] -= current_y;
            }
        }
        CommandType::QuadraticBezier => {
            if params.len() >= 4 {
                params[0] -= current_x;
                params[1] -= current_y;
                params[2] -= current_x;
                params[3] -= current_y;
            }
        }
        CommandType::SmoothQuadraticBezier => {
            if params.len() >= 2 {
                params[0] -= current_x;
                params[1] -= current_y;
            }
        }
        CommandType::Arc => {
            if params.len() >= 7 {
                params[5] -= current_x;
                params[6] -= current_y;
            }
        }
        _ => {}
    }

    params
}

/// Update current position based on command
fn update_position(cmd: &PathCommand, current_x: &mut f64, current_y: &mut f64) {
    match cmd.cmd_type {
        CommandType::MoveTo | CommandType::LineTo => {
            if cmd.params.len() >= 2 {
                *current_x = cmd.params[0];
                *current_y = cmd.params[1];
            }
        }
        CommandType::HorizontalLineTo => {
            if !cmd.params.is_empty() {
                *current_x = cmd.params[0];
            }
        }
        CommandType::VerticalLineTo => {
            if !cmd.params.is_empty() {
                *current_y = cmd.params[0];
            }
        }
        CommandType::CurveTo => {
            if cmd.params.len() >= 6 {
                *current_x = cmd.params[4];
                *current_y = cmd.params[5];
            }
        }
        CommandType::SmoothCurveTo => {
            if cmd.params.len() >= 4 {
                *current_x = cmd.params[2];
                *current_y = cmd.params[3];
            }
        }
        CommandType::QuadraticBezier => {
            if cmd.params.len() >= 4 {
                *current_x = cmd.params[2];
                *current_y = cmd.params[3];
            }
        }
        CommandType::SmoothQuadraticBezier => {
            if cmd.params.len() >= 2 {
                *current_x = cmd.params[0];
                *current_y = cmd.params[1];
            }
        }
        CommandType::Arc => {
            if cmd.params.len() >= 7 {
                *current_x = cmd.params[5];
                *current_y = cmd.params[6];
            }
        }
        _ => {}
    }
}

/// Format a number with optional precision
fn format_number(value: f64, precision: u8, leading_zero: bool) -> String {
    // Format with precision
    let formatted = format!("{:.1$}", value, precision as usize);

    // Remove trailing zeros and decimal point if integer
    let mut trimmed = formatted.trim_end_matches('0').trim_end_matches('.');

    // Handle edge cases
    if trimmed.is_empty() || trimmed == "-" {
        return "0".to_string();
    }

    // Remove leading zero if requested
    if !leading_zero && trimmed.starts_with("0.") {
        trimmed = &trimmed[1..];
    } else if !leading_zero && trimmed.starts_with("-0.") {
        return format!("-{}", &trimmed[2..]);
    }

    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_path() {
        let path = "M10 20 L30 40";
        let commands = parse_path_data(path).unwrap();
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].cmd_type, CommandType::MoveTo);
        assert_eq!(commands[0].params, vec![10.0, 20.0]);
        assert_eq!(commands[1].cmd_type, CommandType::LineTo);
        assert_eq!(commands[1].params, vec![30.0, 40.0]);
    }

    #[test]
    fn test_parse_relative_path() {
        let path = "m10 20 l30 40";
        let commands = parse_path_data(path).unwrap();
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].cmd_type, CommandType::MoveTo);
        assert!(!commands[0].is_absolute);
        assert_eq!(commands[1].cmd_type, CommandType::LineTo);
        assert!(!commands[1].is_absolute);
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(1.0, 3, true), "1");
        assert_eq!(format_number(1.234567, 3, true), "1.235");
        assert_eq!(format_number(0.5, 1, false), ".5");
        assert_eq!(format_number(-0.5, 1, false), "-.5");
    }

    #[test]
    fn test_optimize_removes_useless_lineto() {
        let path = "M10 10 L10 10 L20 20";
        let config = PathOptimizationConfig {
            float_precision: 3,
            transform_precision: 5,
            remove_useless: true,
            collapse_repeated: true,
            utilize_absolute: true,
            leading_zero: true,
            negative_extra_space: true,
        };
        let optimized = optimize_path_data(path, &config).unwrap();
        // Should remove the L10 10 as it's the same as current position
        assert!(!optimized.contains("L10 10"));
    }
}
