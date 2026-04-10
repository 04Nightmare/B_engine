# Browser Engine

A simple browser engine written in Rust, built from scratch to understand how real browsers work under the hood. It takes raw HTML and CSS as input and produces a rendered PNG image as output — no external rendering libraries involved.

---

## How it works

The engine implements a simplified but faithful version of the browser rendering pipeline:

```
HTML + CSS  →  Tokens  →  DOM  →  Styled Tree  →  Layout Tree  →  Display List  →  PNG
```

**1. Tokenizer**
The HTML input is scanned character by character and converted into a flat list of typed tokens — start tags (with attributes), end tags, and text nodes.

**2. DOM Builder**
Tokens are consumed using a stack-based algorithm to build a tree of `Node` objects, mirroring the structure of the HTML. Each node is either an `Element` (with a tag name and attribute map) or a `Text` node.

**3. CSS Parser**
CSS rules are parsed into a `Stylesheet` — a list of rules, each containing a list of selectors (`Type`, `Class`, `Id`) and declarations (`property: value` pairs).

**4. Style Tree**
The DOM tree is walked recursively. For each element, every CSS rule whose selector matches is collected, sorted by specificity (`id > class > type`), and cascaded into a `PropertyMap` of resolved values. Each node in the resulting style tree carries its computed CSS properties.

**5. Layout Tree**
The styled tree is converted into a layout tree of `LayoutBox` nodes. Each box is typed as `Block`, `Inline`, or `Anonymous`. Anonymous boxes are automatically inserted to wrap inline content inside block parents, keeping the layout algorithm clean. For each block box the engine computes:
- **Width** — from the containing block's width, respecting explicit values and `auto`
- **Position** — `x` and `y` coordinates derived from parent position and accumulated sibling height
- **Height** — either an explicit CSS value or the sum of laid-out children

The full CSS box model is implemented: `content`, `padding`, `border`, and `margin` areas are all tracked separately, with helpers for `padding_box()`, `border_box()`, and `margin_box()`.

**6. Painting**
The layout tree is converted into a `DisplayList` — an ordered sequence of `SolidRect` draw commands (backgrounds first, then borders). This two-phase design separates geometry from rasterization. The rasterizer then executes each command against a flat RGBA pixel buffer (`Canvas`), which is saved to disk as a PNG using the `image` crate.

---

## Project structure

```
src/
├── main.rs          # Pipeline wiring, HTML/CSS input, tokenizer, DOM builder
├── dom.rs           # Node, NodeType, ElementData — the DOM tree types
├── css/
│   ├── mod.rs
│   └── stylesheet.rs  # Stylesheet, Rule, Selector, Declaration — CSS parser
├── style.rs         # Specificity, cascade, StyledNode — style tree builder
├── layout.rs        # Dimensions, BoxType, LayoutBox — layout tree + box model
└── paint.rs      # Color, DisplayCommand, Canvas — display list + rasterizer
```
---

## Dependencies

```toml
[dependencies]
image = "0.25"   # PNG encoding
```
---
## Status
 
This is a learning project. Current status:
 
| Feature | Status |
|---|---|
| HTML tokenizer and DOM builder | ✅ |
| CSS selector matching (type, class, id) | ✅ |
| Cascade and specificity | ✅ |
| Block layout and box model | ✅ |
| Background and border painting | ✅ |
| PNG output | ✅ |
| Inline / text layout | 🔜 next |
| Text rendering | 🔜 next |
| Flexbox / grid | 🔜 next  |
| JavaScript | 🔜 next  |
 
---
## 🛠️ Installation
### Clone the repository
```
git clone (https://github.com/04Nightmare/B_engine.git)
cd B_engine
```
### Compile Program
```
cargo build
cargo run
```
Open 'output.png' for the rendered result.
