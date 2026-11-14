mod conv {
    use std::error::Error;
    use deadflies_rust_infra::{primitives::IString, structs::{List, ListCore}};
    use num_bigint::{BigInt, ToBigInt};
    use crate::types::{JBigInt, JType, Key, Number, Object};
    
    #[derive(Debug, Clone, Copy)]
    pub enum ConvPref {
        String,
        Number,
        None
    }

    pub enum Numeric {
        Number(f64),
        BigInt(BigInt),
    }

    impl Numeric {
        #[must_use]
        pub fn same_type(&self, other: &Numeric) -> bool {
            matches!((self, other), (Numeric::Number(_), Numeric::Number(_)) |
                (Numeric::BigInt(_), Numeric::BigInt(_)))
        }

        /// Convenience function to unwrap an `f64` from a `Numeric`.
        /// 
        /// # Panics
        /// 
        /// Intentionally panics if the value unwrapped is not an `f64`.
        #[must_use]
        pub fn number(self) -> f64 {
            match self {
                Numeric::Number(x) => x,
                Numeric::BigInt(_) => panic!("Numeric::number: unwrapped a BigInt"),
            }
        }

        /// Convenience function to unwrap a `BigInt` from a `Numeric`.
        /// 
        /// # Panics
        /// 
        /// Intentionally panics if the value unwrapped is not a `BigInt`.
        #[must_use]
        pub fn bigint(self) -> BigInt {
            match self {
                Numeric::Number(_) => panic!("Numeric::bigint: unwrapped an f64"),
                Numeric::BigInt(x) => x,
            }
        }

        #[must_use]
        pub fn is_nan(&self) -> bool {
            match self {
                Numeric::Number(x) => x.is_nan(),
                Numeric::BigInt(_) => false,
            }
        }

        #[must_use]
        pub fn is_neg_inf(&self) -> bool {
            match self {
                Numeric::Number(x) => x.is_infinite() && x.is_sign_negative(),
                Numeric::BigInt(_) => false,
            }
        }

        #[must_use]
        pub fn is_pos_inf(&self) -> bool {
            match self {
                Numeric::Number(x) => x.is_infinite() && x.is_sign_positive(),
                Numeric::BigInt(_) => false,
            }
        }
    }

    impl PartialEq for Numeric {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Numeric::Number(l0), Numeric::Number(r0)) => l0 == r0,
                (Numeric::BigInt(l0), Numeric::BigInt(r0)) => l0 == r0,
                (Numeric::Number(x), Numeric::BigInt(big_y)) => match x.round().to_bigint() {
                    Some(big_x) => big_x == *big_y,
                    None => false,
                },
                (Numeric::BigInt(big_x), Numeric::Number(y)) => match y.round().to_bigint() {
                    Some(big_y) => *big_x == big_y,
                    None => false,
                },
            }
        }
    }

    impl PartialOrd for Numeric {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            match (self, other) {
                (Numeric::Number(x), Numeric::Number(y)) => x.partial_cmp(y),
                (Numeric::Number(x), Numeric::BigInt(big_y)) => x.round().to_bigint()?.partial_cmp(big_y),
                (Numeric::BigInt(big_x), Numeric::Number(y)) => big_x.partial_cmp(&y.round().to_bigint()?),
                (Numeric::BigInt(big_x), Numeric::BigInt(big_y)) => big_x.partial_cmp(big_y),
            }
        }
    }

    impl JType {
        /// The abstract operation `to_primitive` takes argument `self` (a
        /// `JType`) and argument `pref` (a `ConvPref`) and returns a `Result`
        /// containing a `JType` or an error. It converts its input argument to
        /// a non-`Object` type. If an `Object` is capable of converting to
        /// more than one primitive type, it may use `pref` to favour that
        /// type.
        /// 
        /// # Errors
        /// 
        /// This function usually only relays errors from downstream functions,
        /// but may throw a `TypeError` directly if an exotic `Object`'s
        /// definition of `to_primitive` returns an `Object` for some ungodly
        /// reason.
        #[allow(unused_variables)] // TODO: unfinished
        pub fn to_primitive(&self, pref: ConvPref) -> Result<JType, Box<dyn Error>> {
            if let JType::Object(input) = self {
                // TODO: GetMethod
                todo!()
            }
            Ok(self.clone())
        }

        #[must_use]
        pub fn to_bool(&self) -> bool {
            // NORMATIVE OPTIONAL: if we support the is_html_dda internal slot, check for that within an object and and return false if it exists
            match self {
                JType::Bool(b) => *b,
                JType::Undefined | JType::Null => false,
                JType::Number(x) => !(*x == 0.0 || x.is_nan()),
                JType::BigInt(x) => *x == BigInt::ZERO,
                JType::JString(s) => !s.is_empty(),
                _ => true,
            }
        }

        #[allow(clippy::missing_errors_doc)] // we only relay errors from downstream here
        pub fn to_numeric(&self) -> Result<Numeric, Box<dyn Error>> {
            let prim = self.to_primitive(ConvPref::Number)?;

            if let JType::BigInt(prim) = prim {
                Ok(Numeric::BigInt(prim))
            } else {
                // TODO: ToNumber
                todo!()
            }
        }

        /// The abstract operation `to_number` takes argument `self` (a
        /// `JType`) and returns a `Result` containing an `f64` (`Number`) or
        /// an error. It converts `self` to a value of type `f64` (`Number`).
        /// 
        /// # Errors
        /// 
        /// Throws a `TypeError` if used on a `Symbol` or `BigInt`.
        #[allow(unused_variables)] // TODO: Unfinished
        pub fn to_number(&self) -> Result<f64, Box<dyn Error>> {
            match self {
                JType::Number(x) => Ok(*x),
                // TODO: Error Objects
                JType::Symbol(_) | JType::BigInt(_) => todo!(),
                JType::Undefined => todo!(),
                JType::Null => todo!(),
                JType::Bool(_) => todo!(),
                JType::JString(istring) => todo!(),
                JType::Object(object) => todo!(),
            }
        }

        #[allow(clippy::missing_errors_doc)] // we only relay errors from downstream here
        pub fn to_int_or_inf(&self) -> Result<f64, Box<dyn Error>> {
            let num = self.to_number()?;

            if num.is_nan() || num == 0.0 {
                Ok(0.0)
            } else if num.is_infinite() {
                Ok(num)
            } else {
                Ok(num.trunc())
            }
        }

        #[allow(clippy::missing_errors_doc)] // we only relay errors from downstream here
        pub fn to_int32(&self) -> Result<f64, Box<dyn Error>> {
            Ok(self.to_int_or_inf()?.to_fixed_size_int(true, 32))
        }

        #[allow(clippy::missing_errors_doc)] // we only relay errors from downstream here
        pub fn to_uint32(&self) -> Result<f64, Box<dyn Error>> {
            Ok(self.to_int_or_inf()?.to_fixed_size_int(false, 32))
        }

        #[allow(clippy::missing_errors_doc)] // we only relay errors from downstream here
        pub fn to_int16(&self) -> Result<f64, Box<dyn Error>> {
            Ok(self.to_int_or_inf()?.to_fixed_size_int(true, 16))
        }

        #[allow(clippy::missing_errors_doc)] // we only relay errors from downstream here
        pub fn to_uint16(&self) -> Result<f64, Box<dyn Error>> {
            Ok(self.to_int_or_inf()?.to_fixed_size_int(false, 16))
        }

        #[allow(clippy::missing_errors_doc)] // we only relay errors from downstream here
        pub fn to_int8(&self) -> Result<f64, Box<dyn Error>> {
            Ok(self.to_int_or_inf()?.to_fixed_size_int(true, 8))
        }

        #[allow(clippy::missing_errors_doc)] // we only relay errors from downstream here
        pub fn to_uint8(&self) -> Result<f64, Box<dyn Error>> {
            Ok(self.to_int_or_inf()?.to_fixed_size_int(false, 8))
        }

        #[allow(clippy::missing_errors_doc)] // we only relay errors from downstream here
        pub fn to_uint8_clamp(&self) -> Result<f64, Box<dyn Error>> {
            let num = self.to_number()?;

            if num.is_nan() {
                return Ok(0.0)
            }
            
            let clamped = num.clamp(0.0, 255.0);
            let f = clamped.floor();

            if clamped < f + 0.5 {
                Ok(f)
            } else if clamped > f + 0.5 {
                Ok(f + 1.0)
            } else if f % 2.0 == 0.0 {
                Ok(f)
            } else {
                Ok(f + 1.0)
            }
        }

        /// The abstract operation `to_bigint` takes argument `self` (a
        /// `JType`) and returns a `Result` containing a `BigInt` or am error.
        /// It converts `self` to a `BigInt` value, or throws if an implicit
        /// conversion from `f64` (`Number`) would be required.
        /// 
        /// # Errors
        /// 
        /// Throws a `TypeError` when attempting to convert `Undefined`,
        /// `Null`, `f64` (`Number`) values, or `Symbols`. Throws a
        /// `SyntaxError` when failing to convert an `IString`.
        #[allow(unused_variables)] // TODO: unfinished
        pub fn to_bigint(&self) -> Result<BigInt, Box<dyn Error>> {
            match self.to_primitive(ConvPref::Number)? {
                // TODO: Error Objects (Box<dyn Error> is a placeholder)
                JType::Undefined | JType::Null => todo!(),
                JType::Bool(_) => todo!(),
                JType::JString(istring) => todo!(),
                JType::Symbol(symbol) => todo!(),
                JType::Number(_) => todo!(),
                JType::BigInt(big_int) => todo!(),
                JType::Object(object) => todo!(),
            }
        }

        #[allow(clippy::missing_errors_doc)] // we only relay errors from downstream here
        pub fn to_bigint_64(&self) -> Result<BigInt, Box<dyn Error>> {
            Ok(self.to_bigint()?.to_fixed_size_int(true, 64))
        }

        // NOTE: NOT the BigUInt type, just a positive BigInt
        #[allow(clippy::missing_errors_doc)] // we only relay errors from downstream here
        pub fn to_biguint_64(&self) -> Result<BigInt, Box<dyn Error>> {
            Ok(self.to_bigint()?.to_fixed_size_int(false, 64))
        }

        /// The abstract operation `to_string` takes argument `self` (a
        /// `JType`)  and returns a `Result` containing an `IString` or an
        /// error. It converts `self` to a value of type `IString`.
        /// 
        /// # Errors
        /// 
        /// Throws a `TypeError` when attempting to convert a `Symbol`.
        #[allow(unused_variables)] // TODO: unfinished
        pub fn to_string(&self) -> Result<IString, Box<dyn Error>> {
            match self {
                JType::JString(istring) => Ok(istring.clone()),
                // TODO: Error Objects (Box<dyn Error> is a placeholder)
                JType::Symbol(symbol) => todo!(),
                JType::Undefined => todo!(),
                JType::Null => todo!(),
                JType::Bool(_) => todo!(),
                JType::Number(_) => todo!(),
                JType::BigInt(big_int) => todo!(),
                JType::Object(object) => todo!(),
            }
        }

        /// The abstract operation `to_object` takes argument `self` (a
        /// `JType`) and returns a `Result` containing an `Object` or an error.
        /// It converts `self` to a value of type `Object`.
        /// 
        /// # Errors
        /// 
        /// Throws a `TypeError` if `self` is `Undefined` or `Null.`
        #[allow(unused_variables)] // TODO: unfinished
        pub fn to_object(&self) -> Result<Object, Box<dyn Error>> {
            match self {
                // TODO: Error Objects (Box<dyn Error> is a placeholder)
                JType::Undefined | JType::Null => todo!(),
                JType::Bool(_) => todo!(),
                JType::JString(istring) => todo!(),
                JType::Symbol(symbol) => todo!(),
                JType::Number(_) => todo!(),
                JType::BigInt(big_int) => todo!(),
                JType::Object(object) => todo!(),
            }
        }

        #[allow(clippy::missing_panics_doc)] // if we panic this is a bug
        #[allow(clippy::missing_errors_doc)] // we only propagate errors from downstream here
        pub fn to_property_key(&self) -> Result<Key, Box<dyn Error>> {
            let key = self.to_primitive(ConvPref::String)?;

            if let JType::Symbol(key) = key {
                Ok(Key::Symbol(key.clone()))
            } else {
                Ok(Key::String(key.to_string().unwrap()))
            }
        }

        #[allow(clippy::missing_errors_doc)] // we only propagate errors from downstream here
        pub fn to_length(&self) -> Result<f64, Box<dyn Error>> {
            Ok(self.to_int_or_inf()?.clamp(0.0, 2.0_f64.powi(53) - 1.0))
        }

        /// The abstract operation `to_index` takes argument `self` (a `JType`)
        /// and returns a `Result` containing an `f64` or an error. It converts
        /// `self` to an integer and returns that integer if it is non-negative
        /// and corresponds with an integer index. Otherwise, it throws an
        /// exception.
        /// 
        /// # Errors
        /// 
        /// Throws a `RangeError` if `self` isn't in the range 0..=2^53 - 1.
        #[allow(unused_variables)] // TODO: unfinished
        pub fn to_index(&self) -> Result<f64, Box<dyn Error>> {
            let int = self.to_int_or_inf()?;

            // TODO: Error Objects (Box<dyn Error> is a placeholder)
            todo!()
        }

        /// The abstract operation `to_absolute_index` takes arguments `self`
        /// (a `JType`) and `len` (an f64 constrained to a non-negative
        /// integer) and returns a `Result` containing either an f64
        /// constrained to an integer, +∞, or -∞, or an error. It converts
        /// `self` to an integer, +∞, or -∞, interpreting negative values
        /// relative to `len`.
        /// 
        /// # Panics
        /// 
        /// Panics if `len` is less than 0 or if it has a fractional part.
        #[allow(clippy::missing_errors_doc)] // we only propagate errors from downstream here
        pub fn to_absolute_index(&self, len: f64) -> Result<f64, Box<dyn Error>> {
            assert!(len >= 0.0 && len.fract() == 0.0);

            let mut int = self.to_int_or_inf()?;

            if int.is_finite() && int < 0.0 {
                int += len;
            }
            Ok(int)
        }

        #[allow(clippy::missing_errors_doc)] // we only propagate errors from downstream here
        pub fn to_clamped_index(&self, len: f64) -> Result<f64, Box<dyn Error>> {
            Ok(self.to_absolute_index(len)?.clamp(0.0, len))
        }
    }

    impl Object {
        /// The abstract operation `ord_to_prim` takes arguments `self` (an
        /// `Object`) and `pref` (a `ConvPref`) and returns a `Result`
        /// containing a `JType` or an error.
        /// 
        /// # Errors
        /// 
        /// Returns a `TypeError` if the object in question doesn't have either
        /// the valueOf or toString methods, or if they return objects for some
        /// ungodly reason.
        pub fn ord_to_prim(&self, pref: ConvPref) -> Result<JType, Box<dyn Error>> {
            let method_names = {
                let mut ret = List::new();

                if matches!(pref, ConvPref::String) {
                    ret.append(IString::from_str("toString"));
                    ret.append(IString::from_str("valueOf"));
                } else {
                    ret.append(IString::from_str("valueOf"));
                    ret.append(IString::from_str("toString"));
                }
                ret
            };

            #[allow(unused_variables)] // TODO: unfinished
            for name in &*method_names {
                // TODO: Get
                todo!()
            }
            todo!()
        }
    }
}

