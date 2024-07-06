use std::collections::HashMap;

use once_cell::sync::Lazy;
use scraper::ElementRef;
use scraper::Node;

/// Converts an element to plain text, removing tags like '\<strong\>' while keeping the text within those elements
pub fn element_to_plain_text(element: &ElementRef) -> String {
    let mut plain_text = String::new();
    for node in element.children() {
        match node.value() {
            Node::Text(text) => {
                plain_text.push_str(text.trim_matches('\n'));
            }
            Node::Element(element) => match element.name() {
                "br" => plain_text.push('\n'),
                _ => {
                    let elmt_ref = ElementRef::wrap(node).expect("Node of value Element will always wrap to ElementRef");
                    plain_text.push_str(&element_to_plain_text(&elmt_ref))
                }
            },
            _ => {}
        }
    }
    // For some reason, the nodes start with large blocks of whitespace.
    plain_text.trim().to_string()
}

/// HashMap of expected html entities with their replacement character
static HTML_ENTITIES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert("&nbsp;", " ");
    map.insert("&amp;", "&");
    map.insert("&lt;", "<");
    map.insert("&gt;", ">");
    map.insert("&quot;", "\"");
    map.insert("&#39;", "'");
    map.insert("&apos;", "'");
    map
});

/// Use when getting the inner text of an html element
pub fn replace_entities(mut value: String) -> String {
    for (entity, target) in HTML_ENTITIES.iter() {
        if value.contains(entity) {
            value = value.replace(entity, target)
        }
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Html;

    #[test]
    fn element_to_plain_text_works() {
        let html = Html::parse_fragment(r#"<p>This is a <strong>test</strong> with some <br>extra&nbsp;stuff</p>"#);
        assert_eq!(
            element_to_plain_text(&html.root_element()),
            "This is a test with some \nextra\u{a0}stuff"
        );
    }

    #[test]
    fn element_to_plain_text_works_on_real() {
        let html = Html::parse_fragment(
            r#"
                <p>After entering a boat, Jesus made the crossing, and came into his own town.<br>
And there people brought to him a paralytic lying on a stretcher.<br>
When Jesus saw their faith, he said to the paralytic,<br>
"Courage, child, your sins are forgiven."<br>
At that, some of the scribes said to themselves,<br>
"This man is blaspheming."<br>
Jesus knew what they were thinking, and said,<br>
"Why do you harbor evil thoughts?<br>
Which is easier, to say, 'Your sins are forgiven,'<br>
or to say, 'Rise and walk'?<br>
But that you may know that the Son of Man<br>
has authority on earth to forgive sins"–<br>
he then said to the paralytic,<br>
"Rise, pick up your stretcher, and go home."<br>
He rose and went home.<br>
When the crowds saw this they were struck with awe<br>
and glorified God who had given such authority to men.</p>

<p>&nbsp;</p>"#,
        );
        assert_eq!(
            element_to_plain_text(&html.root_element()),
            r#"After entering a boat, Jesus made the crossing, and came into his own town.
And there people brought to him a paralytic lying on a stretcher.
When Jesus saw their faith, he said to the paralytic,
"Courage, child, your sins are forgiven."
At that, some of the scribes said to themselves,
"This man is blaspheming."
Jesus knew what they were thinking, and said,
"Why do you harbor evil thoughts?
Which is easier, to say, 'Your sins are forgiven,'
or to say, 'Rise and walk'?
But that you may know that the Son of Man
has authority on earth to forgive sins"–
he then said to the paralytic,
"Rise, pick up your stretcher, and go home."
He rose and went home.
When the crowds saw this they were struck with awe
and glorified God who had given such authority to men."#
        );
    }
}
