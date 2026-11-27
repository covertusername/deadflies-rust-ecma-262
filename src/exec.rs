#[allow(dead_code)] // TODO: unfinished
pub mod env {
    use std::{error::Error, sync::{Arc, Weak}};
    use deadflies_rust_infra::{primitives::IString, structs::{ArcRw, List, Map}};
    use parking_lot::RwLock;
    use crate::{abstract_ops::disposable_ops::Disposable, types::{JString, JType, Key, Object, PropDesc, PropDescData, Reference, Todo}};

    #[derive(Debug, Clone)]
    pub struct IndirectBinding {
        target_mod: Todo, // TODO: Module Records
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
        outer: Option<Weak<dyn Env>>,
        inner: Arc<dyn Env>,
    }

    impl Environment {
        pub fn clone_shallow(&self) -> Environment {
            Environment {
                outer: self.outer.clone(),
                inner: self.inner.clone(),
            }
        }

        /*
         * technically need to have a verion where self is JType::Null but this
         * only happens when this function is calling itself recursively on the
         * parent environment (which might not exist). can simulate without
         * recursion
         */
        pub fn get_identifier_reference(&self, name: &IString, strict: bool) -> Result<Reference, Box<dyn Error>> {
            if self.inner.has_binding(name)? {
                Ok(Reference {
                    base: crate::types::RefBase::EnvRec(self.clone_shallow()),
                    referenced_name: crate::types::RefRefName::Value(JType::JString(name.clone())),
                    strict,
                    this_value: None,
                })
            } else {
                match self.outer {
                    Some(ref env) => env.upgrade().unwrap().make_environment().get_identifier_reference(name, strict),
                    None => Ok(Reference {
                        base: crate::types::RefBase::Unresolvable,
                        referenced_name: crate::types::RefRefName::Value(JType::JString(name.clone())),
                        strict,
                        this_value: None,
                    }),
                }
            }
        }
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

        fn get_outer(&self) -> Option<Weak<dyn Env>>;

        fn get_self(&self) -> Arc<dyn Env>;

        fn make_environment(self: Arc<Self>) -> Environment {
            Environment {
                outer: self.get_outer(),
                inner: self.get_self(),
            }
        }
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

        // implementation-specific functions begin here

        fn get_bindings(&self) -> ArcRw<Map<IString, BindingDesc>>;
    }

    #[derive(Debug)]
    pub struct DeclEnvConcrete {
        bindings: ArcRw<Map<IString, BindingDesc>>,
        outer: Option<Weak<dyn Env>>,
        me: Weak<dyn Env>,
        disposable_stack: ArcRw<List<Disposable>>,
    }

    impl DeclEnvConcrete {
        pub fn new(outer: Option<Weak<dyn Env>>) -> Arc<Self> {
            Arc::new_cyclic(|me| DeclEnvConcrete {
                bindings: Arc::new(RwLock::new(Map::new())),
                outer,
                me: me.clone(),
                disposable_stack: Arc::new(RwLock::new(List::new())),
            })
        }

        pub fn new_wrapped(outer: Option<Weak<dyn Env>>) -> Environment {
            DeclEnvConcrete::new(outer).make_environment()
        }
    }

    impl DeclEnv for DeclEnvConcrete {
        fn get_bindings(&self) -> ArcRw<Map<IString, BindingDesc>> {
            self.bindings.clone()
        }
    }

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

        fn get_outer(&self) -> Option<Weak<dyn Env>> {
            self.outer.clone()
        }

