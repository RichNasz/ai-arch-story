use anyhow::{bail, Result};
use std::collections::HashSet;

use super::types::Diagram;

pub fn validate_diagram(diagram: &Diagram) -> Result<()> {
    validate_version(diagram)?;
    validate_unique_ids(diagram)?;
    validate_edge_references(diagram)?;
    validate_flow_references(diagram)?;
    validate_group_references(diagram)?;
    validate_flow_paths(diagram)?;
    Ok(())
}

fn validate_version(diagram: &Diagram) -> Result<()> {
    if diagram.version != "1.0" {
        bail!("Unsupported schema version: {}. Expected \"1.0\"", diagram.version);
    }
    Ok(())
}

fn validate_unique_ids(diagram: &Diagram) -> Result<()> {
    let mut node_ids = HashSet::new();
    for node in &diagram.nodes {
        if !node_ids.insert(&node.id) {
            bail!("Duplicate node id: \"{}\"", node.id);
        }
    }

    let mut edge_ids = HashSet::new();
    for edge in &diagram.edges {
        if !edge_ids.insert(&edge.id) {
            bail!("Duplicate edge id: \"{}\"", edge.id);
        }
    }

    let mut flow_ids = HashSet::new();
    for flow in &diagram.flows {
        if !flow_ids.insert(&flow.id) {
            bail!("Duplicate flow id: \"{}\"", flow.id);
        }
    }

    let mut group_ids = HashSet::new();
    for group in &diagram.groups {
        if !group_ids.insert(&group.id) {
            bail!("Duplicate group id: \"{}\"", group.id);
        }
    }

    Ok(())
}

fn validate_edge_references(diagram: &Diagram) -> Result<()> {
    let node_ids: HashSet<&str> = diagram.nodes.iter().map(|n| n.id.as_str()).collect();

    for edge in &diagram.edges {
        if !node_ids.contains(edge.from.as_str()) {
            bail!("Edge \"{}\" references unknown source node: \"{}\"", edge.id, edge.from);
        }
        if !node_ids.contains(edge.to.as_str()) {
            bail!("Edge \"{}\" references unknown target node: \"{}\"", edge.id, edge.to);
        }
    }

    Ok(())
}

fn validate_flow_references(diagram: &Diagram) -> Result<()> {
    let edge_ids: HashSet<&str> = diagram.edges.iter().map(|e| e.id.as_str()).collect();

    for flow in &diagram.flows {
        if flow.steps.is_empty() {
            bail!("Flow \"{}\" has no steps", flow.id);
        }
        for step in &flow.steps {
            if !edge_ids.contains(step.edge.as_str()) {
                bail!(
                    "Flow \"{}\" step references unknown edge: \"{}\"",
                    flow.id,
                    step.edge
                );
            }
        }
    }

    Ok(())
}

fn validate_group_references(diagram: &Diagram) -> Result<()> {
    let node_ids: HashSet<&str> = diagram.nodes.iter().map(|n| n.id.as_str()).collect();
    let group_ids: HashSet<&str> = diagram.groups.iter().map(|g| g.id.as_str()).collect();

    for group in &diagram.groups {
        for node_id in &group.nodes {
            if !node_ids.contains(node_id.as_str()) {
                bail!(
                    "Group \"{}\" references unknown node: \"{}\"",
                    group.id,
                    node_id
                );
            }
        }
        for sub_group_id in &group.groups {
            if !group_ids.contains(sub_group_id.as_str()) {
                bail!(
                    "Group \"{}\" references unknown sub-group: \"{}\"",
                    group.id,
                    sub_group_id
                );
            }
            if sub_group_id == &group.id {
                bail!("Group \"{}\" references itself as a sub-group", group.id);
            }
        }
    }

    Ok(())
}

fn validate_flow_paths(diagram: &Diagram) -> Result<()> {
    use std::collections::HashMap;

    let edge_map: HashMap<&str, (&str, &str)> = diagram
        .edges
        .iter()
        .map(|e| (e.id.as_str(), (e.from.as_str(), e.to.as_str())))
        .collect();

    for flow in &diagram.flows {
        let mut current_node: Option<&str> = None;

        for (i, step) in flow.steps.iter().enumerate() {
            let (from, to) = edge_map[step.edge.as_str()];

            if step.parallel.unwrap_or(false) {
                if let Some(curr) = current_node {
                    if from != curr {
                        bail!(
                            "Flow \"{}\" parallel step {} edge \"{}\" starts at \"{}\" but current position is \"{}\"",
                            flow.id, i, step.edge, from, curr
                        );
                    }
                }
                // parallel steps don't advance current_node
            } else {
                if let Some(curr) = current_node {
                    if from != curr {
                        bail!(
                            "Flow \"{}\" step {} edge \"{}\" starts at \"{}\" but previous step ended at \"{}\"",
                            flow.id, i, step.edge, from, curr
                        );
                    }
                }
                current_node = Some(to);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::types::*;

    fn minimal_diagram() -> Diagram {
        Diagram {
            version: "1.0".to_string(),
            title: "Test".to_string(),
            description: None,
            theme: "default".to_string(),
            nodes: vec![
                Node {
                    id: "a".to_string(),
                    label: "A".to_string(),
                    node_type: None,
                    icon: None,
                    style: None,
                    position: None,
                    metadata: Default::default(),
                },
                Node {
                    id: "b".to_string(),
                    label: "B".to_string(),
                    node_type: None,
                    icon: None,
                    style: None,
                    position: None,
                    metadata: Default::default(),
                },
            ],
            edges: vec![Edge {
                id: "e1".to_string(),
                from: "a".to_string(),
                to: "b".to_string(),
                label: None,
                direction: None,
                style: None,
                metadata: Default::default(),
            }],
            flows: vec![],
            groups: vec![],
            viewport: None,
            branding: None,
            custom_types: None,
            metadata: Default::default(),
        }
    }

    #[test]
    fn valid_minimal_diagram() {
        assert!(validate_diagram(&minimal_diagram()).is_ok());
    }

    #[test]
    fn rejects_bad_version() {
        let mut d = minimal_diagram();
        d.version = "2.0".to_string();
        assert!(validate_diagram(&d).is_err());
    }

    #[test]
    fn rejects_duplicate_node_ids() {
        let mut d = minimal_diagram();
        d.nodes[1].id = "a".to_string();
        assert!(validate_diagram(&d).is_err());
    }

    #[test]
    fn rejects_edge_to_unknown_node() {
        let mut d = minimal_diagram();
        d.edges[0].to = "nonexistent".to_string();
        assert!(validate_diagram(&d).is_err());
    }

    #[test]
    fn rejects_flow_with_unknown_edge() {
        let mut d = minimal_diagram();
        d.flows.push(Flow {
            id: "f1".to_string(),
            label: "Flow".to_string(),
            description: None,
            steps: vec![FlowStep {
                edge: "nonexistent".to_string(),
                label: None,
                description: None,
                parallel: None,
            }],
            style: None,
            metadata: Default::default(),
        });
        assert!(validate_diagram(&d).is_err());
    }
}
