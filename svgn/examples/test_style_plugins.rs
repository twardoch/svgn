// Example program to test the new style plugins

use svgn::parser::Parser;
use svgn::stringifier::Stringifier;
use svgn::plugin::{Plugin, PluginInfo};
use svgn::plugins::{RemoveStyleElement, MergeStylesPlugin, ConvertStyleToAttrsPlugin};

fn main() {
    // Test RemoveStyleElement
    println!("Testing RemoveStyleElement plugin:");
    let svg1 = r#"<svg>
        <style>.cls-1{fill:red;}</style>
        <rect class="cls-1" width="10" height="10"/>
        <style>.cls-2{stroke:blue;}</style>
    </svg>"#;
    
    let parser = Parser::new();
    let mut doc1 = parser.parse(svg1).unwrap();
    let mut plugin1 = RemoveStyleElement;
    let plugin_info = PluginInfo { path: None, multipass_count: 0 };
    plugin1.apply(&mut doc1, &plugin_info, None).unwrap();
    
    let stringifier = Stringifier::new();
    let output1 = stringifier.stringify(&doc1).unwrap();
    println!("Original:\n{}", svg1);
    println!("After RemoveStyleElement:\n{}\n", output1);
    
    // Test MergeStyles
    println!("Testing MergeStyles plugin:");
    let svg2 = r#"<svg>
        <style>.a{fill:red}</style>
        <style>.b{fill:blue}</style>
        <rect class="a"/>
        <rect class="b"/>
    </svg>"#;
    
    let mut doc2 = parser.parse(svg2).unwrap();
    let mut plugin2 = MergeStylesPlugin;
    plugin2.apply(&mut doc2, &plugin_info, None).unwrap();
    
    let output2 = stringifier.stringify(&doc2).unwrap();
    println!("Original:\n{}", svg2);
    println!("After MergeStyles:\n{}\n", output2);
    
    // Test ConvertStyleToAttrs
    println!("Testing ConvertStyleToAttrs plugin:");
    let svg3 = r#"<svg>
        <rect style="fill: red; stroke: blue; opacity: 0.5" width="100" height="100"/>
        <circle style="fill: green; custom-prop: value" r="50"/>
    </svg>"#;
    
    let mut doc3 = parser.parse(svg3).unwrap();
    let mut plugin3 = ConvertStyleToAttrsPlugin;
    plugin3.apply(&mut doc3, &plugin_info, None).unwrap();
    
    let output3 = stringifier.stringify(&doc3).unwrap();
    println!("Original:\n{}", svg3);
    println!("After ConvertStyleToAttrs:\n{}", output3);
}