mod test_compare {
    use std::{error::Error, sync::Arc};
    use crate::{abstract_ops::conv::{ConvPref, Numeric}, types::{ConsFunc, JBigInt, JString, JType, Key, Number, Object, SymNames}};

    impl Object {
        #[allow(clippy::missing_errors_doc)] // we only propagate errors from downstream here
        pub fn is_extensible(&self) -> Result<bool, Box<dyn Error>> {
            self.inner.is_extensible()
        }
    }

    impl JType {
        /// The abstract operation `require_object_coercible` takes argument
        /// `self` (a `JType`) and returns a `Result` containing () or an
        /// error. It throws an error if `self` is a value that cannot be
        /// converted to an `Object` using `to_object`.
        /// 
        /// # Errors
        /// 
        /// Throws a `TypeError` if `self` is `Undefined` or `Null`.
        pub fn require_object_coercible(&self) -> Result<(), Box<dyn Error>> {
            // TODO: Error Objects (Box<dyn Error> is a placeholder)
            todo!()
        }

        #[allow(unused_variables)] // TODO: unfinished
        #[allow(clippy::missing_errors_doc)] // we only propagate errors from downstream here
        pub fn is_array(&self) -> Result<bool, Box<dyn Error>> {
            if let JType::Object(arg) = self {
                // TODO: Array exotic objects
                todo!();
            }
            Ok(false)
        }

