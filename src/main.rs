mod css;
mod dom;
mod layout;
mod paint;

use dom::Node;
use css::stylesheet::{Stylesheet, Selector};
use css::style_tree::{StyledNode};
use layout::{build_layout_tree, layout, print_layout_tree, Rect, Dimensions};
use paint::{build_display_list, print_display_list, Canvas, paint, save_png};

use std::cell::RefCell;
use std::collections::HashMap;
use std::default;
use std::iter::Peekable;
use std::rc::Rc;
use std::str::Chars;

use crate::css::stylesheet;
use crate::dom::{AttributeMap, NodeRef};



#[allow(unused, unused_assignments, dead_code)]


//tokens
#[derive(Debug, Clone)]
enum Token {
    StartTag {
        tag_name: String,
        attributes: HashMap<String, String>,
    },
    EndTag {
        tag_name: String,
    },
    Text(String),
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '<' {
            if let Some('/') = chars.peek() {
                chars.next();
                let tag = collect_tag_content_until(&mut chars, '>');
                tokens.push(Token::EndTag { tag_name: tag });
                chars.next();
            } else {
                let full_content = collect_tag_content_until(&mut chars, '>');
                let (tag_name, attributes) = parse_tag_content(&full_content);
                tokens.push(Token::StartTag {
                    tag_name: tag_name,
                    attributes: attributes,
                });
                chars.next();
            }
        } else {
            let mut text = String::new();
            text.push(c);
            while let Some(&next) = chars.peek() {
                if next == '<' {
                    break;
                }
                text.push(chars.next().unwrap());
            }
            if !text.trim().is_empty() {
                tokens.push(Token::Text(text));
            }
        }
    }
    return tokens;
}

fn collect_tag_content_until(chars: &mut Peekable<Chars>, stop_char: char) -> String {
    let mut tag_name = String::new();
    while let Some(&ch) = chars.peek() {
        if ch == stop_char {
            break;
        }
        tag_name.push(chars.next().unwrap());
    }
    return tag_name.trim().to_string();
}

fn parse_tag_content(input: &str) -> (String, HashMap<String, String>) {
    let mut char_iter = input.chars().peekable();
    let mut tag_name = String::new();
    let mut attributes = HashMap::new();

    //getting the tag name
    while let Some(&ch) = char_iter.peek() {
        if ch.is_whitespace() {
            break;
        }
        tag_name.push(char_iter.next().unwrap());
    }

    //if no attributes
    if char_iter.peek().is_none() {
        return (tag_name, attributes);
    }

    //skip space
    while let Some(&ch) = char_iter.peek() {
        if !ch.is_whitespace() {
            break;
        }
        char_iter.next();
    }

    //getting the attributes
    while char_iter.peek().is_some() {
        if let Some(&ch) = char_iter.peek() {
            if ch.is_alphabetic() {
                let (key, value) = parse_attributes(&mut char_iter);
                attributes.insert(key, value);
            } else {
                break;
            }
        }
        while let Some(&ch) = char_iter.peek() {
            if !ch.is_whitespace() {
                break;
            }
            char_iter.next();
        }
    }
    return (tag_name, attributes);
}

fn parse_attributes(char_iter: &mut Peekable<Chars>) -> (String, String) {
    let mut key = String::new();
    while let Some(&ch) = char_iter.peek() {
        if ch == '=' {
            break;
        }
        key.push(char_iter.next().unwrap());
    }
    char_iter.next();
    char_iter.next();
    let mut value = String::new();
    while let Some(&ch) = char_iter.peek() {
        if ch == '"' {
            break;
        }
        value.push(char_iter.next().unwrap());
    }
    char_iter.next();
    return (key.trim().to_string(), value);
}

fn dom_builder(tokens: Vec<Token>) -> NodeRef {
    let root = Node::element("document".into(), Default::default(), vec![]);
    let mut stack = vec![root.clone()];

    for token in tokens {
        match token {
            Token::StartTag {
                tag_name,
                attributes,
            } => {
                let mut attri = HashMap::new();
                for (k, v) in attributes {
                    attri.insert(k, v);
                }
                let node = Node::element(tag_name, attri, vec![]);
                stack
                    .last()
                    .unwrap()
                    .borrow_mut()
                    .children
                    .push(node.clone());
                stack.push(node);
            }
            Token::Text(text) => {
                let node = Node::text(text);
                stack
                    .last()
                    .unwrap()
                    .borrow_mut()
                    .children
                    .push(node);
            }
            Token::EndTag { .. } => {
                stack.pop();
            }
        }
    }
    return root;
}



