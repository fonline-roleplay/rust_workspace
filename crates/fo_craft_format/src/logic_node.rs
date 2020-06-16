use super::{KeyValue, LogicChain, Logical};

#[derive(Debug)]
pub enum LogicNode<'a> {
    And(Vec<LogicNode<'a>>),
    Or(Vec<LogicNode<'a>>),
    KeyValue(KeyValue<'a>),
}

impl<'a> LogicChain<'a> {
    pub fn logic_nodes(&self) -> LogicNode<'a> {
        if self.rest.is_empty() {
            LogicNode::KeyValue(self.first.clone())
        } else {
            fn flush_or_section<'a>(or_section: &mut Vec<KeyValue<'a>>) -> Option<LogicNode<'a>> {
                Some(match or_section.len() {
                    1 => LogicNode::KeyValue(or_section.remove(0)),
                    2..=usize::MAX => {
                        LogicNode::Or(or_section.drain(..).map(LogicNode::KeyValue).collect())
                    }
                    _ => return None,
                })
            }
            let mut or_section = vec![self.first.clone()];
            let mut and_section: Vec<LogicNode> = Vec::new();
            for (logical, kv) in &self.rest {
                match logical {
                    Logical::Or => {}
                    Logical::And => {
                        if let Some(node) = flush_or_section(&mut or_section) {
                            and_section.push(node);
                        }
                    }
                }
                or_section.push(kv.clone());
            }
            if and_section.is_empty() {
                flush_or_section(&mut or_section).unwrap()
            } else {
                if let Some(node) = flush_or_section(&mut or_section) {
                    and_section.push(node);
                }
                LogicNode::And(and_section)
            }
        }
    }
}
