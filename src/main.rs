mod css;
mod dom;

use dom::Node;
use css::stylesheet::{Stylesheet, Selector};

use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::Peekable;
use std::rc::Rc;
use std::str::Chars;

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

    //this is getting the tag name
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

    //this is getting the attributes
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
    let css_input = "div { color: red; font-size: 16px; }
                    .box { margin: 10px; }
                    #main { padding: 5px; }";

    let stylesheet = Stylesheet::parse_css(css_input);
    println!("{:#?}", stylesheet);

    let html_input = "<html><body class=\"container\"><div id=\"main\" class=\"box\">attribute parsing</div><p>Hello World</p></body></html>";
    let tokens = tokenize(html_input);
    let dom = dom_builder(tokens);
    println!("{:#?}", &dom);
    println!();
    Node::print(&dom);
    println!();
    // for i in tokens {
    //     match i {
    //         Token::StartTag {
    //             tag_name,
    //             attributes,
    //         } => {
    //             println!("StartTag: {}, attrs: {:?}", tag_name, attributes);
    //         }
    //         Token::EndTag { tag_name } => {
    //             println!("EndTag: {}", tag_name);
    //         }
    //         Token::Text(text) => {
    //             println!("Text: {}", text);
    //         }
    //     }
    // }
}