        fn get_self(&self) -> Arc<dyn Env> {
            self.me.upgrade().unwrap()
        }
    }

    #[derive(Debug)]
    pub struct ObjectEnv {
        outer: Option<Weak<dyn Env>>,
        me: Weak<dyn Env>,
        binding_object: Object,
        is_with_env: bool,
    }

    impl ObjectEnv {
        pub fn new(obj: Object, is_with_env: bool, outer: Option<Weak<dyn Env>>) -> Arc<Self> {
            Arc::new_cyclic(|me| ObjectEnv {
                outer,
                me: me.clone(),
                binding_object: obj,
                is_with_env,
            })
        }

        pub fn new_wrapped(obj: Object, is_with_env: bool, outer: Option<Weak<dyn Env>>) -> Environment {
            ObjectEnv::new(obj, is_with_env, outer).make_environment()
        }
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

        fn get_outer(&self) -> Option<Weak<dyn Env>> {
            self.outer.clone()
        }

        fn get_self(&self) -> Arc<dyn Env> {
            self.me.upgrade().unwrap()
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
        bindings: ArcRw<Map<IString, BindingDesc>>,
        outer: Option<Weak<dyn Env>>,
        me: Weak<dyn Env>,
        disposable_stack: ArcRw<List<Disposable>>,
        this_value: ArcRw<JType>,
        this_status: ArcRw<FuncEnvThisState>,
        func: Object,
        new_target: Option<Object>,
    }

    impl FuncEnv {
        #[allow(unused_variables)] // TODO: unfinished
        #[allow(unreachable_code)] // TODO: unfinished
        #[allow(unused_assignments)] // TODO: unfinished
        pub fn new(func: Object, new_target: Option<Object>) -> Arc<Self> {
            // TODO: Full function object implementation. We currently don't track internal slots.
            Arc::new_cyclic(|me| FuncEnv {
                bindings: Arc::new(RwLock::new(Map::new())),
                outer: todo!(),
                me: todo!(),
                disposable_stack: todo!(),
                this_value: todo!(),
                this_status: todo!(),
                func,
                new_target,
            })
        }

        pub fn new_wrapped(func: Object, new_target: Option<Object>) -> Environment {
            FuncEnv::new(func, new_target).make_environment()
        }

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

    impl DeclEnv for FuncEnv {
        fn get_bindings(&self) -> ArcRw<Map<IString, BindingDesc>> {
            self.bindings.clone()
        }
    }

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

        fn get_outer(&self) -> Option<Weak<dyn Env>> {
            self.outer.clone()
        }

        fn get_self(&self) -> Arc<dyn Env> {
            self.me.upgrade().unwrap()
        }
    }

    #[derive(Debug)]
    pub struct GlobalEnv {
        outer: Option<Weak<dyn Env>>,
        me: Weak<dyn Env>,
        object_record: Arc<ObjectEnv>,
        pub global_this: Object,
        decl_record: Arc<DeclEnvConcrete>,
    }

    impl GlobalEnv {
        pub fn new(obj: Object, this: Object) -> Arc<Self> {
            Arc::new_cyclic(|me| GlobalEnv {
                outer: None,
                me: me.clone(),
                object_record: ObjectEnv::new(obj, false, None),
                global_this: this,
                decl_record: DeclEnvConcrete::new(None),
            })
        }

        pub fn new_wrapped(obj: Object, this: Object) -> Environment {
            GlobalEnv::new(obj, this).make_environment()
        }

        pub fn has_lexical_declaration(&self, name: &IString) -> bool {
            self.decl_record.has_binding(name).unwrap()
        }

        pub fn has_restricted_global_property(&self, name: &IString) -> Result<bool, Box<dyn Error>> {
            if let Some(existing_property) = self.object_record.binding_object.inner.get_own_property(&name.as_key())? {
                Ok(!existing_property.configurable.unwrap())
            } else {
                Ok(false)
            }
        }

        pub fn can_declare_global_var(&self, name: &IString) -> Result<bool, Box<dyn Error>> {
            if self.object_record.binding_object.has_own_property(&name.as_key())? {
                Ok(true)
            } else {
                self.object_record.binding_object.is_extensible()
            }
        }

        pub fn can_declare_global_function(&self, name: &IString) -> Result<bool, Box<dyn Error>> {
            if let Some(existing_property) = self.object_record.binding_object.inner.get_own_property(&name.as_key())? {
                if existing_property.configurable.unwrap() || (existing_property.is_data().is_some_and(|x| x.writable.unwrap()) && existing_property.enumerable.unwrap()) {
                    Ok(true)
                } else {
                    Ok(false)
                }
            } else {
                self.object_record.binding_object.is_extensible()
            }
        }

        pub fn create_global_var_binding(&self, name: &IString, deletable: bool) -> Result<(), Box<dyn Error>> {
            if !self.object_record.binding_object.has_own_property(&name.as_key())? && self.object_record.binding_object.is_extensible()? {
                self.object_record.create_mutable_binding(name.clone(), deletable)?;
                self.object_record.initialize_binding(name, JType::Undefined)?;
            }
            Ok(())
        }

        pub fn create_global_function_binding(&self, name: &IString, val: &Object, deletable: bool) -> Result<(), Box<dyn Error>> {
            let existing_property = self.object_record.binding_object.inner.get_own_property(&name.as_key())?;
            let desc = if existing_property.is_none_or(|x| x.configurable.unwrap()) {
                PropDesc {
                    kind: crate::types::PropDescType::Data(PropDescData {
                        val: Some(JType::Object(val.clone_shallow())),
                        writable: Some(true),
                    }),
                    enumerable: Some(true),
                    configurable: Some(deletable),
                }
            } else {
                PropDesc {
                    kind: crate::types::PropDescType::Data(PropDescData {
                        val: Some(JType::Object(val.clone_shallow())),
                        writable: None,
                    }),
                    enumerable: None,
                    configurable: None,
                }
            };

            self.object_record.binding_object.define_property_or_throw(name.as_key(), desc)?;
            self.object_record.binding_object.set(name.as_key(), JType::Object(val.clone_shallow()), false)
        }
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
            if self.decl_record.has_binding(name).unwrap() {
                self.decl_record.delete_binding(name)
            } else if self.object_record.binding_object.has_own_property(&name.as_key())? {
                self.object_record.delete_binding(name)
            } else {
                Ok(true)
            }
        }
    
        fn has_this_binding(&self) -> bool {
            true
        }
    
        fn get_this_binding(&self) -> Result<JType, Box<dyn Error>> {
            Ok(JType::Object(self.global_this.clone_shallow()))
        }
    
        fn has_super_binding(&self) -> bool {
            false
        }
    
        fn with_base_object(&self) -> Option<Object> {
            None
        }

        fn get_outer(&self) -> Option<Weak<dyn Env>> {
            self.outer.clone()
        }

        fn get_self(&self) -> Arc<dyn Env> {
            self.me.upgrade().unwrap()
        }
    }

    #[derive(Debug)]
    pub struct ModuleEnv {
        bindings: ArcRw<Map<IString, BindingDesc>>,
        me: Weak<dyn Env>,
        outer: Option<Weak<dyn Env>>,
        disposable_stack: ArcRw<List<Disposable>>,
    }

    impl ModuleEnv {
        pub fn new(outer: Weak<GlobalEnv>) -> Arc<Self> {
            Arc::new_cyclic(|me| ModuleEnv {
                bindings: Arc::new(RwLock::new(Map::new())),
                me: me.clone(),
                outer: Some(outer),
                disposable_stack: Arc::new(RwLock::new(List::new())),
            })
        }

        #[allow(unused_variables)] // TODO: unfinished
        #[allow(clippy::needless_pass_by_value)] // TODO: unfinished
        pub fn create_import_binding(&self, name: IString, target_module: Todo, target_name: IString) {
            // TODO: Module Records
            todo!()
        }
    }

    impl DeclEnv for ModuleEnv {
        fn get_bindings(&self) -> ArcRw<Map<IString, BindingDesc>> {
            self.bindings.clone()
        }
    }

    impl Env for ModuleEnv {
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
    
        #[allow(unused_variables)] // TODO: unfinished
        fn get_binding_value(&self, name: &IString, strict: bool) -> Result<JType, Box<dyn Error>> {
            assert!(strict);
            assert!(self.has_binding(name).unwrap());

            match self.get_bindings().read().get(name).unwrap().val {
                BindingType::Indirect(indirect_binding) => {
                    // TODO: Module Records
                    todo!()
                },
                BindingType::None => todo!(),
                BindingType::Direct(jtype) => todo!(),
            }
        }
    
        fn delete_binding(&self, _name: &IString) -> Result<bool, Box<dyn Error>> {
            unimplemented!() // explicitly unimplemented by specification
        }
    
        fn has_this_binding(&self) -> bool {
            true
        }
    
        fn get_this_binding(&self) -> Result<JType, Box<dyn Error>> {
            Ok(JType::Undefined)
        }
    
        fn has_super_binding(&self) -> bool {
            self.der_has_super_binding()
        }
    
        fn with_base_object(&self) -> Option<Object> {
            self.der_with_base_object()
        }

        fn get_outer(&self) -> Option<Weak<dyn Env>> {
            self.outer.clone()
        }

        fn get_self(&self) -> Arc<dyn Env> {
            self.me.upgrade().unwrap()
        }
    }
}