fn main() {
    let css_input = "
        body {
            background-color: #f0f0f0;
        }
        div {
            width: 300px;
            margin-top: 20px;
            margin-left: 30px;
            padding-top: 10px;
            padding-bottom: 10px;
            padding-left: 15px;
            padding-right: 15px;
            background-color: #4a90d9;
            border-color: #1a5fa8;
            border-top: 3px;
            border-bottom: 3px;
            border-left: 3px;
            border-right: 3px;
            color: white;
        }
        p {
            width: 300px;
            margin-top: 15px;
            margin-left: 30px;
            padding-top: 8px;
            padding-bottom: 8px;
            padding-left: 10px;
            padding-right: 10px;
            background-color: #e8f4e8;
            border-color: #5a9a5a;
            border-top: 2px;
            border-bottom: 2px;
            border-left: 2px;
            border-right: 2px;
        }
        .highlight {
            background-color: #ffd700;
            border-color: #b8960c;
        }
    ";
 
    let html_input = r#"
        <html>
          <body>
            <div>First block</div>
            <p class="highlight">A highlighted paragraph</p>
            <p>A plain paragraph</p>
            <div>Second block</div>
          </body>
        </html>
    "#;

    let stylesheet = Stylesheet::parse_css(css_input);
    let tokens = tokenize(html_input);
    let dom  = dom_builder(tokens);

    // Dom tree
    println!("=== Dom Tree ===\n");
    Node::print(&dom);

    println!();
    println!("{}", "*".repeat(50));
    println!();

    // Style tree
    let styled = StyledNode::style_tree_builder(&dom, &stylesheet);
    println!("\n=== Styled Tree ===\n");
    StyledNode::print_style_tree(&styled, 0);

    println!();
    println!("{}", "*".repeat(50));
    println!();

    // Layout Tree
    let viewport = Dimensions{
        content: Rect { x: 0.0, y: 0.0, width: 800.0, height: 0.0 },
        ..Default::default()
    };
    let mut layout_root = match build_layout_tree(&styled) {
        Some(r) => r,
        None => {
            eprintln!("Layout tree empty");
            return;
        }
    };
    layout(&mut layout_root, &viewport);
    println!("\n=== Layout Tree ===\n");
    print_layout_tree(&layout_root, 0);

    println!();
    println!("{}", "*".repeat(50));
    println!();

    // Display list
    let display_list = build_display_list(&layout_root);
    println!("\n=== Display List ===\n");
    print_display_list(&display_list);

    
    // 4. Paint → canvas → PNG
    let mut canvas = Canvas::new(800, 600);
    paint(&display_list, &mut canvas);
 
    match save_png(&canvas, "output.png") {
        Ok(_)  => println!("\n  Saved output.png  (800x600)"),
        Err(e) => eprintln!("Failed to save PNG: {}", e),
    }

    
}

// let stylesheet = Stylesheet::parse_css(css_input);
//     println!("{:#?}", stylesheet);

//     let tokens = tokenize(html_input);
//     let dom = dom_builder(tokens);
//     println!("{:#?}", &dom);
//     println!();
//     Node::print(&dom);
//     println!();
//     println!("{}", "*".repeat(50));
//     println!();

//     let styled = StyledNode::style_tree_builder(&dom, &stylesheet);
//     println!("=== Styled Tree ===\n");
//     StyledNode::print_style_tree(&styled, 0);

//     //Root browser viewpoprt
//     let viewport = Dimensions{
//         content: Rect { x: 0.0, y: 0.0, width: 800.0, height: 0.0 },
//         ..Default::default()
//     };

//     if let Some(mut layout_root) = build_layout_tree(&styled){
//         layout(&mut layout_root, &viewport);
//         println!();
//         println!("{}", "*".repeat(50));
//         println!();
//         println!("\n=== Layout Tree ===\n");
//         print_layout_tree(&layout_root, 0);
//     }