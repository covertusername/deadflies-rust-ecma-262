#[allow(dead_code)] // TODO: unfinished
mod env {
    use std::{error::Error, sync::{Arc, Weak}};
    use deadflies_rust_infra::{primitives::IString, structs::{ArcRw, List, Map}};
    use parking_lot::RwLock;
    use crate::{abstract_ops::disposable_ops::Disposable, types::{JString, JType, Key, Object, PropDesc, PropDescData}};

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
        fn has_binding(&self, name: &IString) -> Result<bool, Box<dyn Error>>;

        fn create_mutable_binding(&self, name: IString, deletable: bool) -> Result<(), Box<dyn Error>>;

        fn create_immutable_binding(&self, name: IString, strict: bool) -> Result<(), Box<dyn Error>>;

        fn initialize_binding(&self, name: &IString, val: JType) -> Result<(), Box<dyn Error>>;

        fn set_mutable_binding(&self, name: IString, val: JType, strict: bool) -> Result<(), Box<dyn Error>>;

        fn get_binding_value(&self, name: &IString, strict: bool) -> Result<JType, Box<dyn Error>>;

        fn delete_binding(&self, name: &IString) -> Result<bool, Box<dyn Error>>;

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

        #[allow(unused_variables)] // TODO: unfinished
        #[allow(clippy::redundant_else)] // TODO: unfinished
        fn der_set_mutable_binding(&self, name: IString, val: JType, strict: bool) -> Result<(), Box<dyn Error>> {
            if self.get_bindings().read().contains_key(&name).is_none() {
                if strict {
                    // TODO: Error Objects (Box<dyn Error> is a placeholder)
                    todo!()
                } else {
                    todo!()
                }
            }
            todo!()
        }

        fn der_get_binding_value(&self, name: &IString, _strict: bool) -> Result<JType, Box<dyn Error>> {
            assert!(self.der_has_binding(name)?);

            if matches!(self.get_bindings().read().get(name).unwrap().val, BindingType::None) {
                // TODO: Error Objects (Box<dyn Error> is a placeholder)
                todo!()
            }
            todo!()
        }

        fn der_delete_binding(&self, name: &IString) -> Result<bool, Box<dyn Error>> {
            assert!(self.der_has_binding(name)?);
            
            if self.get_bindings().read().get(name).unwrap().deletable {
                self.get_bindings().write().remove(|(k, _)| *k.read() == *name);
                Ok(true)
            } else {
                Ok(false)
            }
        }

        fn der_has_this_binding(&self) -> bool {
            false
        }

        fn der_get_this_binding(&self) -> Result<JType, Box<dyn Error>> {
            unimplemented!() // explicitly unimplemented by specification
        }

        fn der_has_super_binding(&self) -> bool {
            false
        }