        #[must_use]
        pub fn is_callable(&self) -> Option<Object> {
            if let JType::Object(arg) = self {
                if arg.is_callable() {
                    Some(arg.clone_shallow())
                } else {
                    None
                }
            } else {
                None
            }
        }

        #[must_use]
        pub fn is_constructor(&self) -> Option<Arc<ConsFunc>> {
            if let JType::Object(arg) = self {
                arg.is_constructor()
            } else {
                None
            }
        }

        #[allow(clippy::missing_errors_doc)] // we only propagate errors from downstream here
        pub fn is_reg_exp(&self) -> Result<bool, Box<dyn Error>> {
            if let JType::Object(arg) = self {
                let matcher = arg.get(&Key::get_known(SymNames::Match))?;

                if matcher.is_defined() {
                    Ok(matcher.to_bool())
                } else {
                    Ok(arg.has_reg_exp_matcher())
                }
            } else {
                Ok(false)
            }
        }

        #[must_use]
        pub fn same_type(&self, other: &JType) -> bool {
            matches!((self, other) , (JType::Undefined, JType::Undefined) |
                (JType::Null, JType::Null) |
                (JType::Bool(_), JType::Bool(_)) |
                (JType::JString(_), JType::JString(_)) |
                (JType::Symbol(_), JType::Symbol(_)) |
                (JType::Number(_), JType::Number(_)) |
                (JType::BigInt(_), JType::BigInt(_)) |
                (JType::Object(_), JType::Object(_)))
        }

        #[must_use]
        #[allow(clippy::missing_panics_doc)] // if we panic this is a bug
        pub fn same_value(&self, other: &JType) -> bool {
            if !self.same_type(other) {
                false
            } else if let JType::Number(x) = self {
                x.same_value(other.to_number().unwrap())
            } else {
                // TODO: SameValueNonNumber
                todo!()
            }
        }

        #[must_use]
        #[allow(clippy::missing_panics_doc)] // if we panic this is a bug
        pub fn same_value_zero(&self, other: &JType) -> bool {
            if !self.same_type(other) {
                false
            } else if let JType::Number(x) = self {
                x.same_value_zero(other.to_number().unwrap())
            } else {
                // TODO: SameValueNonNumber
                todo!()
            }
        }

        /// The abstract operation `same_value_non_number` takes arguments
        /// `self` (a `JType`) and `other` (a `JType`) and returns a `bool`.
        /// 
        /// # Panics
        /// 
        /// Panics if we received `Number` variants, or if `self` and `other`
        /// are not the same type.
        #[must_use]
        pub fn same_value_non_number(&self, other: &JType) -> bool {
            assert!(self.same_type(other));

            match (self, other) {
                (JType::Number(_), _) | (_, JType::Number(_)) => panic!("called same_value_non_number on number"),
                (JType::Undefined | JType::Null, _) => true,
                (JType::BigInt(x), JType::BigInt(y)) => x.equal(y),
                (JType::JString(x), JType::JString(y)) => x.eq(y),
                (JType::Bool(x), JType::Bool(y)) => x.eq(y),
                (JType::Symbol(x), JType::Symbol(y)) => Arc::ptr_eq(x, y),
                (JType::Object(x), JType::Object(y)) => x == y,
                _ => panic!("same_value_non_number: the front fell off")
            }
        }

