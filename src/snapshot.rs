use std::collections::HashSet;

use toy_ufdb::Ufdb;

pub fn render(ufdb: &mut Ufdb) -> String {
    let mut groups: Vec<Vec<String>> = ufdb
        .groups()
        .into_values()
        .map(|members| members.iter().map(|s| s.to_string()).collect())
        .collect();

    for group in &mut groups {
        group.sort();
    }

    groups.sort_by(|a, b| b.len().cmp(&a.len()));

    let body: String = groups
        .iter()
        .map(|members| render_group(ufdb, members))
        .collect();

    format!("<!DOCTYPE html><html><body>{body}</body></html>")
}

fn render_group(ufdb: &Ufdb, members: &[String]) -> String {
    let root = &members[0];

    let mut visited = HashSet::new();
    visited.insert(root.clone());

    let tree = render_node(ufdb, root, &mut visited);

    format!(
        "<details><summary>Group (size {})</summary><ul>{tree}</ul></details>",
        members.len()
    )
}

fn render_node(ufdb: &Ufdb, key: &str, visited: &mut HashSet<String>) -> String {
    let mut children: Vec<String> = ufdb
        .neighbors(key)
        .map(|neighbors| {
            neighbors
                .iter()
                .filter(|n| !visited.contains(*n))
                .cloned()
                .collect()
        })
        .unwrap_or_default();

    children.sort();

    if children.is_empty() {
        return format!("<li>{key}</li>");
    }

    for child in &children {
        visited.insert(child.clone());
    }

    let children_html: String = children
        .iter()
        .map(|child| render_node(ufdb, child, visited))
        .collect();

    format!("<li>{key}<ul>{children_html}</ul></li>")
}
