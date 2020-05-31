use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::{HashMap, HashSet, LinkedList};

#[derive(Debug, PartialEq, Clone)]
struct Node {
    name: Vec<u8>,
    attributes: HashMap<Vec<u8>, Vec<u8>>,
    sub: Vec<Box<Node>>,
    value: Option<Vec<u8>>,
}

impl Node {
    fn partial_match(&self, node: &Node) -> bool {
        if self.name != node.name {
            return false;
        }

        match (&self.value, &node.value) {
            (Some(v), Some(vv)) if v != vv => return false,
            _ => (),
        }

        for (k, v) in node.attributes.iter() {
            match self.attributes.get(k) {
                Some(vv) if v != vv => return false,
                None => return false,
                _ => continue,
            }
        }

        let mut matched = HashSet::new();
        for n in node.sub.iter() {
            let mut find_match = false;

            for (i, nn) in self.sub.iter().enumerate() {
                if matched.contains(&i) {
                    continue;
                }
                if nn.partial_match(n) {
                    find_match = true;
                    matched.insert(i);
                    break;
                }
            }

            if !find_match {
                return false;
            }
        }

        return true;
    }

    fn parse<B>(mut reader: Reader<B>) -> Vec<Box<Node>>
    where
        B: std::io::BufRead,
    {
        let mut top: Vec<Box<Node>> = vec![];
        let mut buf = Vec::new();
        let mut crumbs: LinkedList<Box<Node>> = LinkedList::new();
        loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let mut m = HashMap::new();
                    for r in e.attributes() {
                        let attr = r.unwrap();
                        m.insert(attr.key.to_vec(), attr.value.to_vec());
                    }

                    let nn = Box::new(Node {
                        name: Vec::from(e.name()),
                        attributes: m,
                        sub: vec![],
                        value: None,
                    });

                    crumbs.push_front(nn)
                }
                Ok(Event::Text(bs)) => match crumbs.front_mut() {
                    Some(n) => n.value = Some(bs.to_vec()),
                    None => (),
                },
                Ok(Event::End(_)) => match (crumbs.pop_front(), crumbs.front_mut()) {
                    (Some(child), Some(parent)) => parent.sub.push(child),
                    (Some(child), None) => top.push(child),
                    _ => break, //TODO return Result error case
                },
                Err(e) => break, //TODO return Result error case
                Ok(Event::Eof) => break,
                _ => (),
            }

            buf.clear();
        }
        top
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use quick_xml::Reader;

    #[test]
    fn test_parsing_xml() {
        let xml = r#"<tag1 att1 = "test">
                        <tag2><!--Test comment-->Test</tag2>
                        <tag2>
                            Test 2
                        </tag2>
                    </tag1>"#;

        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);

        let tested = Node::parse(reader);

        let tag2_1 = Box::new(Node {
            name: b"tag2".to_vec(),
            attributes: HashMap::new(),
            sub: vec![],
            value: Some(b"Test".to_vec()),
        });

        let tag2_2 = Box::new(Node {
            name: b"tag2".to_vec(),
            attributes: HashMap::new(),
            sub: vec![],
            value: Some(b"Test 2".to_vec()),
        });

        let mut attrs_tag1 = HashMap::new();
        attrs_tag1.insert(b"att1".to_vec(), b"test".to_vec());

        let tag1 = Box::new(Node {
            name: b"tag1".to_vec(),
            attributes: attrs_tag1,
            sub: vec![tag2_1, tag2_2],
            value: None,
        });

        assert_eq!(vec![tag1], tested);
    }
}
