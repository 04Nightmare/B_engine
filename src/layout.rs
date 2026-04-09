use crate::css::style_tree::StyledNode;
use crate::dom::NodeType;

#[derive(Debug, Clone, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Default)]
pub struct EdgeSize {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

#[derive(Debug, Clone, Default)]
pub struct Dimensions {
    pub content: Rect,
    pub padding: EdgeSize,
    pub border: EdgeSize,
    pub margin: EdgeSize,
}

impl Dimensions {
    pub fn padding_box(&self) -> Rect {
        let x = self.content.x - self.padding.left;
        let y = self.content.y - self.padding.right;
        let width = self.content.width + self.padding.left + self.padding.right;
        let height = self.content.height + self.padding.top + self.padding.bottom;

        return Rect {
            x,
            y,
            width,
            height,
        };
    }

    pub fn border_box(&self) -> Rect {
        let pb = self.padding_box();
        let x = pb.x - self.border.left;
        let y = pb.y - self.border.right;
        let width = pb.width + self.border.left + self.border.right;
        let height = pb.height + self.border.top + self.border.bottom;

        return Rect {
            x,
            y,
            width,
            height,
        };
    }

    pub fn margin_box(&self) -> Rect {
        let bb = self.border_box();
        let x = bb.x - self.margin.left;
        let y = bb.y - self.margin.top;
        let width = bb.width + self.margin.left + self.margin.right;
        let height = bb.height + self.margin.top + self.margin.bottom;

        return Rect {
            x,
            y,
            width,
            height,
        };
    }
}

//Box Type
#[derive(Debug, Clone, PartialEq)]
pub enum BoxType {
    Block,
    Inline,
    Anonymous,
}

pub struct LayoutBox<'a> {
    pub dimensions: Dimensions,
    pub box_type: BoxType,
    pub children: Vec<LayoutBox<'a>>,
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
        let value = self
            .styled_node
            .and_then(|n| n.specified_values.get(name))
            .map(|s| s.as_str());

        return value;
    }

    fn px(&self, name: &str) -> f32 {
        let px = self
            .value(name)
            //.and_then(|v| v.strip_suffix("px")?.trim().parse().ok())  //directly parsed the px.
            .and_then(|v| parse_px(v))
            .unwrap_or(0.0);

        return px;
    }

    fn get_or_create_anonymous(&mut self) -> &mut LayoutBox<'a> {
        if self
            .children
            .last()
            .map(|c| c.box_type == BoxType::Anonymous)
            != Some(true)
        {
            self.children.push(LayoutBox::new(BoxType::Anonymous, None));
        }
        return self.children.last_mut().unwrap();
    }
}

//CSS helper
fn parse_px(value: &str) -> Option<f32> {
    value.trim().strip_suffix("px")?.trim().parse().ok()
}

fn display(node: &StyledNode) -> Option<BoxType> {
    if let Some(val) = node.specified_values.get("display") {
        return match val.as_str() {
            "block" => Some(BoxType::Block),
            "inline" => Some(BoxType::Inline),
            "none" => None,
            _ => Some(BoxType::Inline),
        };
    }

    match &node.node.borrow().node_type {
        NodeType::Text(_) => Some(BoxType::Inline),
        NodeType::Element(e) => match e.tag_name.as_str() {
            "div" | "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "body" | "html" | "ul"
            | "ol" | "li" | "section" | "article" | "header" | "footer" | "main" | "document" => {
                Some(BoxType::Block)
            }

            "head" | "script" | "style" => None,

            _ => Some(BoxType::Inline),
        },
    }
}

pub fn build_layout_tree<'a>(styled: &'a StyledNode) -> Option<LayoutBox<'a>> {
    let box_type = display(styled)?;
    let mut layout_box = LayoutBox::new(box_type.clone(), Some(styled));

    for child in &styled.children {
        let child_box_type = match display(child) {
            Some(t) => t,
            None => continue,
        };

        match child_box_type {
            BoxType::Block => {
                if let Some(child_layout) = build_layout_tree(child) {
                    layout_box.children.push(child_layout);
                }
            }

            BoxType::Inline | BoxType::Anonymous => {
                if let Some(child_layout) = build_layout_tree(child) {
                    layout_box
                        .get_or_create_anonymous()
                        .children
                        .push(child_layout);
                }
            }
        }
    }
    return Some(layout_box);
}

fn layout_block(layout_box: &mut LayoutBox, containing: &Dimensions) {
    // Step 1: width is resolved from the containing block's width.
    calculate_block_width(layout_box, containing);

    // Step 2: position (x, y) is set based on containing block + sibling heights.
    calculate_block_position(layout_box, containing);

    // Step 3: recurse — lay out children inside this box.
    layout_block_children(layout_box);

    // Step 4: height is either explicit or the sum of children.
    calculate_block_height(layout_box);
}