        #[allow(clippy::missing_errors_doc)] // we only propagate errors from downstream here
        pub fn is_less_than(&self, other: &JType, left_first: bool) -> Result<Option<bool>, Box<dyn Error>> {
            let (px, py) = if left_first {
                (self.to_primitive(ConvPref::Number)?, other.to_primitive(ConvPref::Number)?)
            } else {
                // NOTE: The order of evaluation needs to be reversed to preserve left to right evaluation.
                let y = other.to_primitive(ConvPref::Number)?;
                let x = self.to_primitive(ConvPref::Number)?;
                (x, y)
            };

            if let JType::JString(ref px) = px && let JType::JString(py) = py {
                // this part of the algorithm is implemented over in infra, so we can cheat
                Ok(Some(px.lt(&py)))
            } else if let JType::BigInt(ref px) = px && let JType::JString(py) = py {
                Ok(py.to_bigint().map(|ny| px.lt(&ny)))
            } else if let JType::JString(ref px) = px && let JType::BigInt(py) = py {
                Ok(px.to_bigint().map(|nx| nx.lt(&py)))
            } else {
                // NOTE: Because px and py are primitive values, evaluation order is not important.
                let nx = px.to_numeric()?;
                let ny = py.to_numeric()?;

                if nx.same_type(&ny) {
                    match nx {
                        Numeric::Number(x) => Ok(Some(x.lt(&ny.number()))),
                        Numeric::BigInt(x) => Ok(Some(x.lt(&ny.bigint()))),
                    }
                } else if nx.is_nan() || ny.is_nan() {
                    Ok(None)
                } else if nx.is_neg_inf() || ny.is_pos_inf() {
                    Ok(Some(true))
                } else if nx.is_pos_inf() || ny.is_neg_inf() {
                    Ok(Some(false))
                } else {
                    Ok(Some(nx.lt(&ny)))
                }
            }
        }

        #[allow(clippy::missing_errors_doc)] // we only propagate errors from downstream here
        pub fn is_loosely_equal(&self, other: &JType) -> Result<bool, Box<dyn Error>> {
            if self.same_type(other) {
                // TODO: IsStrictlyEqual
                todo!()
            }
            todo!()
        }

        #[must_use]
        #[allow(clippy::missing_panics_doc)] // if we panic this is a bug
        pub fn is_strictly_equal(&self, other: &JType) -> bool {
            if !self.same_type(other) {
                false
            } else if let JType::Number(x) = self {
                x.equal(other.to_number().unwrap())
            } else {
                self.same_value_non_number(other)
            }
        }

        #[must_use]
        pub fn is_key(&self) -> bool {
            matches!(self, JType::JString(_) | JType::Symbol(_))
        }
    }
}

