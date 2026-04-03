use crate::layout::{BoxType, LayoutBox, Rect};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color{
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color{
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        return Color { r, g, b, a };
    }
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        return Color::rgba(r, g, b, 255);
    }
    pub const TRANSPARENT: Color = Color::rgba(0, 0, 0, 0);
    pub const WHITE: Color = Color::rgb(255, 255, 255);
    pub const BLACK: Color = Color::rgb(0, 0, 0);
}

pub fn parse_color(value: &str) -> Option<Color> {
    let v = value.trim();

    //rgb(r, g, b)
    if let Some(inner) = v.strip_prefix("rgb(").and_then(|s| s.strip_suffix(')')) {
        let parts: Vec<u8> = inner
            .split(',')
            .filter_map(|p| p.trim().parse().ok())
            .collect();
        if parts.len() == 3 {
            return Some(Color::rgb(parts[0], parts[1], parts[2]));
        }
    }

    // #rrggbb / #rgb
    if let Some(hex) = v.strip_prefix('#') {
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                return Some(Color::rgb(r, g, b));
            }
            3 => {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
                return Some(Color::rgb(r, g, b));
            }
            _ => {}
        }
    }

    Some(match v {
        "black" => Color::rgb(0,0, 0),
        "white" => Color::rgb(255, 255, 255),
        "red" => Color::rgb(255, 0, 0),
        "green" => Color::rgb(0, 128, 0),
        "blue" => Color::rgb(0, 0, 255),
        "yellow" => Color::rgb(255, 255, 0),
        "orange" => Color::rgb(255, 165, 0),
        "purple" => Color::rgb(128, 0, 128),
        "pink" => Color::rgb(255, 192, 203),
        "gray" => Color::rgb(128, 128, 128),
        "grey" => Color::rgb(128, 128, 128),
        "silver" => Color::rgb(192, 192, 192),
        "cyan" => Color::rgb(0, 255, 255),
        "magenta" => Color::rgb(255, 0, 255),
        "brown" => Color::rgb(165, 42, 42 ),
        "navy" => Color::rgb(0, 0, 128),
        "teal" => Color::rgb(0, 128, 128),
        "lime" => Color::rgb(0, 255, 0),
        _ => return None,
    })
}


fn get_color(layout_box: &LayoutBox, property: &str) -> Option<Color> {
    // Anonymous boxes have no styled node, so they never have a color.
    if layout_box.box_type == BoxType::Anonymous {
        return None;
    }
    layout_box
        .styled_node?
        .specified_values
        .get(property)
        .and_then(|v| parse_color(v))
}


//Display List
#[derive(Debug, Clone)]
pub enum DisplayCommand {
    SolidRect { color: Color, rect: Rect },
}

pub type DisplayList = Vec<DisplayCommand>;

pub fn build_display_list(layout_root: &LayoutBox) -> DisplayList {
    let mut list = Vec::new();
    emit_commands(layout_root, &mut list);
    list
}
 
fn emit_commands(layout_box: &LayoutBox, list: &mut DisplayList) {

    emit_background(layout_box, list);
 
    emit_borders(layout_box, list);
 
    for child in &layout_box.children {
        emit_commands(child, list);
    }
}

//Background
fn emit_background(layout_box: &LayoutBox, list: &mut DisplayList) {
    if let Some(color) = get_color(layout_box, "background-color")
        .or_else(|| get_color(layout_box, "background"))
    {
        list.push(DisplayCommand::SolidRect {
            color,
            rect: layout_box.dimensions.padding_box(),
        });
    }
}

//Borders
fn emit_borders(layout_box: &LayoutBox, list: &mut DisplayList) {
    let color = match get_color(layout_box, "border-color") {
        Some(c) => c,
        None    => return,
    };
 
    let d  = &layout_box.dimensions;
    let border_bx = d.border_box();
    let padding_bx = d.padding_box();
 
    // Top border
    if d.border.top > 0.0 {
        list.push(DisplayCommand::SolidRect {
            color,
            rect: Rect {
                x: border_bx.x,
                y: border_bx.y,
                width: border_bx.width, 
                height: d.border.top
            },
        });
    }
    // Bottom border
    if d.border.bottom > 0.0 {
        list.push(DisplayCommand::SolidRect {
            color,
            rect: Rect {
                x: border_bx.x,
                y: padding_bx.y + padding_bx.height,
                width: border_bx.width,
                height: d.border.bottom,
            },
        });
    }
    // Left border
    if d.border.left > 0.0 {
        list.push(DisplayCommand::SolidRect {
            color,
            rect: Rect {
                x: border_bx.x,
                y: border_bx.y,
                width: d.border.left,
                height: border_bx.height,
            },
        });
    }
    // Right border
    if d.border.right > 0.0 {
        list.push(DisplayCommand::SolidRect {
            color,
            rect: Rect {
                x: padding_bx.x + padding_bx.width,
                y: border_bx.y,
                width: d.border.right,
                height: border_bx.height,
            },
        });
    }
}