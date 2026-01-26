use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::vec;

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

#[derive(Debug, Clone)]
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


fn main() {
    let text = Node::text("Hello World".into());
    let title_text = Node::text("Document".into());

    let p = Node::element(
        "p".into(), 
        HashMap::from([
            ("id".to_string(), "navbar".to_string()),
        ]), 
        vec![text],
    );

    let body = Node::element(
        "body".into(),
        HashMap::from([
            ("className".to_string(), "bg-black".to_string()),
            ("title".to_string(), "drag_to_sidebar".to_string()),
        ]),
        vec![p],
    );


    let title = Node::element(
        "title".into(),
        HashMap::new(),
        vec![title_text],
    );

    let head = Node::element(
        "head".into(),
        HashMap::new(),
        vec![title],
    );

    let html = Node::element(
        "html".into(),
        HashMap::from([
            ("lang".to_string(), "en".to_string()),
        ]),
        vec![head, body],
    );

    Node::print(&html); 
}