mod object_ops {
    use std::{error::Error, sync::Arc};
    use deadflies_rust_infra::{primitives::IString, structs::{List, ListCore}};
    use crate::types::{ClassFieldDef, ConsFunc, Func, JType, Key, Number, Obj, ObjKind, Object, PrivElemKind, PrivName, PrivateElement, PropDesc, PropDescData, PropDescType, Todo};

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum IntegrityLevel {
        Sealed,
        Frozen,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum ElemType {
        All,
        PropertyKey,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum PropSelect {
        Key,
        Value,
        KeyValue,
    }

    impl Object {
        /*
         * MakeBasicObject: skipping, use Object::new() for this.
         * MakeBasicObject deals directly with the semantics of internal slots,
         * which in this implementation correspond to struct fields, which are
         * all hardcoded.
         */
        #[allow(clippy::missing_errors_doc)] // we only propagate errors from downstream here
        pub fn get(&self, key: &Key) -> Result<JType, Box<dyn Error>> {
            self.inner.get(key, &JType::Object(self.clone_shallow()))
        }

        /// The abstract operation `set` takes arguments `self` (an `Object`),
        /// `key` (a `Key`), `val` (a `JType`), and throw (a `bool`) and
        /// returns a `Result` containing `()` or an error. It is used to set
        /// the value of a specific property of an `Object`. `val` is the new
        /// value for the property.
        /// 
        /// # Errors
        /// 
        /// Will throw a `TypeError` when asked to if `throw` is true and `set`
        /// fails on the inner type.
        pub fn set(&self, key: Key, val: JType, throw: bool) -> Result<(), Box<dyn Error>> {
            if !self.inner.clone().set(key, val, &JType::Object(self.clone_shallow()))? && throw {
                // TODO: Error Objects (Box<dyn Error> is a placeholder)
                todo!()
            }
            Ok(())
        }

        #[allow(clippy::missing_errors_doc)] // we only propagate errors from downstream here
        pub fn create_data_property(&self, key: Key, val: JType) -> Result<bool, Box<dyn Error>> {
            self.inner.clone().define_own_property(key, PropDesc {
                kind: PropDescType::Data(PropDescData {
                    val: Some(val),
                    writable: Some(true),
                }),
                enumerable: Some(true),
                configurable: Some(true),
            })
        }

        /// The abstract operation `create_data_property_or_throw` takes
        /// arguments `self` (an `Object`), `key` (a `Key`), and `val` (a
        /// `JType`) and returns a `Result` containing () or an error. It is
        /// used to create a new own property of an `Object`.
        /// 
        /// # Errors
        /// 
        /// It throws a `TypeError` exception if the requested property update
        /// cannot be performed.
        pub fn create_data_property_or_throw(&self, key: Key, val: JType) -> Result<(), Box<dyn Error>> {
            if !self.create_data_property(key, val)? {
                // TODO: Error Objects (Box<dyn Error> is a placeholder)
                todo!()
            }
            Ok(())
        }

        /// The abstract operation
        /// `create_non_enumerable_data_property_or_throw` takes arguments
        /// `self` (an `Object`), `key` (a `Key`), and `val` (a `JType`) and
        /// returns `()`. It is used to create a new non-enumerable own
        /// property of an ordinary `Object`.
        /// 
        /// # Panics
        /// 
        /// Panics if called on an exotic or non-extensible object, or if the
        /// property already exists and is not configurable.
        // long, nasty function name. also, the function never throws.
        #[allow(unused_variables)] // TODO: unfinished
        #[allow(clippy::needless_pass_by_value)] // TODO: unfinished
        pub fn create_non_enumerable_data_property_or_throw(&self, key: Key, val: JType) {
            assert!(self.is_ordinary() && self.is_extensible().is_ok_and(|e| e));

            /*
             * NOTE: technically, the above assert is supposed to check if the
             * object has any non-configurable properties. however, on objects
             * with many properties, doing such a check would be
             * computationally expensive. the unwrap() call below checks if the
             * property exists and is not configurable, but if other non-
             * configurable properties exist we don't have a way to check.
             */
            // TODO: DefinePropertyOrThrow
            todo!()
        }

        /// The abstract operation `define_property_or_throw` takes arguments
        /// `self` (an `Object`), `key` (a `Key`), and `desc` (a `PropDesc`)
        /// and returns a `Result` containing `()` or an error. It is used to
        /// call the `define_own_property` internal method of an `Object` in a
        /// manner that will throw a `TypeError` exception if the requested
        /// property update cannot be performed.
        /// 
        /// # Errors
        /// 
        /// Throws a `TypeError` exception if the requested property update
        /// cannot be performed.
        pub fn define_property_or_throw(&self, key: Key, desc: PropDesc) -> Result<(), Box<dyn Error>> {
            if !self.inner.clone().define_own_property(key, desc)? {
                // TODO: Error Objects (Box<dyn Error> is a placeholder)
                todo!()
            }
            Ok(())
        }

        /// The abstract operation `delete_property_or_throw` takes arguments
        /// `self` (an `Object`) and `key` (a `Key`) and returns a `Result`
        /// containing `()` or an error. It is used to remove a specific own
        /// property of an object.
        /// 
        /// # Errors
        /// 
        /// It throws a `TypeError` exception if the property is not
        /// configurable.
        pub fn delete_property_or_throw(&self, key: Key) -> Result<(), Box<dyn Error>> {
            if !self.inner.clone().delete(key)? {
                // TODO: Error Objects (Box<dyn Error> is a placeholder)
                todo!()
            }
            Ok(())
        }

        #[allow(clippy::missing_errors_doc)] // we only propagate errors from downstream here
        pub fn has_property(&self, key: Key) -> Result<bool, Box<dyn Error>> {
            self.inner.has_property(key)
        }

        #[allow(clippy::missing_errors_doc)] // we only propagate errors from downstream here
        pub fn has_own_property(&self, key: &Key) -> Result<bool, Box<dyn Error>> {
            Ok(self.inner.get_own_property(key)?.is_some())
        }

        #[allow(clippy::missing_errors_doc)] // we only propagate errors from downstream here
        pub fn set_integrity_level(&self, level: IntegrityLevel) -> Result<bool, Box<dyn Error>> {
            if self.inner.prevent_extensions()? {
                let keys = self.inner.own_property_keys()?;

                match level {
                    IntegrityLevel::Sealed => {
                        for key in keys.iter() {
                            self.define_property_or_throw(key.read().clone(), PropDesc {
                                kind: PropDescType::Gen,
                                enumerable: None,
                                configurable: Some(false),
                            })?;
                        }
                    },
                    IntegrityLevel::Frozen => {
                        for key in keys.iter() {
                            if let Some(current_desc) = self.inner.get_own_property(&key.read())? {
                                let desc = {
                                    if current_desc.is_accessor().is_some() {
                                        PropDesc {
                                            kind: PropDescType::Gen,
                                            enumerable: None,
                                            configurable: Some(false),
                                        }
                                    } else {
                                        PropDesc {
                                            kind: PropDescType::Data(PropDescData {
                                                val: None,
                                                writable: Some(false),
                                            }),
                                            enumerable: None,
                                            configurable: Some(false),
                                        }
                                    }
                                };

                                self.define_property_or_throw(key.read().clone(), desc)?;
                            }
                        }
                    },
                }
                Ok(true)
            } else {
                Ok(false)
            }
        }

        #[allow(clippy::missing_panics_doc)] // if we panic this is a bug
        #[allow(clippy::missing_errors_doc)] // we only propagate errors from downstream here
        pub fn test_integrity_level(&self, level: IntegrityLevel) -> Result<bool, Box<dyn Error>> {
            if self.is_extensible()? {
                // NOTE: If the object is extensible, none of its properties are examined.
                Ok(false)
            } else {
                for key in self.inner.own_property_keys()?.iter() {
                    if let Some(curr_desc) = self.inner.get_own_property(&key.read())? {
                        if curr_desc.configurable.unwrap() {
                            return Ok(false);
                        } else if level == IntegrityLevel::Frozen && let Some(data) = curr_desc.is_data() && data.writable.unwrap() {
                            return Ok(false);
                        }
                    }
                }
                Ok(true)
            }
        }

        #[allow(clippy::missing_errors_doc)] // we only propagate errors from downstream here
        #[allow(clippy::cast_possible_truncation)] // intentional here, to_length returns an f64 with no fractional part
        #[allow(clippy::cast_sign_loss)] // intentional here, to_length returns non-negative
        pub fn length_of_array_like(&self) -> Result<usize, Box<dyn Error>> {
            Ok(self.get(&Key::String(IString::from_str("length")))?.to_length()? as usize)
        }

        #[must_use]
        pub fn is_array_like(&self) -> bool {
            self.length_of_array_like().is_ok()
        }

        /// Retreives the constructor that should be used to create new
        /// `Object`s that are derived from `self`. `default_ctor` is the
        /// constructor to use if a constructor %Symbol.species% property
        /// cannot be found starting from `self`.
        /// 
        /// # Errors
        /// 
        /// Throws a `TypeError` if the value associated with `self`'s
        /// "constructor" property is not an `Object` or undefined, or if the
        /// value associated with %Symbol.species% is not undefined, null, or
        /// a constructor.
        pub fn species_constructor(&self, default_ctor: Arc<ConsFunc>) -> Result<Arc<ConsFunc>, Box<dyn Error>> {
            let ctor = self.get(&Key::String(IString::from_str("constructor")))?;

            match ctor {
                JType::Undefined => Ok(default_ctor),
                JType::Object(ctor) => {
                    let species = ctor.get(&Key::get_known(crate::types::SymNames::Species))?;

                    if matches!(species, JType::Undefined | JType::Null) {
                        Ok(default_ctor)
                    } else if let Some(ctor) = species.is_constructor() {
                        Ok(ctor)
                    } else {
                        // TODO: Error Objects (Box<dyn Error> is a placeholder)
                        todo!()
                    }
                },
                _ => {
                    // TODO: Error Objects (Box<dyn Error> is a placeholder)
                    todo!()
                },
            }
        }

        #[allow(clippy::missing_panics_doc)] // if we panic this is a bug
        #[allow(clippy::missing_errors_doc)] // we only propagate errors from downstream here
        pub fn enumerable_own_properties(&self, kind: PropSelect) -> Result<List<JType>, Box<dyn Error>> {
            let mut results = List::new();

            for key in self.inner.own_property_keys()?.iter() {
                if let Key::String(_) = &*key.read() && let Some(desc) = self.inner.get_own_property(&key.read())? && desc.enumerable.unwrap() {
                    // doesn't quite follow the described algorithm from here but it composes better with identical results
                    match kind {
                        PropSelect::Key => results.append(key.read().as_jtype()),
                        PropSelect::Value => results.append(self.get(&key.read())?),
                        PropSelect::KeyValue => todo!(), // TODO: Final CreateArrayFromList impl, current one is unusable because it returns Todo (arrays todo)
                    }
                }
            }
            todo!()
        }

        #[allow(clippy::missing_panics_doc)] // if we panic this is a bug
        #[allow(clippy::missing_errors_doc)] // we only propagate errors from downstream here
        pub fn copy_data_from(&self, src: &JType, excluded_items: &List<JType>) -> Result<(), Box<dyn Error>> {
            if matches!(src, JType::Undefined | JType::Null) {
                Ok(())
            } else {
                let from = src.to_object().unwrap();

                for next_key in from.inner.own_property_keys()?.iter() {
                    let mut excluded = false;

                    for elem in excluded_items.iter() {
                        if elem.read().same_value(&next_key.read().as_jtype()) {
                            excluded = true;
                            break; // not explicitly in standard but at no other point within the loop is excluded ever set back to false, so this should speed things up
                        }
                    }
                    if !excluded && let Some(desc) = from.inner.get_own_property(&next_key.read())? && desc.enumerable.unwrap() {
                        self.create_data_property_or_throw(next_key.read().clone(), from.get(&next_key.read())?).unwrap();
                    }
                }
                Ok(())
            }
        }

        #[must_use]
        pub fn private_element_find(&self, name: &Arc<PrivName>) -> Option<PrivateElement> {
            for entry in self.private_elements.read().iter() {
                if Arc::ptr_eq(&entry.read().key, name) {
                    return Some(entry.read().clone());
                }
            }
            None
        }

        /// Adds a private field to an object.
        /// 
        /// # Errors
        /// 
        /// Throws a `TypeError` if a `PrivateElement` already exists with
        /// `name`.
        #[allow(unused_variables)] // TODO: unfinished
        #[allow(clippy::needless_pass_by_value)] // TODO: unfinished
        pub fn private_field_add(&self, name: Arc<PrivName>, val: JType) -> Result<(), Box<dyn Error>> {
            // TODO: some nebulous place to put host-defined information that we can access from anywhere (static ref to global object behind an rwlock? probably a realm)
            // TODO: global object, realm records
            todo!()
        }

        /// Adds a private method or accessor to an object.
        /// 
        /// # Panics
        /// 
        /// Panics if `method` is a private field.
        /// 
        /// # Errors
        /// 
        /// Throws a `TypeError` if a `PrivateElement` already exists with
        /// `method`.
        #[allow(clippy::needless_pass_by_value)] // TODO: unfinished
        pub fn private_method_or_accessor_add(&self, method: PrivateElement) -> Result<(), Box<dyn Error>> {
            assert!(matches!(method.kind, PrivElemKind::Field(_)));

            // TODO: global object, realm records (see private_field_add)
            todo!()
        }

        #[allow(clippy::missing_errors_doc)] // we don't (yet) return errors here
        pub fn host_ensure_can_add_private_element(&self) -> Result<(), Box<dyn Error>> {
            // we don't really need this yet
            Ok(())
        }

        /// Like `JType::call` but for if the type is already known to be an
        /// `Object`.
        /// 
        /// # Errors
        /// 
        /// Throws a `TypeError` if `self` is not a function.
        pub fn call(&self, this: &JType, args: Option<List<JType>>) -> Result<JType, Box<dyn Error>> {
            let args = args.unwrap_or_default();

            if self.is_callable() {
                self.inner.call(this, args)
            } else {
                // TODO: Error Objects (Box<dyn Error> is a placeholder)
                todo!()
            }
        }

        /// Gets the value associated with a `PrivateElement` contained within
        /// `self`.
        /// 
        /// # Errors
        /// 
        /// Throws a `TypeError` if there is no element named after `name` or
        /// if it's an accessor element with no getter.
        pub fn private_get(&self, name: &Arc<PrivName>) -> Result<JType, Box<dyn Error>> {
            if let Some(entry) = self.private_element_find(name) {
                match entry.kind {
                    PrivElemKind::Field(jtype) | PrivElemKind::Method(jtype) => Ok(jtype.clone()),
                    PrivElemKind::Accessor(priv_elem_accessor) => {
                        if let Some(getter) = priv_elem_accessor.getter {
                            getter.call(&JType::Object(self.clone_shallow()), None)
                        } else {
                            // TODO: Error Objects (Box<dyn Error> is a placeholder)
                            todo!()
                        }
                    },
                }
            } else {
                // TODO: Error Objects (Box<dyn Error> is a placeholder)
                todo!()
            }
        }

        /// Sets the value associated with a `PrivateElement` contained within
        /// `self`.
        /// 
        /// # Errors
        /// 
        /// Throws a `TypeError` if there is no element named after `name`. Use
        /// `Object::private_field_add` if you need to add a new element.
        /// Throws the same error if the element is a method or if it's an
        /// accessor element with no setter.
        #[allow(unused_variables)] // TODO: unfinished
        #[allow(clippy::needless_pass_by_value)] // TODO: unfinished
        pub fn private_set(&self, name: Arc<PrivName>, val: JType) -> Result<(), Box<dyn Error>> {
            if let Some(entry) = self.private_element_find(&name) {
                match entry.kind {
                    PrivElemKind::Method(jtype) => {
                        // TODO: Error Objects (Box<dyn Error> is a placeholder)
                        todo!()
                    },
                    PrivElemKind::Field(jtype) => todo!(),
                    PrivElemKind::Accessor(priv_elem_accessor) => todo!(),
                }
            } else {
                // TODO: Error Objects (Box<dyn Error> is a placeholder)
                todo!()
            }
        }

        #[allow(clippy::missing_errors_doc)] // we only propagate errors from downstream here
        pub fn define_field(&self, field_record: ClassFieldDef) -> Result<(), Box<dyn Error>> {
            let val = match field_record.init {
                Some(init) => init.call(&JType::Object(self.clone_shallow()), None)?,
                None => JType::Undefined,
            };

            match field_record.name {
                crate::types::CFieldDefName::Name(name) => self.private_field_add(name, val),
                crate::types::CFieldDefName::Key(key) => self.create_data_property_or_throw(key, val),
            }
        }

        #[allow(unused_variables)] // TODO: unfinished
        #[allow(clippy::missing_errors_doc)] // we only propagate errors from downstream here
        pub fn initialize_instance_elements(&self, ctor: &Arc<Func>) -> Result<(), Box<dyn Error>> {
            // TODO: Full function object implementation. We currently don't track internal slots.
            todo!()
        }
    }

    pub enum KeyCoercion {
        Property,
        Collection,
    }

    impl JType {
        #[allow(clippy::missing_errors_doc)] // we only propagate errors from downstream here
        pub fn get_v(&self, key: &Key) -> Result<JType, Box<dyn Error>> {
            self.to_object()?.inner.get(key, self)
        }

        /// The abstract operation `get_method` takes arguments `self` (a
        /// `JType`) and `key` (a `Key`) and returns a Result containing either
        /// a `Func` `Object` or `None`, or an error. It is used to get the
        /// value of a specific property of an `JType` when the value of the
        /// property is expected to be a function.
        /// 
        /// # Errors
        /// 
        /// Throws a `TypeError` if the property matching `key` is not a
        /// function.
        #[allow(clippy::missing_panics_doc)] // if we panic this is a bug
        pub fn get_method(&self, key: &Key) -> Result<Option<Object>, Box<dyn Error>> {
            let func = self.get_v(key)?;

            match func {
                JType::Undefined | JType::Null => Ok(None),
                _ => if let Some(obj) = func.is_callable() {
                    Ok(Some(obj))
                } else {
                    // TODO: Error Objects (Box<dyn Error> is a placeholder)
                    todo!()
                },
            }
        }

        /// The abstract operation `call` takes arguments `self` (a `JType`)
        /// and `this` (a `JType`) and optional argument `args` (a
        /// `List<JType>`) and returns a Result containing a `JType` or an
        /// error. It is used to call the `call` internal method of a `Func`
        /// `Object`. `self` is the `Func` `Object`, `this` is a `JType` that
        /// is the this value of the `call`, and `args` is the value passed to
        /// the corresponding argument of the internal method. If `args` is not
        /// present, a new empty `List` is used as its value.
        /// 
        /// # Errors
        /// 
        /// Throws a `TypeError` if `self` is not a function.
        pub fn call(&self, this: &JType, args: Option<List<JType>>) -> Result<JType, Box<dyn Error>> {
            let args = args.unwrap_or_default();

            if let Some(obj) = self.is_callable() {
                obj.inner.call(this, args)
            } else {
                // TODO: Error Objects (Box<dyn Error> is a placeholder)
                todo!()
            }
        }

        /// Creates a `List` from an array-like `Object`.
        /// 
        /// # Errors
        /// 
        /// Throws a `TypeError` if `self` is not an Object or if the caller has
        /// requested valid property keys only and an indexed element is not a
        /// valid `Key`.
        #[allow(clippy::cast_precision_loss)] // the cast in question shouldn't ever cause problems, as an object that large would already have caused an OOM
        pub fn create_list_from_array_like(&self, valid_types: Option<ElemType>) -> Result<List<JType>, Box<dyn Error>> {
            let valid_types = valid_types.unwrap_or(ElemType::All);

            if let JType::Object(obj) = self {
                let len = obj.length_of_array_like()?;
                let mut list = List::new();
                let mut i = 0;

                while i < len {
                    let next = obj.get(&Key::String((i as f64).to_istring(10)))?;

                    if valid_types == ElemType::PropertyKey && !next.is_key() {
                        // TODO: Error Objects (Box<dyn Error> is a placeholder)
                        todo!()
                    }
                    list.append(next);
                    i += 1;
                }
                todo!()
            } else {
                // TODO: Error Objects (Box<dyn Error> is a placeholder)
                todo!()
            }
        }

        #[allow(clippy::missing_errors_doc)] // we only propagate errors from downstream here
        pub fn invoke(&self, key: &Key, args: Option<List<JType>>) -> Result<JType, Box<dyn Error>> {
            self.get_v(key)?.call(self, Some(args.unwrap_or_default()))
        }

        /// Implements the default algorithm for determining if `instance`
        /// inherits from the instance object inheritance path provided by
        /// `self`.
        /// 
        /// # Errors
        /// 
        /// Throws a `TypeError` if `self`'s prototype is not an Object. That
        /// said, if this happens, something is probably very very wrong,
        /// unless `self` is exotic.
        #[allow(unused_variables)] // TODO: unfinished
        pub fn ordinary_has_instance(&self, instance: &JType) -> Result<bool, Box<dyn Error>> {
            if let Some(obj) = self.is_callable() {
                if obj.get_kind() == ObjKind::BoundFunc {
                    // TODO: Reflection, and BoundFunction objects
                    todo!()
                }
                todo!()
            } else {
                Ok(false)
            }
        }

        /// Groups the items of a collection according to a specified callback.
        /// 
        /// # Errors
        /// 
        /// Throws a `TypeError` if `callback` is not a function, or if the
        /// collection has >= 2^53-1 items.
        #[allow(unused_variables)] // TODO: unfinished
        #[allow(unused_mut)] // TODO: unfinished
        pub fn group_by(&self, callback: &JType, key_coercion: &KeyCoercion) -> Result<List<Group>, Box<dyn Error>> {
            self.require_object_coercible()?;
            if let Some(callback) = callback.is_callable() {
                let mut groups: List<JType> = List::new(); // FIXME: shouldn't need type annotation after implementation
                // TODO: GetIterator
                todo!()
            } else {
                // TODO: Error Objects (Box<dyn Error> is a placeholder)
                todo!()
            }
        }

        /// Returns `self` if it is an `Object`, or creates an empty `Object`
        /// if undefined.
        /// 
        /// # Errors
        /// 
        /// Throws a `TypeError` if `self` is anything other than an `Object`
        /// or undefined.
        #[allow(unused_variables)] // TODO: unfinished
        pub fn get_options_object(&self) -> Result<Object, Box<dyn Error>> {
            match self {
                JType::Undefined => {
                    // TODO: OrdinaryObjectCreate
                    todo!()
                },
                JType::Object(object) => todo!(),
                _ => todo!()
            }
        }

        /// A setter that ignores prototype properties.
        /// 
        /// # Errors
        /// 
        /// Throws a `TypeError` if `self` is not an `Object`, or if `self` and
        /// `home` are the same `Object`.
        #[allow(unused_variables)] // TODO: unfinished
        #[allow(clippy::needless_pass_by_value)] // TODO: unfinished
        pub fn setter_that_ignores_prototype_properties(&self, home: &Object, key: Key, val: JType) -> Result<(), Box<dyn Error>> {
            if let JType::Object(this) = self {
                if this == home {
                    // NOTE: Throwing here emulates assignment to a non-writable data property on the home object in strict mode code.
                    // TODO: Error Objects (Box<dyn Error>) is a placeholder
                    todo!()
                } else {
                    todo!()
                }
            } else {
                // TODO: Error Objects (Box<dyn Error>) is a placeholder
                todo!()
            }
        }
    }

    impl ConsFunc {
        #[allow(clippy::missing_errors_doc)] // we only propagate errors from downstream here
        pub fn construct(self: Arc<Self>, args: Option<List<JType>>, new_target: Option<Arc<ConsFunc>>) -> Result<Object, Box<dyn Error>> {
            self.construct_int(args.unwrap_or_default(), new_target.unwrap_or(self.clone()).make_object())
        }
    }

    impl Func {
        // TODO: return a Realm Record once they exist
        #[allow(clippy::missing_errors_doc)] // we only propagate errors from downstream here
        pub fn get_realm(&self) -> Result<Todo, Box<dyn Error>> {
            // TODO: Full function object implementation. We currently don't track internal slots or distinguish between normal and exotic function objects.
            todo!()
        }
    }

    // TODO: needs to be moved to an Array impl (once Arrays exist) and turned into a method named from_list
    #[must_use]
    #[allow(unused_variables)] // TODO: unfinished
    #[allow(dead_code)] // TODO: unfinished
    pub fn create_array_from_list(elems: &List<JType>) -> Todo {
        // TODO: ArrayCreate
        todo!()
    }

    #[allow(dead_code)] // TODO: unfinished
    pub struct Group {
        key: JType,
        elements: List<JType>,
    }

    #[allow(dead_code)] // TODO: unfinished
    pub fn add_value_to_keyed_group(groups: &mut List<Group>, key: JType, val: JType) {
        for group in groups.iter_mut() {
            if group.read().key.same_type(&key) {
                // technically, we should check that only one element in groups has this key, but on large lists such a check would be computationally expensive.
                group.write().elements.append(val);
                return;
            }
        }
        groups.append(Group {
            key,
            elements: {
                let mut elements = List::new();

                elements.append(val);
                elements
            },
        });
    }
}

mod iterator_ops {
    use std::{error::Error, sync::Arc};
    use deadflies_rust_infra::{primitives::IString, structs::{List, ListCore}};
    use crate::types::{Func, JType, Key, Obj, Object};

    #[allow(dead_code)] // TODO: unfinished
    pub struct IterRec {
        iterator: Object,
        next_method: JType,
        done: bool,
    }

    impl IterRec {
        // TODO: FIXME ASAP: look at locking these things, should probably provide synchronous access
        pub fn next(&mut self, val: Option<JType>) -> Result<Object, Box<dyn Error>> {
            let result = self.next_method.call(&JType::Object(self.iterator.clone_shallow()), val.map(|v| {
                let mut ret = List::new();

                ret.append(v);
                ret
            }));

            match result {
                Err(e) => {
                    self.done = true;
                    Err(e)
                },
                Ok(result) => {
                    if let JType::Object(result) = result {
                        Ok(result)
                    } else {
                        self.done = true;
                        // TODO: Error Objects (Box<dyn Error>) is a placeholder
                        todo!()
                    }
                },
            }
        }
    }

    impl Object {
        #[allow(clippy::missing_errors_doc)] // we only propagate errors from downstream here
        pub fn get_iterator_direct(&self) -> Result<IterRec, Box<dyn Error>> {
            Ok(IterRec {
                iterator: self.clone_shallow(),
                next_method: self.get(&Key::String(IString::from_str("next")))?,
                done: false,
            })
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum IterKind {
        Sync,
        Async,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum PrimitiveHandling {
        IterateStrings,
        Reject,
    }

    impl JType {
        /// Gets an iterator from the result of a `method` call.
        /// 
        /// # Errors
        /// 
        /// Throws a `TypeError` if the return value from the call is not an
        /// `Object`.
        pub fn get_iterator_from_method(&self, method: &Arc<Func>) -> Result<IterRec, Box<dyn Error>> {
            if let JType::Object(iterator) = method.call(self, List::new())? {
                iterator.get_iterator_direct()
            } else {
                // TODO: Error Objects (Box<dyn Error> is a placeholder)
                todo!()
            }
        }

        /// Gets an iterator from a `JType`.
        /// 
        /// # Errors
        /// 
        /// Throws a `TypeError` if `kind` is `IterKind::Async` and neither
        /// asyncIterator nor iterator methods are defined, or if `kind` is
        /// `IterKind::Sync` and no iterator method is defined.
        #[allow(unused_variables)] // TODO: unfinished
        pub fn get_iterator(&self, kind: IterKind) -> Result<IterRec, Box<dyn Error>> {
            let method = match kind {
                IterKind::Async => {
                    if let Some(method) = self.get_method(&Key::get_known(crate::types::SymNames::AsyncIterator))? {
                        method
                    } else if let Some(sync_method) = self.get_method(&Key::get_known(crate::types::SymNames::Iterator))? {
                        // TODO: CreateAsyncFromSyncIterator
                        todo!()
                    } else {
                        // TODO: Error Objects (Box<dyn Error> is a placeholder)
                        todo!()
                    }
                },
                IterKind::Sync => todo!(),
            };

            todo!()
        }

        /// Gets a flattenable iterator from a `JType`.
        /// 
        /// # Errors
        /// 
        /// Throws a `TypeError` if `self` is not an `Object` and `handling` is
        /// `PrimitiveHandling::Reject`, or if `handling` is
        /// `PrimitiveHandling::IterateStrings` and `self` is not a string.
        /// Will still throw a `TypeError` if `self` can't be converted into an
        /// iterator from there.
        pub fn get_iterator_flattenable(&self, handling: PrimitiveHandling) -> Result<IterRec, Box<dyn Error>> {
            if matches!(self, JType::Object(_)) {
                match handling {
                    PrimitiveHandling::Reject => {
                        // TODO: Error Objects (Box<dyn Error>) is a placeholder
                        todo!()
                    },
                    PrimitiveHandling::IterateStrings => todo!(),
                }
            }
            todo!()
        }
    }
}