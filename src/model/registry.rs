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
    /// Creates an empty deterministic static registry.  Registration is a
    /// construction-time operation; the compiled model only receives the
    /// immutable registry and therefore cannot change scientific behaviour at
    /// runtime.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers one stable component kind. Duplicate kinds are rejected
    /// rather than allowing construction order to select a factory.
    pub fn register(
        &mut self,
        kind: impl Into<String>,
        factory: ComponentFactory,
    ) -> Result<(), ModelError> {
        let kind = kind.into();
        if kind.trim().is_empty() {
            return Err(ModelError::EmptyIdentifier {
                kind: "component kind",
            });
        }
        if self.factories.contains_key(&kind) {
            return Err(ModelError::DuplicateComponentKind { kind });
        }
        self.factories.insert(kind, factory);
        Ok(())
    }

    pub fn from_static_factories(
        factories: impl IntoIterator<Item = (&'static str, ComponentFactory)>,
    ) -> Self {
        let mut registry = Self::new();
        for (kind, factory) in factories {
            // Built-ins are compile-time declarations. A duplicate here is a
            // programmer error and must never silently choose a factory.
            registry
                .register(kind, factory)
                .expect("duplicate static ISM component kind");
        }
        registry
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

/// Global static registry for the reduced-order built-ins. Factories are
/// immutable after construction; runtime dynamic-library plugins are outside
/// the model-core contract.
pub fn built_in_registry() -> &'static ComponentRegistry {
    BUILT_IN_REGISTRY.get_or_init(|| {
        ComponentRegistry::from_static_factories(super::builtins::static_factories())
    })
}
