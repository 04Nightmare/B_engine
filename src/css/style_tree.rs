use std::collections::HashMap;

use crate::{Selector, Stylesheet};
use crate::dom::{ElementData, NodeRef, NodeType};

pub type PropertyMap = HashMap<String, String>;
type Specificity = (u8, u8, u8);

pub struct StyledNode {
    pub node: NodeRef,       //Rc<RefCell<Node>>
    pub specified_values: PropertyMap,
    pub children: Vec<StyledNode>,
}

fn specificity(selector: &Selector) -> Specificity {
    match selector {
        Selector::Id(_) => (1, 0, 0),
        Selector::Class(_) => (0, 1, 0),
        Selector::Type(_) => (0, 0, 1),
    }
}

fn matches(element: &ElementData, selector: &Selector) -> bool {
    match selector{
        Selector::Type(tag) => &element.tag_name == tag,
        Selector::Class(class_name) => {
            if let Some(classes) = element.attributes.get("class"){
                classes.split_whitespace().any(|c| c == class_name)
            }else{
                false
            }
        }
        Selector::Id(id) => {
            element.attributes.get("id") == Some(id)
        }
    }
}

struct MatchedRule<'a> {
    specificity: Specificity,
    declarations: &'a Vec<crate::css::stylesheet::Declaration>,
}

fn match_rules<'a>(element: &ElementData, stylesheet: &'a Stylesheet) -> Vec<MatchedRule<'a>> {
    let mut matched = Vec::new();

    for rule in &stylesheet.rules {
        let best = rule.selectors.iter()
            .filter(|s| matches(element, s))
            .map(|s| specificity(s))
            .max();

        if let Some(spec) = best {
            matched.push(MatchedRule {
                specificity: spec,
                declarations: &rule.declarations,
            });
        }
    }

    return matched;
}

fn cascade(mut matched_rules: Vec<MatchedRule>) -> PropertyMap {
    matched_rules.sort_by_key(|r| r.specificity);

    let mut values = PropertyMap::new();
    for rule in matched_rules {
        for decl in rule.declarations {
            values.insert(decl.name.clone(), decl.value.clone());
        }
    }
    return values;
}

//Style Tree Builder
impl StyledNode{
    pub fn style_tree_builder(node: &NodeRef, stylesheet: &Stylesheet) -> StyledNode{
        let specified_values = match &node.borrow().node_type {
            NodeType::Element(element) => {
                let matched = match_rules(element, stylesheet);
                cascade(matched)
            }
            NodeType::Text(_) => PropertyMap::new(),
        };

        let child_refs = node.borrow().children.clone();
        let children = child_refs.iter()
            .map(|child| StyledNode::style_tree_builder(child, stylesheet))
            .collect();

        return StyledNode { 
            node: node.clone(), 
            specified_values, 
            children 
        };
    }

    pub fn print_style_tree(style_node: &StyledNode, indent: usize){
        let padding = " ".repeat(indent);
        let borrowed = style_node.node.borrow();

        match &borrowed.node_type {
            NodeType::Text(t) => println!("{}Text: \"{}\"", padding, t),
            NodeType::Element(e) => {
                println!("{}<{}>", padding, e.tag_name);
                if !style_node.specified_values.is_empty() {
                    let mut props: Vec<_> = style_node.specified_values.iter().collect();
                    props.sort_by_key(|(k, _)| *k);
                    for (prop, val) in props {
                        println!("{}  {}: {}", padding, prop, val);
                    }
                }
            }
        }
        for child in &style_node.children {
            StyledNode::print_style_tree(child, indent + 2);
        }
    }
}


