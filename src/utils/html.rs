use error::Result;
use failure::err_msg;

use html5ever::serialize::{serialize, SerializeOpts};
use html5ever::rcdom::{RcDom, NodeData, Handle};
use html5ever::driver::{parse_document, ParseOpts};
use html5ever::tendril::TendrilSink;
use html5ever::interface::{QualName, Attribute};

/// Extracts the contents of the `<head>` and `<body>` tags from an HTML document, as well as the
/// classes on the `<body>` tag, if any.
pub fn extract_head_and_body(html: &str, nonce: &str) -> Result<(String, String, String)> {
    let parser = parse_document(RcDom::default(), ParseOpts::default());
    let dom = parser.one(html);

    let (head, body) = extract_from_rcdom(&dom)?;
    let class = extract_class(&body);

    if !nonce.is_empty() {
        add_nonces(&head, nonce);
        add_nonces(&body, nonce);
    }

    Ok((stringify(head), stringify(body), class))
}

fn extract_from_rcdom(dom: &RcDom) -> Result<(Handle, Handle)> {
    let mut worklist = vec![dom.document.clone()];
    let (mut head, mut body) = (None, None);

    while let Some(handle) = worklist.pop() {
        match handle.data {
            NodeData::Element { ref name, .. } => match name.local.as_ref() {
                "head" => {
                    if head.is_some() {
                        return Err(err_msg("duplicate <head> tag"));
                    } else {
                        head = Some(handle.clone());
                    }
                }
                "body" => {
                    if body.is_some() {
                        return Err(err_msg("duplicate <body> tag"));
                    } else {
                        body = Some(handle.clone());
                    }
                }
                _ => {}  // do nothing
            }
            _ => {}  // do nothing
        }

        worklist.extend(handle.children.borrow().iter().cloned());
    }

    let head = head.ok_or_else(|| err_msg("couldn't find <head> tag in rustdoc output"))?;
    let body = body.ok_or_else(|| err_msg("couldn't find <body> tag in rustdoc output"))?;
    Ok((head, body))
}

fn stringify(node: Handle) -> String {
    let mut vec = Vec::new();
    serialize(&mut vec, &node, SerializeOpts::default())
        .expect("serializing into buffer failed");

    String::from_utf8(vec).expect("html5ever returned non-utf8 data")
}

fn extract_class(node: &Handle) -> String {
    match node.data {
        NodeData::Element { ref attrs, .. } => {
            let attrs = attrs.borrow();

            attrs.iter()
                 .find(|a| &a.name.local == "class")
                 .map_or(String::new(), |a| a.value.to_string())
        }
        _ => String::new()
    }
}

fn add_nonces(node: &Handle, nonce: &str) {
    if let NodeData::Element { ref name, ref attrs, .. } = node.data {
        if &name.local == "style" || &name.local == "script" {
            let mut attrs = attrs.borrow_mut();

            if !attrs.iter().any(|a| &a.name.local == "src") {
                let name = QualName::new(
                    None,
                    ns!(),
                    local_name!("nonce"));
                let new_attr = Attribute {
                    name,
                    value: nonce.to_string().into(),
                };

                attrs.push(new_attr);
            }
        }
    }

    // recurse on children, but not into "docblocks", i.e. converted user-provided markdown
    let class = extract_class(node);
    if class != "docblock" && class != "docblock-short" {
        for child in node.children.borrow().iter() {
            add_nonces(child, nonce);
        }
    }
}
