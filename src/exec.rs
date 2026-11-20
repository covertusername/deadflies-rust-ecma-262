#[allow(dead_code)] // TODO: unfinished
mod env {
    use std::{error::Error, sync::{Arc, Weak}};
    use deadflies_rust_infra::{primitives::IString, structs::{ArcRw, Map}};
    use crate::types::{JType, Object};

    pub struct Environment {
        bindings: ArcRw<Map<IString, JType>>,
        outer_env: Option<Weak<Environment>>,
        inner: Arc<dyn Env>,
    }

    pub trait Env: Send + Sync + std::fmt::Debug {
        fn has_binding(&self, name: IString) -> Result<bool, Box<dyn Error>>;

        fn create_mutable_binding(self: Arc<Self>, name: IString, deletable: bool) -> Result<(), Box<dyn Error>>;

        fn create_immutable_binding(self: Arc<Self>, name: IString, strict: bool) -> Result<(), Box<dyn Error>>;

        fn initialize_binding(self: Arc<Self>, name: IString, val: JType) -> Result<(), Box<dyn Error>>;

        fn set_mutable_binding(self: Arc<Self>, name: IString, val: JType, strict: bool) -> Result<(), Box<dyn Error>>;

        fn get_binding_value(&self, name: IString, strict: bool) -> Result<JType, Box<dyn Error>>;

        fn delete_binding(self: Arc<Self>, name: IString) -> Result<bool, Box<dyn Error>>;

        fn has_this_binding(&self) -> bool;

        fn get_this_binding(&self) -> Result<JType, Box<dyn Error>>;

        fn has_super_binding(&self) -> bool;

        fn with_base_object(&self) -> Option<Object>;

        // implementation-specific functions begin here

        fn get_bindings(&self) -> ArcRw<Map<IString, JType>>;
    }

    pub trait DeclEnv: Env {
        fn der_has_binding(&self, name: &IString) -> Result<bool, Box<dyn Error>> {
            Ok(self.get_bindings().read().contains_key(name).is_some())
        }

        fn der_create_mutable_binding(self: Arc<Self>, name: IString, deletable: bool) -> Result<(), Box<dyn Error>>;

        fn der_create_immutable_binding(self: Arc<Self>, name: IString, strict: bool) -> Result<(), Box<dyn Error>>;

        fn der_initialize_binding(self: Arc<Self>, name: IString, val: JType) -> Result<(), Box<dyn Error>>;

        fn der_set_mutable_binding(self: Arc<Self>, name: IString, val: JType, strict: bool) -> Result<(), Box<dyn Error>>;

        fn der_get_binding_value(&self, name: IString, strict: bool) -> Result<JType, Box<dyn Error>>;

        fn der_delete_binding(self: Arc<Self>, name: IString) -> Result<bool, Box<dyn Error>>;

        fn der_has_this_binding(&self) -> bool;

        fn der_get_this_binding(&self) -> Result<JType, Box<dyn Error>>;

        fn der_has_super_binding(&self) -> bool;

        fn der_with_base_object(&self) -> Option<Object>;
    }
}