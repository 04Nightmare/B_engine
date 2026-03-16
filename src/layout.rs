use crate::dom::NodeType;
use crate::css::style_tree::StyledNode;

#[derive(Debug, Clone, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Default)]
pub struct EdgeSize{
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

#[derive(Debug, Clone, Default)]
pub struct Dimensions{
    pub content: Rect,
    pub padding: EdgeSize,
    pub border: EdgeSize,
    pub margin: EdgeSize,
}

impl Dimensions{
    pub fn padding_box(&self) -> Rect {
        let x = self.content.x - self.padding.left;
        let y = self.content.y - self.padding.right;
        let width = self.content.width + self.padding.left + self.padding.right;
        let height = self.content.height + self.padding.top + self.padding.bottom;

        return Rect{x, y, width, height};
    }

    pub fn border_box(&self) -> Rect {
        let pb = self.padding_box();
        let x = pb.x - self.border.left;
        let y = pb.y - self.border.right;
        let width = pb.width + self.border.left + self.border.right;
        let height = pb.height + self.border.top + self.border.bottom;

        return Rect {x, y, width, height};
    }

    pub fn margin_box(&self) -> Rect {
        let bb = self.border_box();
        let x = bb.x - self.margin.left;
        let y = bb.y - self.margin.top;
        let width = bb.width  + self.margin.left + self.margin.right;
        let height = bb.height + self.margin.top  + self.margin.bottom;
        
        return Rect {x, y, width, height};
    }
}


//Box Type
#[derive(Debug, Clone)]
pub enum BoxType {
    Block,
    Inline,
    Anonymous,
}

pub struct LayoutBox<'a> {
    pub dimensions: Dimensions,
    pub box_type:   BoxType,
    pub children:   Vec<LayoutBox<'a>>,
    pub styled_node: Option<&'a StyledNode>,
}

impl<'a> LayoutBox<'a> {
    fn new(box_type: BoxType, styled_node: Option<&'a StyledNode>) -> Self {
        return LayoutBox {
            dimensions: Dimensions::default(),
            box_type,
            children: Vec::new(),
            styled_node,
        };
    }

    fn value(&self, name: &str) -> Option<&str> {
        let value = self.styled_node
            .and_then(|n| n.specified_values.get(name))
            .map(|s| s.as_str());

        return value;
    }

    fn px(&self, name: &str) -> f32 {
        let px = self.value(name)
            .and_then(|v| v.strip_suffix("px")?.trim().parse().ok())  //directly parsed the px.
            .unwrap_or(0.0);

        return px;
    }

    fn get_or_create_anonymous(&mut self) -> &mut LayoutBox<'a> {
        if self.children.last().map(|c| c.box_type == BoxType::Anonymous) != Some(true) {
           self.children.push(LayoutBox::new(BoxType::Anonymous, None));
        }
        return self.children.last_mut().unwrap();
    }
}