#[allow(dead_code)] // TODO: unfinished, module not yet used elsewhere
mod priv_env {
    use std::sync::{Arc, Weak};
    use deadflies_rust_infra::{primitives::IString, structs::{ArcRw, List}};
    use parking_lot::RwLock;
    use crate::types::PrivName;

    pub struct PrivEnv {
        outer: Option<Weak<PrivEnv>>,
        names: ArcRw<List<Arc<PrivName>>>,
    }

    impl PrivEnv {
        pub fn new(outer: Option<Weak<PrivEnv>>) -> Arc<PrivEnv> {
            Arc::new(PrivEnv {
                outer,
                names: Arc::new(RwLock::new(List::new())),
            })
        }

        pub fn resolve_ident(&self, ident: IString) -> Arc<PrivName> {
            for name in self.names.read().iter() {
                if name.read().description == ident {
                    return name.read().clone();
                }
            }
            // ASSERT: self.outer.is_some() - if we don't have an outer we're screwed anyway, hence the nasty double-unwrap
            self.outer.as_ref().unwrap().upgrade().unwrap().resolve_ident(ident)
        }
    }
}

#[allow(dead_code)] // TODO: unfinished
mod realm {
    use std::{error::Error, str::FromStr, sync::Arc};
    use deadflies_rust_infra::structs::{ArcRw, List};
    use crate::{exec::env::GlobalEnv, types::{JType, Key, Object, PROPDESC_DATA_DEFAULT, PROPDESC_DEFAULT, PropDesc, PropDescData, Todo}};