        fn der_with_base_object(&self) -> Option<Object> {
            None
        }
    }

    #[derive(Debug)]
    pub struct DeclEnvConcrete {
        bindings: Weak<RwLock<Map<IString, BindingDesc>>>,
        outer_env: Option<Weak<dyn Env>>,
        disposable_stack: ArcRw<List<Disposable>>,
    }

    impl DeclEnv for DeclEnvConcrete {}

    impl Env for DeclEnvConcrete {
        fn has_binding(&self, name: &IString) -> Result<bool, Box<dyn Error>> {
            self.der_has_binding(name)
        }
    
        fn create_mutable_binding(&self, name: IString, deletable: bool) -> Result<(), Box<dyn Error>> {
            self.der_create_mutable_binding(name, deletable)
        }
    
        fn create_immutable_binding(&self, name: IString, strict: bool) -> Result<(), Box<dyn Error>> {
            self.der_create_immutable_binding(name, strict)
        }
    
        fn initialize_binding(&self, name: &IString, val: JType) -> Result<(), Box<dyn Error>> {
            self.der_initialize_binding(name, val)
        }
    
        fn set_mutable_binding(&self, name: IString, val: JType, strict: bool) -> Result<(), Box<dyn Error>> {
            self.der_set_mutable_binding(name, val, strict)
        }
    
        fn get_binding_value(&self, name: &IString, strict: bool) -> Result<JType, Box<dyn Error>> {
            self.der_get_binding_value(name, strict)
        }
    
        fn delete_binding(&self, name: &IString) -> Result<bool, Box<dyn Error>> {
            self.der_delete_binding(name)
        }
    
        fn has_this_binding(&self) -> bool {
            self.der_has_this_binding()
        }
    
        fn get_this_binding(&self) -> Result<JType, Box<dyn Error>> {
            self.der_get_this_binding()
        }
    
        fn has_super_binding(&self) -> bool {
            self.der_has_super_binding()
        }
    
        fn with_base_object(&self) -> Option<Object> {
            self.der_with_base_object()
        }
    
        fn get_bindings(&self) -> ArcRw<Map<IString, BindingDesc>> {
            self.bindings.upgrade().unwrap()
        }
    }

    #[derive(Debug)]
    pub struct ObjectEnv {
        // NOTE: this doesn't track its own bindings, its bindings are the named properties of the binding object
        outer_env: Option<Weak<dyn Env>>,
        binding_object: Object,
        is_with_env: bool,
    }

    impl Env for ObjectEnv {
        fn has_binding(&self, name: &IString) -> Result<bool, Box<dyn Error>> {
            if self.binding_object.has_property(&name.as_key())? {
                if self.is_with_env {
                    if let JType::Object(unscopables) = self.binding_object.get(&Key::get_known(crate::types::SymNames::Unscopables))? &&
                        unscopables.get(&name.as_key())?.to_bool() {
                        Ok(false)
                    } else {
                        Ok(true)
                    }
                } else {
                    Ok(true)
                }
            } else {
                Ok(false)
            }
        }
    
        fn create_mutable_binding(&self, name: IString, deletable: bool) -> Result<(), Box<dyn Error>> {
            self.binding_object.define_property_or_throw(Key::String(name), PropDesc {
                kind: crate::types::PropDescType::Data(PropDescData {
                    val: Some(JType::Undefined),
                    writable: Some(true),
                }),
                enumerable: Some(true),
                configurable: Some(deletable),
            })
        }
    
        fn create_immutable_binding(&self, _name: IString, _strict: bool) -> Result<(), Box<dyn Error>> {
            unimplemented!() // explicitly unimplemented by specification
        }
    
        fn initialize_binding(&self, name: &IString, val: JType) -> Result<(), Box<dyn Error>> {
            self.set_mutable_binding(name.clone(), val, false)
        }
    
        #[allow(unused_variables)] // TODO: unfinished
        #[allow(clippy::redundant_else)] // TODO: unfinished
        fn set_mutable_binding(&self, name: IString, val: JType, strict: bool) -> Result<(), Box<dyn Error>> {
            if !self.binding_object.has_property(&name.as_key())? && strict {
                // TODO: Error Objects (Box<dyn Error> is a placeholder)
                todo!()
            } else {
                todo!()
            }
        }
    
        fn get_binding_value(&self, name: &IString, strict: bool) -> Result<JType, Box<dyn Error>> {
            if self.binding_object.has_property(&name.as_key())? {
                self.binding_object.get(&name.as_key())
            } else {
                if strict {
                    // TODO: Error Objects (Box<dyn Error> is a placeholder)
                    todo!()
                }
                todo!()
            }
        }
    
        fn delete_binding(&self, name: &IString) -> Result<bool, Box<dyn Error>> {
            self.binding_object.inner.clone().delete(name.as_key())
        }
    
        fn has_this_binding(&self) -> bool {
            false
        }
    
        fn get_this_binding(&self) -> Result<JType, Box<dyn Error>> {
            unimplemented!() // explicitly unimplemented by specification
        }
    
        fn has_super_binding(&self) -> bool {
            false
        }
    
        fn with_base_object(&self) -> Option<Object> {
            if self.is_with_env {
                Some(self.binding_object.clone_shallow())
            } else {
                None
            }
        }
    
        fn get_bindings(&self) -> ArcRw<Map<IString, BindingDesc>> {
            unimplemented!() // not applicable for this kind of Env
        }
    }

    #[derive(Debug)]
    pub enum FuncEnvThisState {
        Lexical,
        Initialized,
        Uninitialized,
    }

    #[derive(Debug)]
    pub struct FuncEnv {
        bindings: Weak<RwLock<Map<IString, BindingDesc>>>,
        outer_env: Option<Weak<dyn Env>>,
        disposable_stack: ArcRw<List<Disposable>>,
        this_value: ArcRw<JType>,
        this_status: ArcRw<FuncEnvThisState>,
        func: Object,
        new_target: Option<Object>,
    }

    impl FuncEnv {
        #[allow(unused_variables)] // TODO: unfinished
        #[allow(clippy::needless_pass_by_value)] // TODO: unfinished
        pub fn bind_this_value(&self, val: JType) -> Result<(), Box<dyn Error>> {
            match *self.this_status.read() {
                FuncEnvThisState::Lexical => panic!("FuncEnv::bind_this_value: cannot bind this values to lexical function environments"),
                FuncEnvThisState::Initialized => {
                    // TODO: Error Objects (Box<dyn Error> is a placeholder)
                    todo!()
                },
                FuncEnvThisState::Uninitialized => todo!(),
            }
        }

        // technically can only return Object, Null, or Undefined but i did not feel like making a separate enum just for this function
        pub fn get_super_base(&self) -> JType {
            // TODO: Full function object implementation. We currently don't track internal slots.
            todo!()
        }
    }

    impl DeclEnv for FuncEnv {}

    impl Env for FuncEnv {
        fn has_binding(&self, name: &IString) -> Result<bool, Box<dyn Error>> {
            self.der_has_binding(name)
        }
    
        fn create_mutable_binding(&self, name: IString, deletable: bool) -> Result<(), Box<dyn Error>> {
            self.der_create_mutable_binding(name, deletable)
        }
    
        fn create_immutable_binding(&self, name: IString, strict: bool) -> Result<(), Box<dyn Error>> {
            self.der_create_immutable_binding(name, strict)
        }
    
        fn initialize_binding(&self, name: &IString, val: JType) -> Result<(), Box<dyn Error>> {
            self.der_initialize_binding(name, val)
        }
    
        fn set_mutable_binding(&self, name: IString, val: JType, strict: bool) -> Result<(), Box<dyn Error>> {
            self.der_set_mutable_binding(name, val, strict)
        }
    
        fn get_binding_value(&self, name: &IString, strict: bool) -> Result<JType, Box<dyn Error>> {
            self.der_get_binding_value(name, strict)
        }
    
        fn delete_binding(&self, name: &IString) -> Result<bool, Box<dyn Error>> {
            self.der_delete_binding(name)
        }
    
        fn has_this_binding(&self) -> bool {
            !matches!(*self.this_status.read(), FuncEnvThisState::Lexical)
        }
    
        fn get_this_binding(&self) -> Result<JType, Box<dyn Error>> {
            match *self.this_status.read() {
                FuncEnvThisState::Lexical => panic!("FuncEnv::get_this_binding: no this binding for lexical function environments"),
                FuncEnvThisState::Uninitialized => {
                    // TODO: Error Objects (Box<dyn Error> is a placeholder)
                    todo!()
                },
                FuncEnvThisState::Initialized => todo!(),
            }
        }
    
        fn has_super_binding(&self) -> bool {
            // TODO: Full function object implementation. We currently don't track internal slots.
            todo!()
        }
    
        fn with_base_object(&self) -> Option<Object> {
            self.der_with_base_object()
        }
    
        fn get_bindings(&self) -> ArcRw<Map<IString, BindingDesc>> {
            self.bindings.upgrade().unwrap()
        }
    }

    #[derive(Debug)]
    pub struct GlobalEnv {
        outer_env: Option<Weak<dyn Env>>,
        object_record: ObjectEnv,
        decl_record: DeclEnvConcrete,
    }

    impl Env for GlobalEnv {
        fn has_binding(&self, name: &IString) -> Result<bool, Box<dyn Error>> {
            if self.decl_record.has_binding(name).unwrap() {
                Ok(true)
            } else {
                self.object_record.has_binding(name)
            }
        }
    
        #[allow(unused_variables)] // TODO: unfinished
        fn create_mutable_binding(&self, name: IString, deletable: bool) -> Result<(), Box<dyn Error>> {
            if self.decl_record.has_binding(&name).unwrap() {
                // TODO: Error Objects (Box<dyn Error> is a placeholder)
                todo!()
            }
            todo!()
        }
    
        #[allow(unused_variables)] // TODO: unfinished
        fn create_immutable_binding(&self, name: IString, strict: bool) -> Result<(), Box<dyn Error>> {
            if self.decl_record.has_binding(&name).unwrap() {
                // TODO: Error Objects (Box<dyn Error> is a placeholder)
                todo!()
            }
            todo!()
        }
    
        fn initialize_binding(&self, name: &IString, val: JType) -> Result<(), Box<dyn Error>> {
            if self.decl_record.has_binding(name).unwrap() {
                self.decl_record.initialize_binding(name, val)
            } else {
                self.object_record.initialize_binding(name, val)
            }
        }
    
        fn set_mutable_binding(&self, name: IString, val: JType, strict: bool) -> Result<(), Box<dyn Error>> {
            if self.decl_record.has_binding(&name).unwrap() {
                self.decl_record.set_mutable_binding(name, val, strict)
            } else {
                self.object_record.set_mutable_binding(name, val, strict)
            }
        }
    
        fn get_binding_value(&self, name: &IString, strict: bool) -> Result<JType, Box<dyn Error>> {
            if self.decl_record.has_binding(name).unwrap() {
                self.decl_record.get_binding_value(name, strict)
            } else {
                self.object_record.get_binding_value(name, strict)
            }
        }
    
        fn delete_binding(&self, name: &IString) -> Result<bool, Box<dyn Error>> {
            todo!()
        }
    
        fn has_this_binding(&self) -> bool {
            todo!()
        }
    
        fn get_this_binding(&self) -> Result<JType, Box<dyn Error>> {
            todo!()
        }
    
        fn has_super_binding(&self) -> bool {
            todo!()
        }
    
        fn with_base_object(&self) -> Option<Object> {
            todo!()
        }
    
        fn get_bindings(&self) -> ArcRw<Map<IString, BindingDesc>> {
            todo!()
        }
    }
}