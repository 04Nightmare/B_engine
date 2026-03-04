use std::collections::HashMap;

use crate::{Selector, Stylesheet};
use crate::dom::{ElementData, NodeRef, NodeType};

pub type PropertyMap = HashMap<String, String>;
type Specificity = (u8, u8, u8);

pub struct StyledNode {
    pub node: NodeRef,
    pub specified_values: PropertyMap,
    pub children: Vec<StyledNode>,
}

fn specificity(selector: &Selector) -> Specificity {
    match selector {
        Selector::Id(_)    => (1, 0, 0),
        Selector::Class(_) => (0, 1, 0),
        Selector::Type(_)  => (0, 0, 1),
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
