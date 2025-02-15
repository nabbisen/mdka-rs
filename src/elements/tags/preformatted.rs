use std::collections::HashMap;

use markup5ever_rcdom::{Handle, NodeData};

use super::super::utils::{
    block_trailing_new_line, element_name, enclose, indent, is_emtpy_element,
};

use super::super::{consts::INDENT_DEFAULT_SIZE, utils::inner_html};

/// code language class prefix
const CODE_LANGUAGE_CLASS_PREFIX: &str = "language-";

/// pre, code
pub fn preformatted_md(
    node: &Handle,
    indent_size: Option<usize>,
    attrs_map: &HashMap<String, String>,
    is_inline: bool,
) -> String {
    if is_inline {
        let content = inner_html(node, indent_size);

        if is_emtpy_element(content.as_str(), attrs_map) {
            return content;
        }
        if content.is_empty() {
            let empty_str = String::new();
            return enclose(empty_str.as_str(), indent_size, attrs_map, false);
        }

        let enclosed = format!(" `{}` ", content);
        return enclose(enclosed.as_str(), indent_size, attrs_map, false);
    }

    let node_children = &node.children.borrow();
    let code_node = node_children.iter().find(|child| {
        let name = element_name(child);
        name == "code"
    });
    let next_node = if code_node.is_some() {
        code_node.unwrap()
    } else {
        node
    };
    let content = inner_html(next_node, indent_size);

    if is_emtpy_element(content.as_str(), attrs_map) {
        return content;
    }
    if content.is_empty() {
        let empty_str = String::new();
        return enclose(empty_str.as_str(), indent_size, attrs_map, false);
    }

    let prefix = if code_node.is_some() {
        let code_lang = match code_node.unwrap().data {
            NodeData::Element { ref attrs, .. } => {
                let vec = attrs.borrow().to_owned();
                let class_list_attr = vec
                    .iter()
                    .find(|attr| attr.name.local.to_string().as_str() == "class");
                match class_list_attr {
                    Some(class_list_attr) => {
                        let class_list_str = class_list_attr.value.to_string();
                        let language_class = class_list_str
                            .split(" ")
                            .find(|class| class.starts_with(CODE_LANGUAGE_CLASS_PREFIX));
                        match language_class {
                            Some(language_class) => language_class
                                .strip_prefix(CODE_LANGUAGE_CLASS_PREFIX)
                                .unwrap()
                                .to_owned(),
                            _ => String::new(),
                        }
                    }
                    _ => vec
                        .iter()
                        .find(|attr| attr.name.local.to_string().as_str() == "lang")
                        .map(|attr| attr.value.to_string())
                        .unwrap_or(String::new()),
                }
            }
            _ => String::new(),
        };
        format!("```{}", code_lang)
    } else {
        "```".to_owned()
    };
    let is_nested = INDENT_DEFAULT_SIZE < indent_size.unwrap();
    let leading = if is_nested {
        block_trailing_new_line(indent_size)
    } else {
        String::new()
    };
    let trailing = block_trailing_new_line(indent_size);
    let indent_str = indent(indent_size);
    let enclosed = format!(
        "{}{}\n{}\n{}```\n{}{}",
        leading, prefix, content, indent_str, indent_str, trailing
    );
    enclose(enclosed.as_str(), indent_size, attrs_map, true)
}
