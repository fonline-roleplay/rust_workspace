use super::{logic_node::LogicNode, KeyValue, LogicChain, Logical};

impl<'a> std::fmt::Display for LogicChain<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;

        let mut or_section = vec![&self.first];
        let mut multi_and = false;
        let mut flush = |or_section: &[&KeyValue], in_and: bool| -> std::fmt::Result {
            if in_and {
                multi_and = true;
            }
            if or_section.len() > 1 {
                if multi_and {
                    write!(f, "(")?;
                }
                write!(f, "{}", or_section[0])?;
                for &kv in &or_section[1..] {
                    write!(f, " or {}", kv)?;
                }
                if multi_and {
                    write!(f, ")")?;
                }
            } else if or_section.len() > 0 {
                write!(f, "{}", or_section[0])?;
            }
            if in_and {
                write!(f, " and ")?;
            }
            Ok(())
        };

        for (logical, kv) in &self.rest {
            match logical {
                Logical::Or => {
                    or_section.push(kv);
                }
                Logical::And => {
                    flush(&or_section, true)?;
                    or_section.clear();
                    or_section.push(kv);
                }
            }
        }
        flush(&or_section, false)?;
        Ok(())
    }
}
/*
#[derive(Debug)]
pub enum LogicNode<'a> {
    And(Vec<LogicNode<'a>>),
    Or(Vec<LogicNode<'a>>),
    KeyValue(KeyValue<'a>),
}
 */

fn display_logic_vec<'a>(
    nodes: &[LogicNode<'a>],
    f: &mut std::fmt::Formatter<'_>,
    sep: &str,
    root: bool,
) -> std::fmt::Result {
    if !root && nodes.len() != 1 {
        write!(f, "(")?;
    }
    if let Some(first) = nodes.first() {
        display_logic_node(first, f, false)?;
        for rest in nodes.iter().skip(1) {
            write!(f, "{}", sep)?;
            display_logic_node(rest, f, false)?;
        }
    }
    if !root && nodes.len() != 1 {
        write!(f, ")")?;
    }
    Ok(())
}

fn display_logic_node<'a>(
    node: &LogicNode<'a>,
    f: &mut std::fmt::Formatter<'_>,
    root: bool,
) -> std::fmt::Result {
    match node {
        LogicNode::And(nodes) => display_logic_vec(nodes, f, " and ", root),
        LogicNode::Or(nodes) => display_logic_vec(nodes, f, " or ", root),
        LogicNode::KeyValue(node) => write!(f, "{}", node),
    }
}

impl<'a> std::fmt::Display for LogicNode<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        display_logic_node(self, f, true)
    }
}

impl<'a> std::fmt::Display for KeyValue<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;
        write!(f, "{}: {}", self.key, self.value)
    }
}
