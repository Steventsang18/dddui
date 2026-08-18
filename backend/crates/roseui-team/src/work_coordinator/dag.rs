//! Workflow DAG — pure validation + topological scheduling (dry-run safe).
//!
//! This module introduces a lightweight *directed acyclic graph* over team
//! work units, closing the "workflow composition layer" gap identified in the
//! product roadmap. It does **not** execute anything: it only validates a
//! [`WorkflowDef`], detects cycles, and produces a topologically-ordered plan
//! that the existing [`super::coordinator`] can later consume as a sequence of
//! `WorkIntent`s.
//!
//! Design note: a workflow node binds to a team slot (`slot_id`) and carries a
//! prompt. Edges (`depends_on`) express "node B may start only after node A
//! completes". The scheduler resolves an execution order that respects every
//! edge. No new runtime state, no changes to the rupoo kernel — purely additive
//! on top of the existing team coordination layer.

use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

/// A single node in a workflow DAG.
///
/// `slot_id` references an existing team member (see [`crate::types::TeamAgent`]);
/// `prompt` is the instruction sent to that member when the node is scheduled.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowNode {
    pub id: String,
    #[serde(default)]
    pub slot_id: String,
    #[serde(default)]
    pub prompt: String,
    /// Node ids that must complete before this node may start.
    #[serde(default)]
    pub depends_on: Vec<String>,
}

/// A workflow definition: a set of nodes plus their dependency edges.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowDef {
    #[serde(default)]
    pub nodes: Vec<WorkflowNode>,
}

/// Result of validating + ordering a [`WorkflowDef`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowPlan {
    /// Node ids in a valid topological order (dependencies first).
    pub order: Vec<String>,
    /// For each node, the set of node ids it transitively depends on.
    pub depth: HashMap<String, usize>,
}

/// Validation error for a [`WorkflowDef`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DagError {
    /// A node id is referenced by `depends_on` but not declared.
    UnknownDependency { node: String, missing: String },
    /// The graph contains a cycle; the contained path lists the cycle members.
    Cycle { path: Vec<String> },
    /// A node has no `id` / empty id.
    EmptyNodeId,
    /// A node references an empty `slot_id` (must bind to a team member).
    UnboundSlot { node: String },
}

impl WorkflowDef {
    /// Validate the definition and compute a topological execution order.
    ///
    /// Pure function: no I/O, no side effects. Returns [`DagError::Cycle`] when
    /// the graph is not a DAG. This is the dry-run gate that must pass before
    /// any node is enqueued into the coordinator.
    pub fn plan(&self) -> Result<WorkflowPlan, DagError> {
        // Index nodes by id; reject duplicates / empties early.
        let mut by_id: HashMap<&str, &WorkflowNode> = HashMap::new();
        for node in &self.nodes {
            if node.id.trim().is_empty() {
                return Err(DagError::EmptyNodeId);
            }
            if node.slot_id.trim().is_empty() {
                return Err(DagError::UnboundSlot { node: node.id.clone() });
            }
            by_id.insert(node.id.as_str(), node);
        }

        // Verify every dependency edge points at a declared node.
        for node in &self.nodes {
            for dep in &node.depends_on {
                if !by_id.contains_key(dep.as_str()) {
                    return Err(DagError::UnknownDependency {
                        node: node.id.clone(),
                        missing: dep.clone(),
                    });
                }
            }
        }

        // Kahn's algorithm: nodes with in-degree 0 first.
        let mut indegree: HashMap<&str, usize> = HashMap::new();
        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
        for node in &self.nodes {
            indegree.entry(node.id.as_str()).or_insert(0);
            for dep in &node.depends_on {
                // edge dep -> node (dep must finish before node)
                adj.entry(dep.as_str()).or_default().push(node.id.as_str());
                *indegree.get_mut(node.id.as_str()).unwrap() += 1;
            }
        }

        let mut queue: VecDeque<&str> = indegree
            .iter()
            .filter(|&(_, &d)| d == 0)
            .map(|(&id, _)| id)
            .collect();
        let mut order: Vec<String> = Vec::new();
        let mut depth: HashMap<String, usize> = HashMap::new();

        while let Some(id) = queue.pop_front() {
            // depth = 1 + max(dep depths)
            let d = node_depth(id, &by_id, &adj, &mut depth);
            depth.insert(id.to_string(), d);
            order.push(id.to_string());
            if let Some(successors) = adj.get(id) {
                for &succ in successors {
                    let e = indegree.get_mut(succ).unwrap();
                    *e -= 1;
                    if *e == 0 {
                        queue.push_back(succ);
                    }
                }
            }
        }

        if order.len() != self.nodes.len() {
            // Remaining nodes form at least one cycle; recover one for the error.
            let scheduled: HashSet<&str> = order.iter().map(|s| s.as_str()).collect();
            let cycle_start = self
                .nodes
                .iter()
                .find(|n| !scheduled.contains(n.id.as_str()))
                .map(|n| n.id.clone())
                .unwrap_or_default();
            return Err(DagError::Cycle {
                path: recover_cycle(&cycle_start, &by_id),
            });
        }

        Ok(WorkflowPlan { order, depth })
    }
}

