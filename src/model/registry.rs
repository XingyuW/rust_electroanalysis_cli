use super::{component::ComponentDescriptor, component::IsmComponent, error::ModelError};
use std::{collections::BTreeMap, sync::OnceLock};

/// Static factory signature. Factories are compiled into the binary; dynamic
/// library plugin loading is deliberately unsupported.
pub type ComponentFactory = fn(&ComponentDescriptor) -> Result<Box<dyn IsmComponent>, ModelError>;

/// Immutable lookup table for component factories keyed by stable kind string.
#[derive(Clone, Default)]
pub struct ComponentRegistry {
    factories: BTreeMap<String, ComponentFactory>,
}

impl ComponentRegistry {
    pub fn from_static_factories(
        factories: impl IntoIterator<Item = (&'static str, ComponentFactory)>,
    ) -> Self {
        Self {
            factories: factories
                .into_iter()
                .map(|(kind, factory)| (kind.to_string(), factory))
                .collect(),
        }
    }

    pub fn create(
        &self,
        descriptor: &ComponentDescriptor,
    ) -> Result<Box<dyn IsmComponent>, ModelError> {
        let Some(factory) = self.factories.get(&descriptor.kind) else {
            return Err(ModelError::UnknownComponentKind {
                component: descriptor.id.clone(),
                kind: descriptor.kind.clone(),
            });
        };
        factory(descriptor)
    }
}

static BUILT_IN_REGISTRY: OnceLock<ComponentRegistry> = OnceLock::new();

/// Global static registry for built-in production components. It is empty in
/// Phase 02 because real scientific components are intentionally deferred.
pub fn built_in_registry() -> &'static ComponentRegistry {
    BUILT_IN_REGISTRY.get_or_init(|| {
        ComponentRegistry::from_static_factories(super::builtins::static_factories())
    })
}
