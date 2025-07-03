
this_file: svgn/src/plugins/convert_colors.rs

use std::collections::HashMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use crate::ast::*;
use crate::plugin::{Plugin, PluginInfo};
use crate::collections::{COLORS_NAMES, COLORS_PROPS, COLORS_SHORT_NAMES};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
enum CurrentColorConfig {
    Bool(bool),
    String(String),
    Regex(String), // Store as string, compile later
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertColorsConfig {
    #[serde(default)]
    pub current_color: Option<CurrentColorConfig>,
    #[serde(default = "default_true")]
    pub names2hex: bool,
    #[serde(default = "default_true")]
    pub rgb2hex: bool,
    #[serde(default = "default_convert_case")]
    pub convert_case: Option<String>, // "lower", "upper", or None
    #[serde(default = "default_true")]
    pub shorthex: bool,
    #[serde(default = "default_true")]
    pub shortname: bool,
}

fn default_true() -> bool {
    true
}

fn default_convert_case() -> Option<String> {
    Some("lower".to_string())
}

pub struct ConvertColors {
    config: ConvertColorsConfig,
    reg_rgb: Regex,
    reg_hex: Regex,
    mask_counter: usize,
}

impl ConvertColors {
    pub fn new(config: ConvertColorsConfig) -> Self {
        Self {
            config,
            reg_rgb: Regex::new(r"^rgb\(\s*([+-]?(?:\d*\.\d+|\d+\.?)(?:%)?)\s*(?:,\s*|\s+)([+-]?(?:\d*\.\d+|\d+\.?)(?:%)?)\s*(?:,\s*|\s+)([+-]?(?:\d*\.\d+|\d+\.?)(?:%)?)\s*\)$").unwrap(),
            reg_hex: Regex::new(r"^#(([a-fA-F0-9])\2){3}$").unwrap(),
            mask_counter: 0,
        }
    }

    fn convert_rgb_to_hex(&self, r: f64, g: f64, b: f64) -> String {
        let hex_number = (((256 + r as u32) << 8) | g as u32) << 8 | b as u32;
        format!("#{}", &format!("{:x}", hex_number)[1..]).to_uppercase()
    }

    fn includes_url_reference(&self, value: &str) -> bool {
        value.contains("url(")
    }
}

impl Plugin for ConvertColors {
    fn name(&self) -> &str {
        "convertColors"
    }

    fn description(&self) -> &'static str {
        "converts colors: rgb() to #rrggbb and #rrggbb to #rgb"
    }

    fn apply(&mut self, document: &mut Document, _plugin_info: &PluginInfo, _params: Option<&serde_json::Value>) -> PluginResult<()> {
        for node_idx in document.tree.traverse() {
            if let Some(node) = document.tree.get_mut(node_idx) {
                if let Node::Element(element) = node {
                    if element.name == "mask" {
                        self.mask_counter += 1;
                    }

                    for (name, value) in element.attributes.iter_mut() {
                        if COLORS_PROPS.contains(name.as_str()) {
                            let mut val = value.clone();

                            // convert colors to currentColor
                            if let Some(current_color_config) = &self.config.current_color {
                                if self.mask_counter == 0 {
                                    let mut matched = false;
                                    match current_color_config {
                                        CurrentColorConfig::Bool(b) => {
                                            if *b {
                                                matched = val != "none";
                                            }
                                        },
                                        CurrentColorConfig::String(s) => {
                                            matched = val == *s;
                                        },
                                        CurrentColorConfig::Regex(r_str) {
                                            if let Ok(regex) = Regex::new(r_str) {
                                                matched = regex.is_match(&val);
                                            }
                                        },
                                    }
                                    if matched {
                                        val = "currentColor".to_string();
                                    }
                                }
                            }

                            // convert color name keyword to long hex
                            if self.config.names2hex {
                                if let Some(hex_val) = COLORS_NAMES.get(&val.to_lowercase()) {
                                    val = hex_val.to_string();
                                }
                            }

                            // convert rgb() to long hex
                            if self.config.rgb2hex {
                                if let Some(captures) = self.reg_rgb.captures(&val) {
                                    let mut nums = Vec::new();
                                    for i in 1..4 {
                                        let m = captures.get(i).unwrap().as_str();
                                        let n = if m.contains('%') {
                                            (m.replace('%', "").parse::<f64>().unwrap_or(0.0) * 2.55).round()
                                        } else {
                                            m.parse::<f64>().unwrap_or(0.0)
                                        };
                                        nums.push(n.max(0.0).min(255.0));
                                    }
                                    if nums.len() == 3 {
                                        val = self.convert_rgb_to_hex(nums[0], nums[1], nums[2]);
                                    }
                                }
                            }

                            if let Some(convert_case) = &self.config.convert_case {
                                if !self.includes_url_reference(&val) && val != "currentColor" {
                                    if convert_case == "lower" {
                                        val = val.to_lowercase();
                                    } else if convert_case == "upper" {
                                        val = val.to_uppercase();
                                    }
                                }
                            }

                            // convert long hex to short hex
                            if self.config.shorthex {
                                if let Some(captures) = self.reg_hex.captures(&val) {
                                    let hex_str = captures.get(0).unwrap().as_str();
                                    if hex_str.len() == 7 { // #RRGGBB
                                        let r = hex_str.chars().nth(1).unwrap();
                                        let g = hex_str.chars().nth(3).unwrap();
                                        let b = hex_str.chars().nth(5).unwrap();
                                        val = format!("#{}{}{}", r, g, b);
                                    }
                                }
                            }

                            // convert hex to short name
                            if self.config.shortname {
                                if let Some(short_name) = COLORS_SHORT_NAMES.get(&val.to_lowercase()) {
                                    val = short_name.to_string();
                                }
                            }

                            *value = val;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