/// Compute the dependency depth of `id` (longest chain of predecessors).
/// Memoized via `cache` to avoid re-walking.
fn node_depth(
    id: &str,
    by_id: &HashMap<&str, &WorkflowNode>,
    adj: &HashMap<&str, Vec<&str>>,
    cache: &mut HashMap<String, usize>,
) -> usize {
    if let Some(&d) = cache.get(id) {
        return d;
    }
    let predecessors: Vec<&str> = by_id
        .get(id)
        .map(|n| n.depends_on.iter().map(|s| s.as_str()).collect())
        .unwrap_or_default();
    let d = if predecessors.is_empty() {
        0
    } else {
        1 + predecessors
            .iter()
            .map(|&p| node_depth(p, by_id, adj, cache))
            .max()
            .unwrap_or(0)
    };
    cache.insert(id.to_string(), d);
    d
}

/// Walk backwards from `start` along `depends_on` edges to surface a cycle path.
fn recover_cycle(start: &str, by_id: &HashMap<&str, &WorkflowNode>) -> Vec<String> {
    let mut path = vec![start.to_string()];
    let mut seen = HashSet::new();
    let mut cur = start.to_string();
    while let Some(node) = by_id.get(cur.as_str()) {
        if !node.depends_on.is_empty() {
            let next = &node.depends_on[0];
            if seen.contains(next.as_str()) {
                path.push(next.clone());
                break;
            }
            seen.insert(cur.clone());
            path.push(next.clone());
            cur = next.clone();
        } else {
            break;
        }
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: &str, slot: &str, deps: &[&str]) -> WorkflowNode {
        WorkflowNode {
            id: id.to_string(),
            slot_id: slot.to_string(),
            prompt: String::new(),
            depends_on: deps.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn empty_def_plans_to_empty_order() {
        let def = WorkflowDef { nodes: vec![] };
        let plan = def.plan().unwrap();
        assert!(plan.order.is_empty());
    }

    #[test]
    fn linear_chain_orders_by_dependency() {
        let def = WorkflowDef {
            nodes: vec![
                node("c", "slot3", &["b"]),
                node("a", "slot1", &[]),
                node("b", "slot2", &["a"]),
            ],
        };
        let plan = def.plan().unwrap();
        assert_eq!(plan.order, vec!["a", "b", "c"]);
        assert_eq!(plan.depth["a"], 0);
        assert_eq!(plan.depth["b"], 1);
        assert_eq!(plan.depth["c"], 2);
    }

    #[test]
    fn diamond_resolves_without_cycle() {
        let def = WorkflowDef {
            nodes: vec![
                node("a", "s1", &[]),
                node("b", "s2", &["a"]),
                node("c", "s3", &["a"]),
                node("d", "s4", &["b", "c"]),
            ],
        };
        let plan = def.plan().unwrap();
        let pos = |id: &str| plan.order.iter().position(|x| x == id).unwrap();
        assert!(pos("a") < pos("b"));
        assert!(pos("a") < pos("c"));
        assert!(pos("b") < pos("d"));
        assert!(pos("c") < pos("d"));
    }

    #[test]
    fn detects_simple_cycle() {
        let def = WorkflowDef {
            nodes: vec![node("x", "s1", &["y"]), node("y", "s2", &["x"])],
        };
        match def.plan() {
            Err(DagError::Cycle { path }) => assert!(path.contains(&"x".to_string())),
            other => panic!("expected Cycle, got {other:?}"),
        }
    }

    #[test]
    fn detects_self_cycle() {
        let def = WorkflowDef {
            nodes: vec![node("x", "s1", &["x"])],
        };
        assert!(matches!(def.plan(), Err(DagError::Cycle { .. })));
    }

    #[test]
    fn rejects_unknown_dependency() {
        let def = WorkflowDef {
            nodes: vec![node("a", "s1", &["ghost"])],
        };
        assert!(matches!(
            def.plan(),
            Err(DagError::UnknownDependency { node, missing }) if node == "a" && missing == "ghost"
        ));
    }

    #[test]
    fn rejects_unbound_slot() {
        let def = WorkflowDef {
            nodes: vec![WorkflowNode {
                id: "a".into(),
                slot_id: String::new(),
                prompt: String::new(),
                depends_on: vec![],
            }],
        };
        assert!(matches!(def.plan(), Err(DagError::UnboundSlot { node }) if node == "a"));
    }
}