    // nasty
    pub struct Intrinsics {
        pub agg_err: Object,
        pub arr: Object,
        pub arr_buf: Object,
        pub arr_iter_proto: Object,
        pub async_disposable_stack: Object,
        pub async_from_sync_iter_proto: Object,
        pub async_func: Object,
        pub async_gen_func: Object,
        pub async_gen_proto: Object,
        pub async_iter_proto: Object,
        pub atomics: Object,
        pub bigint: Object,
        pub bigint_64_arr: Object,
        pub biguint_64_arr: Object,
        pub bool: Object,
        pub data_view: Object,
        pub date: Object,
        pub decode_uri: Object,
        pub decode_uri_comp: Object,
        pub disposable_stack: Object,
        pub encode_uri: Object,
        pub encode_uri_comp: Object,
        pub error: Object,
        pub eval: Object,
        pub eval_error: Object,
        pub final_reg: Object,
        pub f16_arr: Object,
        pub f32_arr: Object,
        pub f64_arr: Object,
        pub for_in_iter_proto: Object,
        pub func: Object,
        pub gen_func: Object,
        pub gen_proto: Object,
        pub i8_arr: Object,
        pub i16_arr: Object,
        pub i32_arr: Object,
        pub is_finite: Object,
        pub is_nan: Object,
        pub iter: Object,
        pub iter_helper_proto: Object,
        pub json: Object,
        pub map: Object,
        pub map_iter_proto: Object,
        pub math: Object,
        pub number: Object,
        pub object: Object,
        pub parse_float: Object,
        pub parse_int: Object,
        pub promise: Object,
        pub proxy: Object,
        pub range_err: Object,
        pub ref_err: Object,
        pub reflect: Object,
        pub regexp: Object,
        pub regexp_string_iter_proto: Object,
        pub set: Object,
        pub set_iter_proto: Object,
        pub shared_arr_buf: Object,
        pub string: Object,
        pub string_iter_proto: Object,
        pub suppressed_err: Object,
        pub symbol: Object,
        pub syntax_err: Object,
        pub throw_type_err: Object,
        pub typed_arr: Object,
        pub type_err: Object,
        pub u8_arr: Object,
        pub u8_clamp_arr: Object,
        pub u16_arr: Object,
        pub u32_arr: Object,
        pub uri_err: Object,
        pub weak_map: Object,
        pub weak_ref: Object,
        pub weak_set: Object,
        pub wrap_for_valid_iter_proto: Object,
    }

