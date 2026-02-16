use std::iter::Peekable;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::{HashMap};
use std::str::Chars;

#[allow(unused, unused_assignments, dead_code)]
type NodeRef = Rc<RefCell<Node>>;
type AttributeMap = HashMap<String, String>;

#[derive(Debug, Clone)]
struct Node{
    children: Vec<NodeRef>,
    node_type: NodeType,
}

#[derive(Debug, Clone)]
enum NodeType{
    Element(ElementData),
    Text(String),
}

#[derive(Debug, Clone, Default)]
struct ElementData{
    tag_name: String,
    attributes: AttributeMap,
}


impl Node {
    fn text(data: String) -> NodeRef{
        let text_node = Node {
            children: vec![],
            node_type: NodeType::Text(data),
        };
        return Rc::new(RefCell::new(text_node));
    }

    fn element(tag_name: String, attributes: AttributeMap, children: Vec<NodeRef>) -> NodeRef {
        let element_data = ElementData{
            tag_name,
            attributes,
        };
        let element_node = Node {
            children,
            node_type: NodeType::Element(element_data),
        };
        return Rc::new(RefCell::new(element_node));
    }
}

impl Node {
    fn dom_traverse_print(node: &NodeRef, indent: usize){
        let node = node.borrow();
        let padding = " ".repeat(indent);

        match &node.node_type {
            NodeType::Text(text) => {
                println!("{}\"{}\"", padding, text);
            }

            NodeType::Element(ele) => {
                print!("{}<{}",padding, ele.tag_name);
                for (k, v) in &ele.attributes{
                    print!(" {}=\"{}\"", k, v);
                }
                println!(">");

                for child in &node.children{
                    Node::dom_traverse_print(child, indent+2);
                }
                println!("{}</{}>", padding, ele.tag_name);
            }
        }
    }

    fn print(node: &NodeRef){
        Self::dom_traverse_print(node, 0);
    }
}


//tokens
#[derive(Debug, Clone)]
enum Token{
    StartTag{
        tag_name: String,
        attributes: AttributeMap
    },
    EndTag{
        tag_name: String,
    },
    Text(String),
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next(){
        if c == '<'{
            if let Some('/') = chars.peek(){
                chars.next();
                let tag = collect_tag_content_until(&mut chars, '>');
                tokens.push(Token::EndTag {
                    tag_name: tag
                });
                chars.next();
            }else{
                let full_content = collect_tag_content_until(&mut chars, '>');
                let (tag_name, attributes) = parse_tag_content(&full_content);
                tokens.push(Token::StartTag {
                    tag_name: tag_name, 
                    attributes: attributes,
                });
                chars.next();
            }
        }else{
            let mut text = String::new();
            text.push(c);
            while let Some(&next) = chars.peek(){
                if next == '<'{
                    break;
                }
                text.push(chars.next().unwrap());
            }
            if !text.trim().is_empty(){
                tokens.push(Token::Text(text));
            }
        }
    }
    return tokens;
}

fn collect_tag_content_until(chars: &mut Peekable<Chars>, stop_char: char) -> String{
    let mut tag_name = String::new();
    while let Some(&ch) = chars.peek(){
        if ch == stop_char{
            break;
        }
        tag_name.push(chars.next().unwrap());
    }
    return tag_name.trim().to_string();
}

fn parse_tag_content(input: &str) -> (String, AttributeMap){
    let mut char_iter = input.chars().peekable();
    let mut tag_name = String::new();
    while let Some(&ch) = char_iter.peek(){
        if ch.is_whitespace(){
            break;
        }
        tag_name.push(char_iter.next().unwrap());
    }
    
    while let Some(&ch) = char_iter.peek(){
        if !ch.is_whitespace(){
            break;
        }
        char_iter.next();
    }

    let mut attributes = HashMap::new();

    while char_iter.peek().is_some(){
        let (key, value) = parse_attributes(&mut char_iter);
        attributes.insert(key, value);
        while let Some(&ch) = char_iter.peek(){
            if !ch.is_whitespace(){
                break;
            }
            char_iter.next();
        }
    }
    return (tag_name, attributes);
}

fn parse_attributes(char_iter: &mut Peekable<Chars>) -> (String, String){
    let mut key = String::new();
    while let Some(&ch) = char_iter.peek(){
        if ch == '='{
            break;
        }
        key.push(char_iter.next().unwrap());
    }
    char_iter.next();
    char_iter.next();
    let mut value = String::new();
    while let Some(&ch) = char_iter.peek(){
        if ch == '"'{
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

    for token in tokens{
        match token {
            Token::StartTag { tag_name, .. } => {
                let node = Node::element(tag_name, Default::default(), vec![]);
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
            Token::EndTag {..} => {
                stack.pop();
            }
        }
    }
    root
}


fn main() {
    let html_input = "<html><body><div>attribute parsing</div><p>Hello World</p></body></html>";
    let tokens = tokenize(html_input);
    let dom = dom_builder(tokens.clone());
    Node::print(&dom);
    println!();
    for i in tokens{
        match i{
            Token::StartTag{tag_name, attributes} => {
                println!("StartTag: {}, attrs: {:?}", tag_name, attributes);
            }
            Token::EndTag{tag_name} => {
                println!("EndTag: {}", tag_name);
            }
            Token::Text(text) => {
                println!("Text: {}", text);
            }
        }
    }

}
