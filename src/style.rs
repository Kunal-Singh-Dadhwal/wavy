use crate::css::{Rule, Selector, SimpleSelector, Specificity, Stylesheet, Value};
use crate::dom::{ElementData, Node, NodeType};
use std::collections::HashMap;

pub type PropertyMap = HashMap<String, Value>;

pub struct StyleNode<'a> {
    pub node: &'a Node,
    pub specified_values: PropertyMap,
    pub children: Vec<StyleNode<'a>>,
}

#[derive(PartialEq)]
pub enum Display {
    Inline,
    Block,
    None,
}

impl<'a> StyleNode<'a> {
    pub fn value(&self, name: &str) -> Option<Value> {
        self.specified_values.get(name).cloned()
    }

    pub fn lookup(&self, name: &str, fallback_name: &str, default: &Value) -> Value {
        self.value(name)
            .unwrap_or_else(|| self.value(fallback_name).unwrap_or_else(|| default.clone()))
    }
    pub fn display(&self) -> Display {
        match self.value("display") {
            Some(Value::Keyword(s)) => match &*s {
                "block" => Display::Block,
                "none" => Display::None,
                _ => Display::Inline,
            },
            _ => Display::Inline,
        }
    }
}

pub fn style_tree<'a>(root: &'a Node, stylesheet: &'a Stylesheet) -> StyleNode<'a> {
    StyleNode {
        node: root,
        specified_values: match root.node_type {
            NodeType::Element(ref elem) => specified_values(elem, stylesheet),
            NodeType::Text(_) => HashMap::new(),
        },
        children: root
            .children
            .iter()
            .map(|child| style_tree(child, stylesheet))
            .collect(),
    }
}

fn specified_values(elem: &ElementData, stylesheet: &Stylesheet) -> PropertyMap {
    let mut values = HashMap::new();
    let mut rules = matching_rules(elem, stylesheet);

    rules.sort_by(|&(a, _), &(b, _)| a.cmp(&b));
    for (_, rule) in rules {
        for declaration in &rule.declarations {
            values.insert(declaration.name.clone(), declaration.value.clone());
        }
    }
    values
}

type MatchedRule<'a> = (Specificity, &'a Rule);

fn matching_rules<'a>(elem: &ElementData, stylesheet: &'a Stylesheet) -> Vec<MatchedRule<'a>> {
    stylesheet
        .rules
        .iter()
        .filter_map(|rule| match_rule(elem, rule))
        .collect()
}

fn match_rule<'a>(elem: &ElementData, rule: &'a Rule) -> Option<MatchedRule<'a>> {
    rule.selectors
        .iter()
        .find(|selector| matches(elem, selector))
        .map(|selector| (selector.specificity(), rule))
}

fn matches(elem: &ElementData, selector: &Selector) -> bool {
    match selector {
        Selector::Simple(s) => matches_simple_selector(elem, s),
    }
}

fn matches_simple_selector(elem: &ElementData, selector: &SimpleSelector) -> bool {
    if selector.tag_name.iter().any(|name| elem.tag_name != *name) {
        return false;
    }

    if selector.id.iter().any(|id| elem.id() != Some(id)) {
        return false;
    }

    if selector
        .class
        .iter()
        .any(|class| !elem.classes().contains(class.as_str()))
    {
        return false;
    }

    true
}