    pub struct Template {
        site: Todo, // TODO: Parse Nodes
        array: Todo, // TODO: Arrays
    }

    pub struct Realm {
        agent: Todo, // TODO: Agents/Agent Records
        pub intrinsics: Option<Intrinsics>, // exceedingly gross, but needs to be None during initialization and then never again
        global_obj: Object,
        global_env: Arc<GlobalEnv>,
        template_map: ArcRw<List<Template>>,
        loaded_modules: ArcRw<List<Todo>>, // TODO: LoadedModuleRequest records
        host_defined: JType, // not currently used so will be undefined for now
    }

    impl Realm {
        // NOTE: this only ever gets called once during realm init, so we shouldn't need to worry about locking anything
        #[allow(unreachable_code)] // TODO: unfinished
        fn create_intrinsics(&mut self) {
            // TODO: None of these objects exist yet.
            self.intrinsics = Some(Intrinsics {
                agg_err: todo!(),
                arr: todo!(),
                arr_buf: todo!(),
                arr_iter_proto: todo!(),
                async_disposable_stack: todo!(),
                async_from_sync_iter_proto: todo!(),
                async_func: todo!(),
                async_gen_func: todo!(),
                async_gen_proto: todo!(),
                async_iter_proto: todo!(),
                atomics: todo!(),
                bigint: todo!(),
                bigint_64_arr: todo!(),
                biguint_64_arr: todo!(),
                bool: todo!(),
                data_view: todo!(),
                date: todo!(),
                decode_uri: todo!(),
                decode_uri_comp: todo!(),
                disposable_stack: todo!(),
                encode_uri: todo!(),
                encode_uri_comp: todo!(),
                error: todo!(),
                eval: todo!(),
                eval_error: todo!(),
                final_reg: todo!(),
                f16_arr: todo!(),
                f32_arr: todo!(),
                f64_arr: todo!(),
                for_in_iter_proto: todo!(),
                func: todo!(),
                gen_func: todo!(),
                gen_proto: todo!(),
                i8_arr: todo!(),
                i16_arr: todo!(),
                i32_arr: todo!(),
                is_finite: todo!(),
                is_nan: todo!(),
                iter: todo!(),
                iter_helper_proto: todo!(),
                json: todo!(),
                map: todo!(),
                map_iter_proto: todo!(),
                math: todo!(),
                number: todo!(),
                object: todo!(),
                parse_float: todo!(),
                parse_int: todo!(),
                promise: todo!(),
                proxy: todo!(),
                range_err: todo!(),
                ref_err: todo!(),
                reflect: todo!(),
                regexp: todo!(),
                regexp_string_iter_proto: todo!(),
                set: todo!(),
                set_iter_proto: todo!(),
                shared_arr_buf: todo!(),
                string: todo!(),
                string_iter_proto: todo!(),
                suppressed_err: todo!(),
                symbol: todo!(),
                syntax_err: todo!(),
                throw_type_err: todo!(),
                typed_arr: todo!(),
                type_err: todo!(),
                u8_arr: todo!(),
                u8_clamp_arr: todo!(),
                u16_arr: todo!(),
                u32_arr: todo!(),
                uri_err: todo!(),
                weak_map: todo!(),
                weak_ref: todo!(),
                weak_set: todo!(),
                wrap_for_valid_iter_proto: todo!(),
            });
        } 

