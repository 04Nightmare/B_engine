#[derive(Debug)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

#[derive(Debug)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug)]
pub enum Selector {
    Type(String),
    Class(String),
    Id(String),
}

#[derive(Debug)]
pub struct Declaration {
    pub name: String,
    pub value: String,
}

impl Stylesheet {
    pub fn parse_css(css_input: &str) -> Stylesheet {
        let mut rules = Vec::new();
        let block_parts = css_input.split('}');
        for block in block_parts{
            if let Some((selector_part, declaration_part)) = block.split_once("{"){
                let selectors = parse_selectors(selector_part);
                let declarations = parse_declarations(declaration_part);
                rules.push(Rule{
                    selectors,
                    declarations,
                });
            }
        }
        return Stylesheet { rules };
    }
}


fn parse_selectors(selector_input: &str) -> Vec<Selector> {
    let mut selectors_vec = Vec::new();

    for selector in selector_input.split(',') {
        let selector = selector.trim();
        if selector.is_empty() {
            continue;
        }

        let first_char = selector.chars().next().unwrap();
        match first_char {
            '.' => selectors_vec.push(Selector::Class(selector[1..].to_string())),
            '#' => selectors_vec.push(Selector::Id(selector[1..].to_string())),
            _ => selectors_vec.push(Selector::Type(selector.to_string())),
        }
    }

    return selectors_vec;
}

fn parse_declarations(declaration_input: &str) -> Vec<Declaration> {
    let mut declarations_vec = Vec::new();

    for decl in declaration_input.split(';') {
        let decl = decl.trim();
        if decl.is_empty() {
            continue;
        }

        if let Some((name, value)) = decl.split_once(':') {
            declarations_vec.push(Declaration {
                name: name.trim().to_string(),
                value: value.trim().to_string(),
            });
        }
    }

    return declarations_vec;
}
