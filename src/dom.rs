use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type NodeRef = Rc<RefCell<Node>>;
pub type AttributeMap = HashMap<String, String>;

#[derive(Debug, Clone)]
pub struct Node {
    pub children: Vec<NodeRef>,
    pub node_type: NodeType,
}

#[derive(Debug, Clone)]
pub enum NodeType {
    Element(ElementData),
    Text(String),
}

#[derive(Debug, Clone, Default)]
pub struct ElementData {
    pub tag_name: String,
    pub attributes: AttributeMap,
}

impl Node {
    pub fn text(data: String) -> NodeRef {
        let text_node = Node {
            children: vec![],
            node_type: NodeType::Text(data),
        };
        return Rc::new(RefCell::new(text_node));
    }

    pub fn element(tag_name: String, attributes: AttributeMap, children: Vec<NodeRef>) -> NodeRef {
        let element_data = ElementData {
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
    fn dom_traverse_print(node: &NodeRef, indent: usize) {
        let node = node.borrow();
        let padding = " ".repeat(indent);

        match &node.node_type {
            NodeType::Text(text) => {
                println!("{}\"{}\"", padding, text);
            }

            NodeType::Element(ele) => {
                print!("{}<{}", padding, ele.tag_name);
                for (k, v) in &ele.attributes {
                    print!(" {}=\"{}\"", k, v);
                }
                println!(">");

                for child in &node.children {
                    Node::dom_traverse_print(child, indent + 2);
                }
                println!("{}</{}>", padding, ele.tag_name);
            }
        }
    }

    pub fn print(node: &NodeRef) {
        Self::dom_traverse_print(node, 0);
    }
}