        fn set_default_global_bindings(&self) -> Result<(), Box<dyn Error>> {
            self.global_obj.add_named_val_prop("globalThis", JType::Object(self.global_env.global_this.clone_shallow()), true, false, true)?;
            self.global_obj.add_named_val_prop("Infinity", JType::Number(f64::INFINITY), false, false, false)?;
            self.global_obj.add_named_val_prop("NaN", JType::Number(f64::NAN), false, false, false)?;
            self.global_obj.add_named_val_prop("undefined", JType::Undefined, false, false, false)?;
            self.global_obj.add_named_func("eval", self.intrinsics.as_ref().unwrap().eval.clone_shallow())?;
            self.global_obj.add_named_func("isFinite", self.intrinsics.as_ref().unwrap().is_finite.clone_shallow())?;
            self.global_obj.add_named_func("isNaN", self.intrinsics.as_ref().unwrap().is_nan.clone_shallow())?;
            self.global_obj.add_named_func("parseFloat", self.intrinsics.as_ref().unwrap().parse_float.clone_shallow())?;
            self.global_obj.add_named_func("parseInt", self.intrinsics.as_ref().unwrap().parse_int.clone_shallow())?;
            self.global_obj.add_named_func("decodeURI", self.intrinsics.as_ref().unwrap().decode_uri.clone_shallow())?;
            self.global_obj.add_named_func("decodeURIComponent", self.intrinsics.as_ref().unwrap().decode_uri_comp.clone_shallow())?;
            self.global_obj.add_named_func("encodeURI", self.intrinsics.as_ref().unwrap().encode_uri.clone_shallow())?;
            self.global_obj.add_named_func("encodeURIComponent", self.intrinsics.as_ref().unwrap().encode_uri_comp.clone_shallow())?;
            self.global_obj.add_named_func("AggregateError", self.intrinsics.as_ref().unwrap().agg_err.clone_shallow())?;
            self.global_obj.add_named_func("Array", self.intrinsics.as_ref().unwrap().arr.clone_shallow())?;
            self.global_obj.add_named_func("ArrayBuffer", self.intrinsics.as_ref().unwrap().arr_buf.clone_shallow())?;
            self.global_obj.add_named_func("AsyncDisposableStack", self.intrinsics.as_ref().unwrap().async_disposable_stack.clone_shallow())?;
            self.global_obj.add_named_func("BigInt", self.intrinsics.as_ref().unwrap().bigint.clone_shallow())?;
            self.global_obj.add_named_func("BigInt64Array", self.intrinsics.as_ref().unwrap().bigint_64_arr.clone_shallow())?;
            self.global_obj.add_named_func("BigUInt64Array", self.intrinsics.as_ref().unwrap().biguint_64_arr.clone_shallow())?;
            self.global_obj.add_named_func("Boolean", self.intrinsics.as_ref().unwrap().bool.clone_shallow())?;
            self.global_obj.add_named_func("DataView", self.intrinsics.as_ref().unwrap().data_view.clone_shallow())?;
            self.global_obj.add_named_func("Date", self.intrinsics.as_ref().unwrap().date.clone_shallow())?;
            self.global_obj.add_named_func("DisposableStack", self.intrinsics.as_ref().unwrap().disposable_stack.clone_shallow())?;
            self.global_obj.add_named_func("Error", self.intrinsics.as_ref().unwrap().error.clone_shallow())?;
            self.global_obj.add_named_func("EvalError", self.intrinsics.as_ref().unwrap().eval_error.clone_shallow())?;
            self.global_obj.add_named_func("FinalizationRegistry", self.intrinsics.as_ref().unwrap().final_reg.clone_shallow())?;
            self.global_obj.add_named_func("Float16Array", self.intrinsics.as_ref().unwrap().f16_arr.clone_shallow())?;
            self.global_obj.add_named_func("Float32Array", self.intrinsics.as_ref().unwrap().f32_arr.clone_shallow())?;
            self.global_obj.add_named_func("Float64Array", self.intrinsics.as_ref().unwrap().f64_arr.clone_shallow())?;
            self.global_obj.add_named_func("Function", self.intrinsics.as_ref().unwrap().func.clone_shallow())?;
            self.global_obj.add_named_func("Int8Array", self.intrinsics.as_ref().unwrap().i8_arr.clone_shallow())?;
            self.global_obj.add_named_func("Int16Array", self.intrinsics.as_ref().unwrap().i16_arr.clone_shallow())?;
            self.global_obj.add_named_func("Int32Array", self.intrinsics.as_ref().unwrap().i32_arr.clone_shallow())?;
            self.global_obj.add_named_func("Iterator", self.intrinsics.as_ref().unwrap().iter.clone_shallow())?;
            self.global_obj.add_named_func("Map", self.intrinsics.as_ref().unwrap().map.clone_shallow())?;
            self.global_obj.add_named_func("Number", self.intrinsics.as_ref().unwrap().number.clone_shallow())?;
            self.global_obj.add_named_func("Object", self.intrinsics.as_ref().unwrap().object.clone_shallow())?;
            self.global_obj.add_named_func("Promise", self.intrinsics.as_ref().unwrap().promise.clone_shallow())?;
            self.global_obj.add_named_func("Proxy", self.intrinsics.as_ref().unwrap().proxy.clone_shallow())?;
            self.global_obj.add_named_func("RangeError", self.intrinsics.as_ref().unwrap().range_err.clone_shallow())?;
            self.global_obj.add_named_func("ReferenceError", self.intrinsics.as_ref().unwrap().ref_err.clone_shallow())?;
            self.global_obj.add_named_func("RegExp", self.intrinsics.as_ref().unwrap().regexp.clone_shallow())?;
            self.global_obj.add_named_func("Set", self.intrinsics.as_ref().unwrap().set.clone_shallow())?;
            self.global_obj.add_named_func("SharedArrayBuffer", self.intrinsics.as_ref().unwrap().shared_arr_buf.clone_shallow())?;
            self.global_obj.add_named_func("String", self.intrinsics.as_ref().unwrap().string.clone_shallow())?;
            self.global_obj.add_named_func("SuppressedError", self.intrinsics.as_ref().unwrap().suppressed_err.clone_shallow())?;
            self.global_obj.add_named_func("Symbol", self.intrinsics.as_ref().unwrap().symbol.clone_shallow())?;
            self.global_obj.add_named_func("SyntaxError", self.intrinsics.as_ref().unwrap().syntax_err.clone_shallow())?;
            self.global_obj.add_named_func("TypeError", self.intrinsics.as_ref().unwrap().type_err.clone_shallow())?;
            self.global_obj.add_named_func("Uint8Array", self.intrinsics.as_ref().unwrap().u8_arr.clone_shallow())?;
            self.global_obj.add_named_func("Uint8ClampedArray", self.intrinsics.as_ref().unwrap().u8_clamp_arr.clone_shallow())?;
            self.global_obj.add_named_func("Uint16Array", self.intrinsics.as_ref().unwrap().u16_arr.clone_shallow())?;
            self.global_obj.add_named_func("Uint32Array", self.intrinsics.as_ref().unwrap().u32_arr.clone_shallow())?;
            self.global_obj.add_named_func("URIError", self.intrinsics.as_ref().unwrap().uri_err.clone_shallow())?;
            self.global_obj.add_named_func("WeakMap", self.intrinsics.as_ref().unwrap().weak_map.clone_shallow())?;
            self.global_obj.add_named_func("WeakRef", self.intrinsics.as_ref().unwrap().weak_ref.clone_shallow())?;
            self.global_obj.add_named_func("WeakSet", self.intrinsics.as_ref().unwrap().weak_set.clone_shallow())?;
            self.global_obj.add_named_func("Atomics", self.intrinsics.as_ref().unwrap().atomics.clone_shallow())?;
            self.global_obj.add_named_func("JSON", self.intrinsics.as_ref().unwrap().json.clone_shallow())?;
            self.global_obj.add_named_func("Math", self.intrinsics.as_ref().unwrap().math.clone_shallow())?;
            self.global_obj.add_named_func("Reflect", self.intrinsics.as_ref().unwrap().reflect.clone_shallow())
        }
    }

