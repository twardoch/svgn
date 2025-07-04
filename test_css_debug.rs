use lightningcss::{
    stylesheet::{ParserOptions, StyleSheet},
    rules::CssRule,
};

fn main() {
    let css = r#"
        .test { 
            fill: #ff0000; 
            opacity: 0.75;
            stroke-width: 3px;
            font-size: 16px;
        }
    "#;
    
    let stylesheet = StyleSheet::<'_, '_>::parse(css, ParserOptions::default()).unwrap();
    
    for rule in &stylesheet.rules.0 {
        if let CssRule::Style(style_rule) = rule {
            for property in &style_rule.declarations.declarations {
                println!("Property debug: {:?}", property);
            }
        }
    }
}