//Position
fn calculate_block_width(layout_box: &mut LayoutBox, containing: &Dimensions) {
    let container_width = containing.content.width;

    let margin_left = layout_box.px("margin-left");
    let margin_right = layout_box.px("margin-right");
    let border_left = layout_box.px("border-left");
    let border_right = layout_box.px("border-right");
    let padding_left = layout_box.px("padding-left");
    let padding_right = layout_box.px("padding-right");
    println!(
        "[DEBUG] <tag> bl={border_left} br={border_right} pl={padding_left} pr={padding_right}"
    );

    let width_auto = layout_box.value("width").map_or(true, |v| v == "auto");
    let width = if width_auto {
        (container_width
            - margin_left
            - margin_right
            - border_left
            - border_right
            - padding_left
            - padding_right)
            .max(0.0)
    } else {
        layout_box.px("width")
    };

    layout_box.dimensions.content.width = width;
    layout_box.dimensions.margin.left = margin_left;
    layout_box.dimensions.margin.right = margin_right;
    layout_box.dimensions.border.left = border_left;
    layout_box.dimensions.border.right = border_right;
    layout_box.dimensions.padding.left = padding_left;
    layout_box.dimensions.padding.right = padding_right;
}

fn calculate_block_position(layout_box: &mut LayoutBox, containing: &Dimensions) {
    let margin_top = layout_box.px("margin-top");
    let margin_bottom = layout_box.px("margin-bottom");
    let border_top = layout_box.px("border-top");
    let border_bottom = layout_box.px("border-bottom");
    let padding_top = layout_box.px("padding-top");
    let padding_bottom = layout_box.px("padding-bottom");

    layout_box.dimensions.margin.top = margin_top;
    layout_box.dimensions.margin.bottom = margin_bottom;
    layout_box.dimensions.border.top = border_top;
    layout_box.dimensions.border.bottom = border_bottom;
    layout_box.dimensions.padding.top = padding_top;
    layout_box.dimensions.padding.bottom = padding_bottom;

    layout_box.dimensions.content.x = containing.content.x
        + layout_box.dimensions.margin.left
        + layout_box.dimensions.border.left
        + layout_box.dimensions.padding.left;

    layout_box.dimensions.content.y = containing.content.y
        + containing.content.height
        + layout_box.dimensions.margin.top
        + layout_box.dimensions.border.top
        + layout_box.dimensions.padding.top;
}

//Height
fn calculate_block_height(layout_box: &mut LayoutBox) {
    if let Some(h) = layout_box.value("height").and_then(parse_px) {
        layout_box.dimensions.content.height = h;
    }
}

//Children
fn layout_block_children(layout_box: &mut LayoutBox) {
    let mut containing_snapshot = layout_box.dimensions.clone();

    for child in &mut layout_box.children {
        layout(child, &containing_snapshot);
        containing_snapshot.content.height += child.dimensions.margin_box().height;
    }

    layout_box.dimensions.content.height = containing_snapshot.content.height;
}

pub fn layout<'a>(layout_box: &mut LayoutBox<'a>, containing: &Dimensions) {
    match layout_box.box_type {
        BoxType::Block | BoxType::Anonymous => layout_block(layout_box, containing),
        BoxType::Inline => {}
    }
}

//Printing
pub fn print_layout_tree(node: &LayoutBox, indent: usize) {
    let padding = " ".repeat(indent);
    let d = &node.dimensions;

    let label = match node.box_type {
        BoxType::Block => "Block",
        BoxType::Inline => "Inline",
        BoxType::Anonymous => "Anonymous",
    };

    let tag = node
        .styled_node
        .map(|sn| match &sn.node.borrow().node_type {
            NodeType::Element(e) => format!("<{}>", e.tag_name),
            NodeType::Text(t) => format!("\"{}\"", t.trim()),
        })
        .unwrap_or_else(|| "(anon)".to_string());

    println!(
        "{}{} {}   →   x:{:.1} y:{:.1} w:{:.1} h:{:.1}    [margin t:{:.1} r:{:.1} b:{:.1} l:{:.1}]",
        padding,
        label,
        tag,
        d.content.x,
        d.content.y,
        d.content.width,
        d.content.height,
        d.margin.top,
        d.margin.right,
        d.margin.bottom,
        d.margin.left,
    );

    for child in &node.children {
        print_layout_tree(child, indent + 2);
    }
}