    impl Object {
        fn add_named_val_prop(&self, key: &str, val: JType, writable: bool, enumerable: bool, configurable: bool) -> Result<(), Box<dyn Error>> {
            self.define_property_or_throw(Key::from_str(key)?, PropDesc {
                kind: crate::types::PropDescType::Data(PropDescData {
                    val: Some(val),
                    writable: Some(writable),
                }),
                enumerable: Some(enumerable),
                configurable: Some(configurable),
            })
        }

        fn add_named_func(&self, key: &str, val: Object) -> Result<(), Box<dyn Error>> {
            self.define_property_or_throw(Key::from_str(key)?, PropDesc {
                kind: crate::types::PropDescType::Data(PropDescData {
                    val: Some(JType::Object(val)),
                    writable: PROPDESC_DATA_DEFAULT.writable,
                }),
                enumerable: PROPDESC_DEFAULT.enumerable,
                configurable: PROPDESC_DEFAULT.configurable,
            })
        }
    }

    // this effectively serves as the core init function for the entire javascript interpreter, but this implementation is still full of holes.
    #[allow(unused_variables)] // TODO: unfinished
    #[allow(unreachable_code)] // TODO: unfinished
    pub fn init_host_defined_realm() -> Result<(), Box<dyn Error>> {
        // TODO: ...everything, but mainly Agents and the CreateIntrinsics function or similar
        let mut realm = Realm {
            agent: todo!(),
            intrinsics: None,
            global_obj: todo!(),
            global_env: todo!(),
            template_map: todo!(),
            loaded_modules: todo!(),
            host_defined: todo!(),
        };
    }
}

