use super::{component::ComponentDescriptor, error::ModelError};
use std::collections::{BTreeMap, BTreeSet};

/// Resolve a deterministic topological component order and reject cycles.
pub(crate) fn dependency_order(
    components: &[ComponentDescriptor],
) -> Result<Vec<String>, ModelError> {
    let known: BTreeSet<&str> = components
        .iter()
        .map(|component| component.id.as_str())
        .collect();
    let mut indegree = BTreeMap::<String, usize>::new();
    let mut dependents = BTreeMap::<String, BTreeSet<String>>::new();
    for component in components {
        indegree.insert(component.id.clone(), component.depends_on.len());
        for dependency in &component.depends_on {
            if !known.contains(dependency.as_str()) {
                return Err(ModelError::MissingDependency {
                    component: component.id.clone(),
                    dependency: dependency.clone(),
                });
            }
            dependents
                .entry(dependency.clone())
                .or_default()
                .insert(component.id.clone());
        }
    }

    let mut ready: BTreeSet<String> = indegree
        .iter()
        .filter_map(|(id, count)| (*count == 0).then_some(id.clone()))
        .collect();
    let mut ordered = Vec::with_capacity(components.len());
    while let Some(id) = ready.pop_first() {
        ordered.push(id.clone());
        if let Some(children) = dependents.get(&id) {
            for child in children {
                let Some(count) = indegree.get_mut(child) else {
                    return Err(ModelError::MissingDependency {
                        component: child.clone(),
                        dependency: id.clone(),
                    });
                };
                *count -= 1;
                if *count == 0 {
                    ready.insert(child.clone());
                }
            }
        }
    }
    if ordered.len() != components.len() {
        let cycle = indegree
            .into_iter()
            .filter_map(|(id, count)| (count > 0).then_some(id))
            .collect();
        return Err(ModelError::CircularDependency { components: cycle });
    }
    Ok(ordered)
}
