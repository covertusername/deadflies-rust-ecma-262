#[allow(dead_code)] // TODO: unfinished
mod env {
    use std::{error::Error, sync::{Arc, Weak}};
    use deadflies_rust_infra::{primitives::IString, structs::{ArcRw, Map}};
    use crate::types::{JType, Object};

    #[derive(Debug, Clone)]
    pub struct IndirectBinding {
        target_mod: !, // TODO: Module Records
        bind_key: IString,
    }

    #[derive(Debug, Clone)]
    pub enum BindingType {
        None, // uninitialized
        Direct(JType),
        Indirect(IndirectBinding),
    }

    /*
     * not a specification type, but contains things marked as "to be recorded"
     * within a DeclarativeEnv. spec doesn't say how, probably impl-defined
     */
    #[derive(Debug, Clone)]
    pub struct BindingDesc {
        val: BindingType, // if not initialized, is BindingValType::None
        mutable: bool,
        deletable: bool,
        strict: bool,
    }

    pub struct Environment {
        bindings: ArcRw<Map<IString, BindingDesc>>,
        outer_env: Option<Weak<dyn Env>>,
        inner: Arc<dyn Env>,
    }

    pub trait Env: Send + Sync + std::fmt::Debug {
        fn has_binding(&self, name: IString) -> Result<bool, Box<dyn Error>>;

        fn create_mutable_binding(&self, name: IString, deletable: bool) -> Result<(), Box<dyn Error>>;

        fn create_immutable_binding(&self, name: IString, strict: bool) -> Result<(), Box<dyn Error>>;

        fn initialize_binding(&self, name: IString, val: JType) -> Result<(), Box<dyn Error>>;

        fn set_mutable_binding(&self, name: IString, val: JType, strict: bool) -> Result<(), Box<dyn Error>>;

        fn get_binding_value(&self, name: IString, strict: bool) -> Result<JType, Box<dyn Error>>;

        fn delete_binding(&self, name: IString) -> Result<bool, Box<dyn Error>>;

        fn has_this_binding(&self) -> bool;

        fn get_this_binding(&self) -> Result<JType, Box<dyn Error>>;

        fn has_super_binding(&self) -> bool;

        fn with_base_object(&self) -> Option<Object>;

        // implementation-specific functions begin here

        fn get_bindings(&self) -> ArcRw<Map<IString, BindingDesc>>;
    }

    pub trait DeclEnv: Env {
        fn der_has_binding(&self, name: &IString) -> Result<bool, Box<dyn Error>> {
            Ok(self.get_bindings().read().contains_key(name).is_some())
        }

        fn der_create_mutable_binding(&self, name: IString, deletable: bool) -> Result<(), Box<dyn Error>> {
            assert!(!self.der_has_binding(&name)?);

            self.get_bindings().write().set(name, BindingDesc {
                val: BindingType::None,
                mutable: true,
                deletable,
                strict: false,
            });
            Ok(())
        }

        fn der_create_immutable_binding(&self, name: IString, strict: bool) -> Result<(), Box<dyn Error>> {
            assert!(!self.der_has_binding(&name)?);

            self.get_bindings().write().set(name, BindingDesc {
                val: BindingType::None,
                mutable: false,
                deletable: false,
                strict,
            });
            Ok(())
        }

        fn der_initialize_binding(&self, name: &IString, val: JType) -> Result<(), Box<dyn Error>> {
            assert!(matches!(self.get_bindings().read().get(name).unwrap().val, BindingType::None));

            self.get_bindings().write().get_mut(name).unwrap().val = BindingType::Direct(val);
            Ok(())
        }

        fn der_set_mutable_binding(&self, name: IString, val: JType, strict: bool) -> Result<(), Box<dyn Error>>;

        fn der_get_binding_value(&self, name: IString, strict: bool) -> Result<JType, Box<dyn Error>>;

        fn der_delete_binding(&self, name: IString) -> Result<bool, Box<dyn Error>>;

        fn der_has_this_binding(&self) -> bool;

        fn der_get_this_binding(&self) -> Result<JType, Box<dyn Error>>;

        fn der_has_super_binding(&self) -> bool;

        fn der_with_base_object(&self) -> Option<Object>;
    }
}