#[allow(dead_code)] // TODO: unfinished
pub mod ctx {
    use crate::{CTX_STACK, exec::{env::Environment, priv_env::PrivEnv, realm::Realm}, types::{Object, Todo}};

    #[derive(Debug, Clone)]
    pub enum OrigCode {
        Module(Todo), // TODO: Module Records
        Script(Todo), // TODO: Script Records
        None,
    }

    pub struct ECMAEnvCtx {
        lex: Environment,
        var: Environment,
        private: Option<PrivEnv>,
    }

    pub struct Ctx {
        // unspecified state goes here. will be added to as the implementation continues
        func: Object,
        // unsure if this needs to be inside an Arc yet or not
        realm: Realm,
        script_or_module: OrigCode,
        ecma: Option<ECMAEnvCtx>,
        jgen: Option<Todo>, // TODO: Generators
    }

    pub fn get_active_script_or_module() -> OrigCode {
        let ctx_stack = CTX_STACK.lock();

        if ctx_stack.is_empty() {
            return OrigCode::None;
        }

        let mut found = OrigCode::None;

        for ctx in ctx_stack.iter().rev() {
            if !matches!(ctx.script_or_module, OrigCode::None) {
                found = ctx.script_or_module.clone();
                break;
            }
        }
        found
    }
}