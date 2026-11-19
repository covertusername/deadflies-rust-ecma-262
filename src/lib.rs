#![warn(clippy::pedantic)]

mod util;
mod abstract_ops;
mod syntax_directed_ops;

pub mod types {
    use deadflies_rust_infra::{primitives::IString, structs::{ArcRw, List, Map}};
    use std::{error::Error, ops::{Add, Div, Mul, Neg, Not, Rem, Shl, Sub}, str::FromStr, sync::{Arc, Weak, atomic::{AtomicBool, AtomicU64, Ordering}}};
    use parking_lot::{Mutex, RwLock};
    use num_bigint::BigInt;
    use astro_float::{BigFloat, Consts, RoundingMode};
    use crate::util::{bf_to_i64_unchecked, bf_to_usize_unchecked, sane_round};

    static KNOWN_SYMBOLS: Mutex<Map<SymNames, Arc<Symbol>>> = Mutex::new(Map::new());
    #[allow(dead_code)] // TODO: for future use, not filled in anywhere yet, still need to implement what goes here
    static KNOWN_OBJECTS: Mutex<Map<&str, Object>> = Mutex::new(Map::new());

    // in standard but not used yet
    #[allow(dead_code)]
    const ASCII_WORD_CHARS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_";
    const _DEBUG: bool = true;

    #[derive(Debug)]
    pub enum Todo {}

    pub trait JString {
        fn index_of(&self, search_val: IString, from_i: usize) -> Option<usize>;

        fn last_index_of(&self, search_val: IString, from_i: usize) -> Option<usize>;

        fn to_number(&self) -> f64;

        fn to_bigint(&self) -> Option<BigInt>;

        fn canonical_numeric_index(&self) -> Option<f64>;

        fn is_canonical_numeric(&self) -> bool;

        fn is_string_well_formed_unicode(&self) -> bool;
    }

    impl JString for IString {
        fn index_of(&self, search_val: IString, from_i: usize) -> Option<usize> {
            let len = self.len();
            let search_len = search_val.len();
            let mut i = from_i;

            if search_len == 0 && from_i <= len {
                return Some(from_i)
            }
            while i <= len - search_len {
                let cand = self.cu_substr_len(i, search_len);

                if cand == search_val {
                    return Some(i)
                }
                i += 1;
            }
            None
        }
        
        fn last_index_of(&self, search_val: IString, from_i: usize) -> Option<usize> {
            let len = self.len();
            let search_len = search_val.len();
            let mut i = len - search_len;

            assert!(from_i + search_len <= len);

            loop {
                let cand = self.cu_substr_len(i, search_len);

                if cand == search_val {
                    return Some(i)
                }
                if i == 0 {
                    break
                }
                i -= 1;
            }
            None
        }

        fn to_number(&self) -> f64 {
            // TODO: ParseText. old parsing code is preserved in a seperate file, but it uses older APIs and needs to be converted.
            todo!()
        }
        
        fn to_bigint(&self) -> Option<BigInt> {
            // TODO: ParseText, BigInt parsers
            todo!()
        }

        fn canonical_numeric_index(&self) -> Option<f64> {
            if *self == IString::from_str("-0") {
                return Some(-0.0);
            }

            let n = self.to_number();

            if n.to_istring(10) == *self {
                Some(n)
            } else {
                None
            }
        }

        fn is_canonical_numeric(&self) -> bool {
            self.canonical_numeric_index().is_some()
        }

        // already implemented over in the infra crate without javascript clutter. just going to piggyback on that
        fn is_string_well_formed_unicode(&self) -> bool {
            self.is_scalar()
        }
    }

    /*
     * Symbol tweaked from https://users.rust-lang.org/t/does-rust-have-something-like-symbol-in-javascript/128209/5
     */

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
    pub enum SymNames {
        AsyncDispose,
        AsyncIterator,
        Dispose,
        HasInstance,
        IsConcatSpreadable,
        Iterator,
        Match,
        MatchAll,
        Replace,
        Search,
        Species,
        Split,
        ToPrimitive,
        ToStringTag,
        Unscopables,
    }
    
    #[derive(Debug)]
    pub struct Symbol {
        me: Weak<Symbol>,
        // TODO: utility of the id is questionable since we implement eq() based on the address of the Weak references. may remove later
        #[allow(dead_code)]
        id: u64,
        description: Option<IString>,
    }

    impl Symbol {
        #[must_use]
        pub fn new(desc: &Option<IString>) -> Arc<Self> {
            static SYMBOL_COUNTER: AtomicU64 = AtomicU64::new(0);

            Arc::new_cyclic(|me |
                Symbol {
                    me: me.clone(),
                    id: SYMBOL_COUNTER.fetch_add(1, Ordering::Relaxed),
                    description: desc.clone(),
                }
            )
        }

        #[must_use]
        pub fn description(self: Arc<Self>) -> Option<IString> {
            self.description.clone()
        }

        #[must_use]
        #[allow(clippy::missing_panics_doc)] // if we panic this is a bug
        pub fn get_known(key: SymNames) -> Arc<Symbol> {
            Symbol::init_known_symbols();
            KNOWN_SYMBOLS.lock().get(&key).unwrap()
        }

        /*
         * known symbols map to known object property values, but none of these are
         * implemented yet. can still create the symbols themselves for now
         */
        fn init_known_symbols() {
            static INIT_COMPLETE: AtomicBool = AtomicBool::new(false);

            if INIT_COMPLETE.load(Ordering::Relaxed) {
                return;
            }

            let mut syms = KNOWN_SYMBOLS.lock();
            let names = [
                ("Symbol.asyncDispose", SymNames::AsyncDispose),
                ("Symbol.asyncIterator", SymNames::AsyncIterator),
                ("Symbol.dispose", SymNames::Dispose),
                ("Symbol.hasInstance", SymNames::HasInstance),
                ("Symbol.isConcatSpreadable", SymNames::IsConcatSpreadable),
                ("Symbol.iterator", SymNames::Iterator),
                ("Symbol.match", SymNames::Match),
                ("Symbol.matchAll", SymNames::MatchAll),
                ("Symbol.replace", SymNames::Replace),
                ("Symbol.search", SymNames::Search),
                ("Symbol.species", SymNames::Species),
                ("Symbol.split", SymNames::Split),
                ("Symbol.toPrimitive", SymNames::ToPrimitive),
                ("Symbol.toStringTag", SymNames::ToStringTag),
                ("Symbol.unscopables", SymNames::Unscopables),
            ];

            for s in names {
                syms.set(s.1, Symbol::new(&Some(IString::from_str(s.0))));
            }
            INIT_COMPLETE.store(true, Ordering::Relaxed);
        }
    }

    impl PartialEq for Symbol {
        fn eq(&self, other: &Self) -> bool {
            Weak::ptr_eq(&self.me, &other.me)
        }
    }

    pub enum BitOps {
        Amp,
        Caret,
        Pipe,
    }

    pub trait Number {
        fn to_fixed_size_int(&self, signed: bool, bit_width: u32) -> f64;

        fn exponent(&self) -> i16;

        fn mantissa(&self) -> u64;

        fn unary_minus(&self) -> f64;

        fn bitwise_not(&self) -> f64;

        fn exponentiate(&self, exp: f64) -> f64;

        fn multiply(&self, rhs: f64) -> f64;

        fn divide(&self, rhs: f64) -> f64;

        fn remainder(&self, rhs: f64) -> f64;

        fn subtract(&self, rhs: f64) -> f64;

        fn left_shift(&self, rhs: f64) -> f64;

        fn signed_shr(&self, _rhs: f64) -> f64;

        fn unsigned_shr(&self, _rhs: f64) -> f64;

        fn less_than(&self, rhs: f64) -> Option<bool>;

        fn equal(&self, rhs: f64) -> bool;

        fn same_value(&self, other: f64) -> bool;

        fn same_value_zero(&self, other: f64) -> bool;

        fn bitwise_op(&self, _rhs: f64, _op: BitOps) -> f64;

        fn bitwise_and(&self, rhs: f64) -> f64;

        fn bitwise_xor(&self, rhs: f64) -> f64;

        fn bitwise_or(&self, rhs: f64) -> f64;

        fn to_istring(&self, radix: i32) -> IString;
    }

    impl Number for f64 {
        #[allow(clippy::cast_possible_wrap)] // checked via assert
        #[allow(clippy::cast_possible_truncation)] // intentional here
        fn to_fixed_size_int(&self, signed: bool, bit_width: u32) -> f64 {
            assert!(i32::try_from(bit_width).is_ok());

            if self.is_infinite() {
                0.0
            } else if self.fract() == 0.0 {
                let two_pow = 2.0_f64.powi(bit_width as i32);
                let mut fixed_int = self.rem(two_pow);

                // NOTE: does not change the two's complement representation of fixed_int
                if signed && fixed_int >= 2.0_f64.powi((bit_width as i32) - 1) {
                    fixed_int -= two_pow;
                }
                fixed_int
            } else {
                panic!("to_fixed_size_int: received fractional number or NaN")
            }
        }

        // guts of exponent() and mantissa() stolen from integer_decode in num-traits
        fn exponent(&self) -> i16 {
            (((self.to_bits() >> 52) & 0x7FF) as i16) - 1023 + 52
        }

        fn mantissa(&self) -> u64 {
            if ((self.to_bits() >> 52) & 0x7ff) as i16 == 0 {
                (self.to_bits() & 0xf_ffff_ffff_ffff) << 1
            } else {
                (self.to_bits() & 0xf_ffff_ffff_ffff) | 0x0010_0000_0000_0000
            }
        }

        fn unary_minus(&self) -> f64 {
            if self.is_nan() {
                f64::NAN
            }
            else {
                self.neg()
            }
        }

        fn bitwise_not(&self) -> f64 {
            // TODO: ToInt32
            todo!()
        }

        #[allow(clippy::float_cmp)] // comp'ed floats have no fractal part so are precise
        #[allow(clippy::missing_panics_doc)] // if we panic this is a bug
        fn exponentiate(&self, exp: Self) -> Self {
            if exp.is_nan() {
                return f64::NAN
            }
            if exp == 0.0 {
                return 1.0
            }
            if self.is_nan() {
                return f64::NAN
            }
            if self.is_infinite() && self.is_sign_positive() {
                if exp > 0.0 {
                    return f64::INFINITY
                }
                return 0.0
            }
            if self.is_infinite() && self.is_sign_negative() {
                if exp > 0.0 {
                    if exp.fract() == 0.0 && exp % 2.0 == 1.0 {
                        return f64::NEG_INFINITY
                    }
                    return f64::INFINITY
                }
                if exp.fract() == 0.0 && exp % 2.0 == 1.0 {
                    return -0.0
                }
                return 0.0
            }
            if *self == 0.0 && self.is_sign_positive() {
                if exp > 0.0 {
                    return 0.0
                }
                return f64::INFINITY
            }
            if *self == 0.0 && self.is_sign_negative() {
                if exp > 0.0 {
                    if exp.fract() == 0.0 && exp % 2.0 == 1.0 {
                        return -0.0
                    }
                    return 0.0
                }
                if exp.fract() == 0.0 && exp % 2.0 == 1.0 {
                    return f64::NEG_INFINITY
                }
                return f64::INFINITY
            }

            assert!(self.is_finite() && *self != 0.0);

            if exp == f64::INFINITY {
                if self.abs() > 1.0 {
                    return f64::INFINITY
                }
                if self.abs() == 1.0 {
                    return f64::NAN
                }
                return 0.0
            }
            if exp == f64::NEG_INFINITY {
                if self.abs() > 1.0 {
                    return 0.0
                }
                if self.abs() == 1.0 {
                    return f64::NAN
                }
                return f64::INFINITY
            }

            assert!(exp.is_finite() && exp != 0.0);

            if self < &-0.0 && exp.fract() != 0.0 {
                return f64::NAN
            }
            self.powf(exp)
        }

        fn multiply(&self, rhs: f64) -> f64 {
            self.mul(rhs)
        }

        fn divide(&self, rhs: f64) -> f64 {
            self.div(rhs)
        }

        fn remainder(&self, rhs: f64) -> f64 {
            if self.is_nan() || rhs.is_nan() {
                return f64::NAN
            }
            if self.is_infinite() {
                return f64::NAN
            }
            if rhs.is_infinite() {
                return *self
            }
            if rhs == 0.0 {
                return f64::NAN
            }
            if *self == 0.0 {
                return *self
            }

            assert!(self.is_finite() && *self != 0.0 && rhs.is_finite() && rhs != 0.0);

            let quot = self / rhs;
            let trunc_quot = quot.trunc();
            let rem = *self - (rhs * trunc_quot);

            if rem == 0.0 && *self <= -0.0 {
                return -0.0
            }
            rem
        }

        fn subtract(&self, rhs: f64) -> f64 {
            self.add(rhs.neg())
        }

        fn left_shift(&self, _rhs: f64) -> f64 {
            // TODO: ToInt32
            todo!()
        }

        fn signed_shr(&self, _rhs: Self) -> Self {
            // TODO: ToInt32
            todo!()
        }

        fn unsigned_shr(&self, _rhs: Self) -> Self {
            // TODO: ToUint32
            todo!()
        }

        #[allow(clippy::missing_panics_doc)] // if we panic this is a bug
        #[allow(clippy::float_cmp)] // we are looking for exactness here
        fn less_than(&self, rhs: Self) -> Option<bool> {
            if self.is_nan() || rhs.is_nan() {
                return None
            }
            if *self == rhs {
                return Some(true)
            }
            if *self == f64::INFINITY {
                return Some(false)
            }
            if rhs == f64::INFINITY {
                return Some(true)
            }
            if rhs == f64::NEG_INFINITY {
                return Some(false)
            }
            if *self == f64::NEG_INFINITY {
                return Some(true)
            }

            assert!(self.is_finite() && rhs.is_finite());

            Some(*self < rhs)
        }

        #[allow(clippy::float_cmp)] // we are looking for exactness here
        fn equal(&self, rhs: Self) -> bool {
            if self.is_nan() || rhs.is_nan() {
                return false
            }
            *self == rhs
        }

        #[allow(clippy::float_cmp)]
        fn same_value(&self, other: Self) -> bool {
            if self.is_nan() && other.is_nan() {
                return true
            }
            if (*self == 0.0 && self.is_sign_positive() && other == 0.0 && other.is_sign_negative()) ||
                (*self == 0.0 && self.is_sign_negative() && other == 0.0 && other.is_sign_positive()) {
                return false
            }
            *self == other
        }

        #[allow(clippy::float_cmp)]
        fn same_value_zero(&self, other: Self) -> bool {
            if self.is_nan() && other.is_nan() {
                return true
            }
            *self == other
        }

        fn bitwise_op(&self, _rhs: Self, _op: BitOps) -> Self {
            // TODO: ToInt32
            todo!()
        }

        fn bitwise_and(&self, rhs: f64) -> f64 {
            self.bitwise_op(rhs, BitOps::Amp)
        }

        fn bitwise_xor(&self, rhs: f64) -> f64 {
            self.bitwise_op(rhs, BitOps::Caret)
        }

        fn bitwise_or(&self, rhs: f64) -> f64 {
            self.bitwise_op(rhs, BitOps::Pipe)
        }

        /// From the ECMAScript specification, with variable names changed to
        /// match written code:
        /// 
        /// The abstract operation `Number::to_string` takes arguments `self` (a
        /// Number) and `radix` (an integer in the inclusive interval from 2 to
        /// 36) and returns a String. It represents `self` as a String using a
        /// positional numeral system with radix `radix`. The digits used in
        /// the representation of a number using radix r are taken from the
        /// first r code units of "0123456789abcdefghijklmnopqrstuvwxyz" in
        /// order. The representation of numbers with magnitude greater than or
        /// equal to 1𝔽 never includes leading zeroes.
        /// 
        /// # Panics
        /// 
        /// Panics if the provided radix is not in the inclusive interval from
        /// 2 to 36, or if we ran out of memory.
        // DISCLAIMER: implementation is very naive and uses bigfloats to derive n, k, and s. i couldn't figure out a better way to implement it.
        #[allow(clippy::cast_possible_truncation)] // truncation is required to make the algorithm work, especially when mixing ints and floats
        #[allow(clippy::cast_sign_loss)] // not possible for negative values to make it to the sign loss
        #[allow(clippy::cast_lossless)] // the types here are never going to change
        fn to_istring(&self, radix: i32) -> IString {
            const DIGITS: &[u16] = &[0x0030, 0x0031, 0x0032, 0x0033, 0x0034, 0x0035, 0x0036, 0x0037, 0x0038, 0x0039, 0x0061, 0x0062, 0x0063, 0x0064, 0x0065, 0x0066, 0x0067, 0x0068,
                                     0x0069, 0x006A, 0x006B, 0x006C, 0x006D, 0x006E, 0x006F, 0x0070, 0x0071, 0x0072, 0x0073, 0x0074, 0x0075, 0x0076, 0x0077, 0x0078, 0x0079, 0x007A];

            assert!((2..=36).contains(&radix));

            if self.is_nan() { return IString::from_str("NaN") }
            if *self == 0.0 { return IString::from_str("0") }
            if *self < -0.0 { return IString::concat(&[IString::from_str("-"), self.neg().to_istring(radix)], &IString::from_str("")) }
            if *self == f64::INFINITY { return IString::from_str("Infinity") }

            let mut k = 1;

            while (radix as u64).pow(k) <= self.mantissa() { k += 1; }
            
            /*
             * if we run out of memory we're fucked anyway so just die on
             * error. we don't need cc anywhere else (yet.) we will probably
             * need to lug some sort of state around for javascript execution
             * later anyway, so if we need cc elsewhere we can init it once and
             * put it in that
             */
            let mut cc = Consts::new().unwrap();
            let big_self = BigFloat::from(*self);
            let big_radix = BigFloat::from(radix);
            let big_k = BigFloat::from(k); // not the cereal
            let starting_n = big_self
                                .div(&BigFloat::from(self.mantissa()), 128, RoundingMode::ToEven)
                                .log(&big_radix, 64, RoundingMode::None, &mut cc) // do NOT set this to ToEven, causes an infinite loop in some cases
                                .add_full_prec(&big_k);
            let (n, s) = {
                let tried_s_floor = big_self.
                                            div(&big_radix.pow(
                                                &starting_n.floor().sub_full_prec(&big_k), 1088, RoundingMode::ToEven, &mut cc
                                                ), 128, RoundingMode::ToEven
                                            );
                let tried_s_ceil = big_self.
                                            div(&big_radix.pow(
                                                &starting_n.ceil().sub_full_prec(&big_k), 1088, RoundingMode::ToEven, &mut cc
                                                ), 128, RoundingMode::ToEven
                                            );
                if tried_s_floor >= big_radix.pow(&big_k.sub_full_prec(&BigFloat::from(1)), 64, RoundingMode::ToEven, &mut cc) && 
                    tried_s_floor < big_radix.pow(&big_k, 64, RoundingMode::None, &mut cc) {
                        (starting_n.floor(), sane_round(&tried_s_floor))
                    }
                else if tried_s_ceil >= big_radix.pow(&big_k.sub_full_prec(&BigFloat::from(1)), 64, RoundingMode::ToEven, &mut cc) && 
                    tried_s_ceil < big_radix.pow(&big_k, 64, RoundingMode::None, &mut cc) {
                        (starting_n.ceil(), sane_round(&tried_s_ceil))
                }
                else {
                    panic!("neither the floor nor ceil of starting_n are correct")
                }
            };
            let mut chars_rev: Vec<u16> = Vec::new();
            let mut s_iter = s.clone();

            while !s_iter.is_zero() {
                chars_rev.push(DIGITS[bf_to_usize_unchecked(&s_iter.rem(&big_radix))]);
                s_iter = s_iter.div(&big_radix, 128, RoundingMode::ToEven).int();
            }

            let mut chars = Vec::new();

            while let Some(elem) = chars_rev.pop() {
                chars.push(elem);
            }

            let mut ntz = 0;

            while chars[chars.len() - (ntz + 1)] == 0x0030 {
                ntz += 1;
            }
            
            let little_n = bf_to_i64_unchecked(&n);

            // FUCKING FINALLY WE HAVE VALID DIGITS. just gotta make the correct text string now and we're off to the races

            if radix != 10 || (-5..=21).contains(&little_n) {
                if little_n >= k as i64 {
                    let mut nzeros = little_n - (k as i64);

                    while nzeros != 0 {
                        chars.push(0x0030); // 0
                        nzeros -= 1;
                    }
                    return IString::wrap(chars);
                }
                else if little_n > 0 {
                    chars.insert(little_n as usize, 0x002E); // .
                    while ntz != 0 && chars[chars.len() - 1] != 0x002E {
                        chars.pop();
                        ntz -= 1;
                    }
                    return IString::wrap(chars);
                }
                let mut zdot = Vec::from([0x0030u16, 0x002Eu16]);
                let mut zeros = Vec::new();
                let mut nzeroes = -little_n;

                while nzeroes != 0 {
                    zeros.push(0x0030);
                    nzeroes -= 1;
                }
                zdot.append(&mut zeros);
                zdot.append(&mut chars);
                while ntz != 0 {
                    zdot.pop();
                    ntz -= 1;
                }
                return IString::wrap(zdot)
            }
            // scientific notation
            let exponent_sign = if little_n < 0 {
                0x002Du16 // -
            } else {
                0x002Bu16 // +
            };

            if k > 1 {
                chars.insert(1, 0x002E); // .
            }
            chars.push(0x0065); // e
            chars.push(exponent_sign);
            chars.append(&mut IString::from_str(&little_n.to_string()).unwrap().into_vec());
            IString::wrap(chars)
        }
    }

    #[allow(dead_code)] // to be used later
    pub trait JBigInt {
        fn to_fixed_size_int(&self, signed: bool, bit_width: u32) -> BigInt;

        fn unary_minus(&self) -> BigInt;

        fn bitwise_not(&self) -> BigInt;

        /// Exponentiates.
        /// 
        /// # Errors
        /// 
        /// Throws a `RangeError` if `exp` is less than 0.
        // TODO: Error Objects (Box<dyn Error> is a placeholder)
        fn exponentiate(&self, exp: &BigInt) -> Result<BigInt, Box<dyn Error>>;

        fn multiply(&self, y: &BigInt) -> BigInt;

        /// Divides.
        /// 
        /// # Errors
        /// 
        /// Throws a `RangeError` if `y` is 0.
        // TODO: Error Objects (Box<dyn Error> is a placeholder)
        fn divide(&self, y: &BigInt) -> Result<BigInt, Box<dyn Error>>;

        /// Remainders.
        /// 
        /// # Errors
        /// 
        /// Throws a `RangeError` if `denom` is 0.
        // TODO: Error Objects (Box<dyn Error> is a placeholder)
        fn remainder(&self, denom: &BigInt) -> Result<BigInt, Box<dyn Error>>;

        fn subtract(&self, y: &BigInt) -> BigInt;

        fn left_shift(&self, y: &BigInt) -> BigInt;

        fn signed_right_shift(&self, y: &BigInt) -> BigInt;

        /// Implemented for completeness; this operation can't be done to
        /// `BigInts`, so only returns an error.
        /// 
        /// # Errors
        /// 
        /// Throws a `TypeError`. Does nothing else.
        // TODO: Error Objects (Box<dyn Error> is a placeholder)
        fn unsigned_right_shift(&self, y: &BigInt) -> Result<(), Box<dyn Error>>;

        fn less_than(&self, other: &BigInt) -> bool;

        fn equal(&self, other: &BigInt) -> bool;

        fn bitwise_op(&self, rhs: &BigInt, op: BitOps) -> BigInt;

        fn bitwise_and(&self, y: &BigInt) -> BigInt;

        fn bitwise_xor(&self, y: &BigInt) -> BigInt;

        fn bitwise_or(&self, y: &BigInt) -> BigInt;

        fn to_string(&self, radix: u32) -> IString;
    }

    impl JBigInt for BigInt {
        #[allow(clippy::cast_possible_wrap)] // checked via assert
        #[allow(clippy::cast_possible_truncation)] // intentional here
        fn to_fixed_size_int(&self, signed: bool, bit_width: u32) -> BigInt {
            let two = BigInt::from(2);
            let two_pow = two.pow(bit_width);
            let mut fixed_int = self.rem(&two_pow);

            // NOTE: does not change the two's complement representation of fixed_int
            if signed && fixed_int >= two.pow(bit_width - 1) {
                fixed_int -= two_pow;
            }
            fixed_int
        }

        fn unary_minus(&self) -> BigInt {
            self.neg()
        }
    
        fn bitwise_not(&self) -> BigInt {
            self.not()
        }
    
        fn exponentiate(&self, exp: &BigInt) -> Result<BigInt, Box<dyn Error>> {
            if exp < &BigInt::ZERO {
                // TODO: Error Objects (Box<dyn Error> is a placeholder)
                todo!()
            }
            todo!()
        }
    
        fn multiply(&self, y: &BigInt) -> BigInt {
            self.mul(y)
        }
    
        fn divide(&self, y: &BigInt) -> Result<BigInt, Box<dyn Error>> {
            if y == &BigInt::ZERO {
                // TODO: Error Objects (Box<dyn Error> is a placeholder)
                todo!()
            }
            todo!()
        }
    
        fn remainder(&self, denom: &BigInt) -> Result<BigInt, Box<dyn Error>> {
            if denom == &BigInt::ZERO {
                // TODO: Error Objects (Box<dyn Error> is a placeholder)
                todo!()
            }
            todo!()
        }
    
        fn subtract(&self, y: &BigInt) -> BigInt {
            self.sub(y)
        }
    
        /*
         * rust's version of this does NOT take a bigint as y, because that is
         * ludicrous. you'd fill up memory incredibly quickly. just convert y
         * into an i64. if you oom, that's your fault - you are wrong for doing
         * that. we convert the lazy way by only keeping the 64 least
         * significant bits of y. we do preserve the sign.
         */
        fn left_shift(&self, y: &BigInt) -> BigInt {
            let two = BigInt::from(2);

            if y > &two.pow(30) {
                println!("ecma-262: BigInt: warning: crazy left shift by {y} bits");
            }

            let y_parts = y.to_u64_digits();
            let sign = match y_parts.0 {
                num_bigint::Sign::Minus => -1,
                num_bigint::Sign::NoSign | num_bigint::Sign::Plus => 1,
            };
            let small_y = (y_parts.1[0] & 0x7FFF_FFFF_FFFF_FFFF).cast_signed() * sign;

            self.shl(small_y)
        }
    
        fn signed_right_shift(&self, y: &BigInt) -> BigInt {
            self.left_shift(&y.neg())
        }
    
        fn unsigned_right_shift(&self, _y: &BigInt) -> Result<(), Box<dyn Error>> {
            // TODO: Error Objects (Box<dyn Error> is a placeholder)
            todo!()
        }
    
        fn less_than(&self, other: &BigInt) -> bool {
            self < other
        }
    
        fn equal(&self, other: &BigInt) -> bool {
            self == other
        }

        /*
         * i really really did not feel like fully implementing this per spec.
         * since the spec does not indicate that this can fail, just piggyback
         * on the Rust implementations. the binary_and, binary_or, and
         * binary_xor functions aren't implemented and won't be used.
         */
        fn bitwise_op(&self, rhs: &BigInt, op: BitOps) -> BigInt {
            match op {
                BitOps::Amp => self & rhs,
                BitOps::Caret => self ^ rhs,
                BitOps::Pipe => self | rhs,
            }
        }
    
        fn bitwise_and(&self, y: &BigInt) -> BigInt {
            self.bitwise_op(y, BitOps::Amp)
        }
    
        fn bitwise_xor(&self, y: &BigInt) -> BigInt {
            self.bitwise_op(y, BitOps::Caret)
        }
    
        fn bitwise_or(&self, y: &BigInt) -> BigInt {
            self.bitwise_op(y, BitOps::Pipe)
        }
        
        // hooray, no nasty shit this time. thank you, num-bigint, for keeping a function the rest of rust decided to throw away
        fn to_string(&self, radix: u32) -> IString {
            IString::from_str(&self.to_str_radix(radix))
        }
    }

    /*
     * Keys must be Ord in order to be used with an Object's internal Map. Symbols
     * are not themselves Ord, and will never be, so instead we implement Ord on
     * the Key. The semantics are as follows:
     * All Symbols are lesser than all Strings.
     * Symbols will be ordered based on their internal id (and this may change
     * between different incovations of the same program.)
     * Strings will be ordered using IString::cmp().
     */
    #[derive(Debug, Clone)]
    pub enum Key {
        Symbol(Arc<Symbol>),
        String(IString),
    }

    impl PartialEq for Key {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Self::String(l0), Self::String(r0)) => l0 == r0,
                (Self::Symbol(l0), Self::Symbol(r0)) => Arc::ptr_eq(l0, r0),
                _ => false,
            }
        }
    }

    impl Eq for Key {}

    impl PartialOrd for Key {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for Key {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            match self {
                Key::Symbol(s_sym) => match other {
                    Key::Symbol(o_sym) => s_sym.id.cmp(&o_sym.id),
                    Key::String(_) => std::cmp::Ordering::Less,
                },
                Key::String(s_istring) => match other {
                    Key::Symbol(_) => std::cmp::Ordering::Greater,
                    Key::String(o_istring) => s_istring.cmp(o_istring),
                },
            }
        }
    }

    impl Key {
        #[must_use]
        pub fn is_name(&self) -> bool {
            match self {
                Key::String(_) => true,
                Key::Symbol(_) => false,
            }
        }

        #[must_use]
        pub fn is_int_index(&self) -> bool {
            match self {
                // TODO: CanonicalNumericIndexString
                Key::String(_istring) => todo!(),
                Key::Symbol(_) => false,
            }
        }

        #[must_use]
        pub fn is_array_index(&self) -> bool {
            match self {
                // TODO: CanonicalNumericIndexString
                Key::String(_istring) => todo!(),
                Key::Symbol(_) => false,
            }
        }

        #[must_use]
        pub fn get_known(key: SymNames) -> Key {
            Key::Symbol(Symbol::get_known(key))
        }

        #[must_use]
        pub fn as_jtype(&self) -> JType {
            match self {
                Key::Symbol(symbol) => JType::Symbol(symbol.clone()),
                Key::String(istring) => JType::JString(istring.clone()),
            }
        }
    }

    impl FromStr for Key {
        type Err = !;
    
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Key::String(IString::from_str(s)))
        }
    }

    #[derive(Debug, Clone)]
    pub struct PropDescData {
        pub val: Option<JType>,
        pub writable: Option<bool>,
    }

    #[derive(Debug, Clone)]
    pub struct PropDescAccessor {
        // kinda gross: if outer option is None, field doesn't exist. if inner option is None, field exists but is undefined
        pub getter: Option<Option<Object>>,
        pub setter: Option<Option<Object>>,
    }

    #[derive(Clone, Debug)]
    pub enum PropDescType {
        Data(PropDescData),
        Accessor(PropDescAccessor),
        Gen
    }
    
    #[derive(Clone, Debug)]
    #[allow(dead_code)] // TODO: unfinished
    pub struct PropDesc {
        // goddamn keyword conflicts. ideally named "type"
        pub kind: PropDescType,
        pub enumerable: Option<bool>,
        pub configurable: Option<bool>,
    }

    pub const PROPDESC_DATA_DEFAULT: PropDescData = PropDescData {
        val: Some(JType::Undefined),
        writable: Some(false),
    };

    pub const PROPDESC_ACCESSOR_DEFAULT: PropDescAccessor = PropDescAccessor {
        getter: Some(None),
        setter: Some(None),
    };

    pub const PROPDESC_DEFAULT: PropDesc = PropDesc {
        kind: PropDescType::Gen,
        enumerable: Some(false),
        configurable: Some(false),
    };

    impl PropDesc {
        #[must_use]
        pub fn is_fully_populated(&self) -> bool {
            (self.is_accessor().is_some() || self.is_data().is_some()) && self.enumerable.is_some() && self.configurable.is_some()
        }

        #[must_use]
        pub fn is_accessor(&self) -> Option<PropDescAccessor> {
            if let PropDescType::Accessor(access) = &self.kind && access.getter.is_some() && access.setter.is_some() {
                Some(access.clone())
            } else {
                None
            }
        }

        #[must_use]
        pub fn is_data(&self) -> Option<PropDescData> {
            if let PropDescType::Data(data) = &self.kind && data.val.is_some() && data.writable.is_some() {
                Some(data.clone())
            } else {
                None
            }
        }

        #[must_use]
        pub fn is_generic(&self) -> bool {
            !(self.is_accessor().is_some() || self.is_data().is_some())
        }

        #[allow(clippy::missing_panics_doc)] // if we panic this is a bug
        pub fn complete(&mut self) {
            if self.is_generic() || self.is_data().is_some() {
                match self.kind {
                    PropDescType::Data(ref mut data) => {
                        if data.val.is_none() {
                            data.val.clone_from(&PROPDESC_DATA_DEFAULT.val);
                        }
                        if data.writable.is_none() {
                            data.writable = PROPDESC_DATA_DEFAULT.writable;
                        }
                    },
                    PropDescType::Accessor(_) => panic!("PropDesc::complete: the front fell off"),
                    PropDescType::Gen => self.kind = PropDescType::Data(PROPDESC_DATA_DEFAULT.clone()),
                }
            } else {
                match self.kind {
                    PropDescType::Data(_) | PropDescType::Gen => panic!("PropDesc::complete: the front fell off"),
                    PropDescType::Accessor(ref mut accessor) => {
                        if accessor.getter.is_none() {
                            accessor.getter.clone_from(&PROPDESC_ACCESSOR_DEFAULT.getter);
                        }
                        if accessor.setter.is_none() {
                            accessor.setter.clone_from(&PROPDESC_ACCESSOR_DEFAULT.setter);
                        }
                    },
                }
            }
            if self.enumerable.is_none() {
                self.enumerable = PROPDESC_DEFAULT.enumerable;
            }
            if self.configurable.is_none() {
                self.configurable = PROPDESC_DEFAULT.configurable;
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ObjKind {
        Func,
        ConsFunc,
        BoundFunc,
    }

    #[allow(unused_variables)] // TODO: unfinished
    #[allow(clippy::missing_errors_doc)] // errors will (usually) happen internally based on if an object is exotic or not. we just propagate
    pub trait Obj: Send + Sync + std::fmt::Debug {
        fn get_prototype_of(&self) -> Result<Option<&Object>, Box<dyn Error>> {
            todo!()
        }

        fn set_prototype_of(&mut self, proto: Option<&Object>) -> Result<bool, Box<dyn Error>> {
            todo!()
        }

        fn is_extensible(&self) -> Result<bool, Box<dyn Error>> {
            todo!()
        }

        fn prevent_extensions(&self) -> Result<bool, Box<dyn Error>> {
            todo!()
        }

        fn get_own_property(&self, key: &Key) -> Result<Option<PropDesc>, Box<dyn Error>> {
            todo!()
        }

        fn define_own_property(self: Arc<Self>, key: Key, desc: PropDesc) -> Result<bool, Box<dyn Error>> {
            todo!()
        }

        fn has_property(&self, key: Key) -> Result<bool, Box<dyn Error>> {
            todo!()
        }

        fn get(&self, key: &Key, recv: &JType) -> Result<JType, Box<dyn Error>> {
            todo!()
        }

        fn set(self: Arc<Self>, key: Key, val: JType, recv: &JType) -> Result<bool, Box<dyn Error>> {
            todo!()
        }

        fn delete(self: Arc<Self>, key: Key) -> Result<bool, Box<dyn Error>> {
            todo!()
        }

        fn own_property_keys(&self) -> Result<List<Key>, Box<dyn Error>> {
            todo!()
        }

        fn call(&self, this: &JType, args: List<JType>) -> Result<JType, Box<dyn Error>> {
            todo!()
        }

        fn construct_int(&self, args: List<JType>, new: Object) -> Result<Object, Box<dyn Error>> {
            todo!()
        }

        fn clone(&self) -> Arc<dyn Obj>;

        fn is_ordinary(&self) -> bool;

        fn is_callable(&self) -> bool;

        fn is_constructor(self: Arc<Self>) -> Option<Arc<ConsFunc>>;

        fn has_reg_exp_matcher(&self) -> bool;

        fn make_object(self: Arc<Self>) -> Object
        where 
            Self: Sized + 'static,
        {
            Object {
                id: self.get_id(),
                map: self.get_map().upgrade().unwrap(),
                private_elements: self.get_private_elements().upgrade().unwrap(),
                inner: self.clone(),
            }
        }

        fn get_id(&self) -> u64;

        fn get_clone_id(&self) -> u64;

        fn get_map(&self) -> Weak<RwLock<Map<Key, PropDesc>>>;

        fn get_private_elements(&self) -> Weak<RwLock<List<PrivateElement>>>;

        fn get_kind(&self) -> ObjKind;
    }

    /*
     * this and ConsFunc may be better off as structs that other Obj structs
     * can include rather than implementing Obj themselves, but i won't know
     * until i actually start implementing other Obj structs
     */
    #[derive(Clone, Debug)]
    pub struct Func {
        id: u64,
        clone_id: u64,
        map: Weak<RwLock<Map<Key, PropDesc>>>,
        private_elements: Weak<RwLock<List<PrivateElement>>>,
    }

    impl Obj for Func {
        fn clone(&self) -> Arc<dyn Obj> {
            todo!()
        }

        fn is_ordinary(&self) -> bool {
            todo!()
        }

        fn is_callable(&self) -> bool {
            true
        }

        fn is_constructor(self: Arc<Self>) -> Option<Arc<ConsFunc>> {
            None
        }

        fn has_reg_exp_matcher(&self) -> bool {
            false
        }
        
        fn get_id(&self) -> u64 {
            self.id
        }
        
        fn get_clone_id(&self) -> u64 {
            self.clone_id
        }
        
        fn get_map(&self) -> Weak<RwLock<Map<Key, PropDesc>>> {
            self.map.clone()
        }
        
        fn get_private_elements(&self) -> Weak<RwLock<List<PrivateElement>>> {
            self.private_elements.clone()
        }

        fn get_kind(&self) -> ObjKind {
            ObjKind::Func
        }
    }

    #[derive(Debug)]
    #[allow(dead_code)] // TODO: unfinished
    pub struct ConsFunc {
        inner: Func,
        id: u64,
        clone_id: u64,
        map: Weak<RwLock<Map<Key, PropDesc>>>,
        private_elements: Weak<RwLock<List<PrivateElement>>>,
    }

    impl ConsFunc {
        #[must_use]
        #[allow(clippy::new_ret_no_self)] // intentional: generally, structs that implement Obj will always be wrapped in Object structs and never interacted with directly
        pub fn new() -> Object {
            Object::new(|id, map, private_elements| Arc::new(ConsFunc {
                inner: Func {
                    id,
                    clone_id: 0,
                    map: map.clone(),
                    private_elements: private_elements.clone(),
                },
                id,
                clone_id: 0,
                map,
                private_elements,
            }))
        }
    }

    impl Obj for ConsFunc {
        fn clone(&self) -> Arc<dyn Obj> {
            todo!()
        }

        fn is_ordinary(&self) -> bool {
            todo!()
        }

        fn is_callable(&self) -> bool {
            true
        }

        fn is_constructor(self: Arc<Self>) -> Option<Arc<ConsFunc>> {
            Some(self.clone())
        }

        fn has_reg_exp_matcher(&self) -> bool {
            false
        }
        
        fn get_id(&self) -> u64 {
            self.id
        }
        
        fn get_clone_id(&self) -> u64 {
            self.clone_id
        }
        
        fn get_map(&self) -> Weak<RwLock<Map<Key, PropDesc>>> {
            self.map.clone()
        }
        
        fn get_private_elements(&self) -> Weak<RwLock<List<PrivateElement>>> {
            self.private_elements.clone()
        }

        fn get_kind(&self) -> ObjKind {
            ObjKind::ConsFunc
        }
    }

    #[derive(Debug)]
    #[allow(dead_code)] // TODO: unfinished
    pub struct Object {
        id: u64,
        map: ArcRw<Map<Key, PropDesc>>,
        // TODO: PrivateElement type
        pub private_elements: ArcRw<List<PrivateElement>>,
        // NOTE: not an ArcRw so we can be more granular with the contents
        pub inner: Arc<dyn Obj>,
    }

    // TODO: Error Objects (Box<dyn Error> is a placeholder)
    // TODO: Default implementations for the standard internal methods
    #[allow(unused_variables)] // TODO: unfinished
    #[allow(clippy::needless_pass_by_value)] // TODO: unfinished
    #[allow(clippy::missing_errors_doc)] // errors will (usually) happen internally based on if an object is exotic or not. we just propagate
    impl Object {
        #[must_use]
        pub fn from_prop_desc(prop_desc: &PropDesc) -> Object {
            // TODO: OrdinaryObjectCreate
            todo!()
        }

        // we are not going to be able to determine this algorithmically without a fuckton of glue. we are just going to trust that the inner type has set the ordinary flag correctly
        #[must_use]
        pub fn is_ordinary(&self) -> bool {
            self.inner.is_ordinary()
        }

        #[must_use]
        pub fn is_exotic(&self) -> bool {
            !self.is_ordinary()
        }

        #[must_use]
        pub fn is_callable(&self) -> bool {
            self.inner.is_callable()
        }

        #[must_use]
        pub fn is_constructor(&self) -> Option<Arc<ConsFunc>> {
            self.inner.clone().is_constructor()
        }

        #[must_use]
        pub fn has_reg_exp_matcher(&self) -> bool {
            self.inner.has_reg_exp_matcher()
        }

        #[must_use]
        pub fn clone_shallow(&self) -> Object {
            Object {
                id: self.id,
                map: self.map.clone(),
                private_elements: self.private_elements.clone(),
                inner: self.inner.clone(),
            }
        }

        #[must_use]
        pub fn get_kind(&self) -> ObjKind {
            self.inner.get_kind()
        }

        #[must_use]
        pub fn new<F>(data_fn: F) -> Object
        where 
            F: FnOnce(u64, Weak<RwLock<Map<Key, PropDesc>>>, Weak<RwLock<List<PrivateElement>>>) -> Arc<dyn Obj>
        {
            static OBJ_COUNTER: AtomicU64 = AtomicU64::new(0);
            let id = OBJ_COUNTER.fetch_add(1, Ordering::Relaxed);
            let map = Arc::new(RwLock::new(Map::new()));
            let private_elements = Arc::new(RwLock::new(List::new()));

            Object {
                id,
                map: map.clone(),
                private_elements: private_elements.clone(),
                inner: data_fn(id, Arc::downgrade(&map), Arc::downgrade(&private_elements)),
            }
        }
    }

    // deep clone
    impl Clone for Object {
        fn clone(&self) -> Self {
            Self {
                id: self.id,
                map: Arc::new(RwLock::new(self.map.read().clone())),
                private_elements: Arc::new(RwLock::new(self.private_elements.read().clone())),
                inner: (*self.inner).clone()
            }
        }
    }

    impl PartialEq for Object {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id && self.inner.get_clone_id() == other.inner.get_clone_id()
        }
    }

    #[derive(Debug)]
    pub enum JType {
        Undefined,
        Null,
        Bool(bool),
        JString(IString),
        Symbol(Arc<Symbol>),
        Number(f64),
        BigInt(BigInt),
        Object(Object),
    }

    impl JType {
        #[must_use]
        pub fn get_value(self) -> JType {
            self
        }

        /// A "gotcha" function for people trying to use the `Reference` version
        /// on a `JType`.
        /// 
        /// # Errors
        /// 
        /// This function's only purpose is to return an error. Don't use it if
        /// you can help it.
        pub fn put_value(&self, _val: JType) -> Result<(), Box<dyn Error>> {
            // TODO: Error Objects (Box<dyn Error> is a placeholder)
            todo!()
        }

        #[must_use]
        pub fn is_defined(&self) -> bool {
            !matches!(self, JType::Undefined)
        }

        /// The abstract operation `to_prop_desc` takes argument `self` (an
        /// ECMAScript language value) and returns a `Result` containing a
        /// `PropDesc` or an error.
        /// 
        /// # Errors
        /// 
        /// Returns a `TypeError` if `self` is not an `Object`, if `self` has a
        /// defined getter that is not an object, if `self` has a defined
        /// setter that is not an object, or if `self` would become a `PropDesc`
        /// that is both an accessor descriptor and data descriptor
        /// simultaneously (which is not a valid `PropDesc`.)
        pub fn to_prop_desc(&self) -> Result<PropDesc, Box<dyn Error>> {
            // TODO: Error Objects (Box<dyn Error> is a placeholder)
            todo!()
        }
    }

    impl Clone for JType {
        fn clone(&self) -> Self {
            match self {
                Self::Undefined => Self::Undefined,
                Self::Null => Self::Null,
                Self::Bool(arg0) => Self::Bool(*arg0),
                Self::JString(arg0) => Self::JString(arg0.clone()),
                Self::Symbol(arg0) => Self::Symbol(arg0.clone()),
                Self::Number(arg0) => Self::Number(*arg0),
                Self::BigInt(arg0) => Self::BigInt(arg0.clone()),
                // we specifically don't want to deep clone Objects constantly
                Self::Object(arg0) => Self::Object(arg0.clone_shallow()),
            }
        }
    }

    /* past here we start getting into specification types. unlike the language
     * types, these are part of the specification algorithms and are never
     * exposed to the user. many of these correspond to existing Rust or Infra
     * types and control features, though a few may need wrapper traits to work
     * correctly.
     * Enum: Corresponds directly to Rust enums. No special type need be
     * created.
     * List: Implemented within Infra.
     * Record: This type is somewhat hairy. The fields of a Record are more-or-
     * less hardcoded on creation and will not change, so most Records can be
     * described with a corresponding specific struct type within Rust. If
     * something ever refers to records in general, we may be able to implement
     * it as a trait on various structures when needed. some of the other
     * specification types are actually Records and have just been made into
     * structs.
     * Set: Implemented within Infra.
     * Relations: Hard for me to describe. They themselves are Sets, but also
     * describe other sets by consisting of ordered pairs of values from
     * another Set. I'm not implementing them until I need to, since by that
     * point I will know more about what I need to do, and even then it will be
     * a simple wrapper around Sets.
     * Completion Records: Unlike most other Records, these are more like
     * enums in how they are described and used. Their first field is even an
     * enum, and based on what that enum's value is it acts differently. The
     * primary types of NORMAL and THROW map directly to Rust's Result<T, E>
     * enum, and the spec even has the ? shorthand that exists within Rust. !
     * maps to Result::unwrap(). Those two are used throughout the main
     * specification, the rest of the enum's members are used during actual
     * Javascript evaluation and execution and can be described using a
     * separate construct once I get there.
     * Reference Records: TODO: Are their own type and must be implemented.
     * Property Descriptors: TODO: Partially implemented; structure exists but
     * needs an impl block with described methods.
     * Environment Records: TODO: These are complex enough that they weren't
     * described in this chapter and won't be implemented in this file.
     * Abstract Closures: Correspond directly to Rust closures.
     * Data Blocks: Implemented more or less by the computer hardware, since
     * these are just memory locations of a certain size. They can be shared;
     * if so, they can never overlap each other in memory. All that really
     * needs to be done here is write the described abstract operations.
     * PrivateElement: TODO: Composes the PrivateElement list in Objects. Must
     * be implemented.
     * ClassFieldDefinition: TODO: Are their own type and must be implemented.
     * Private Names: TODO: Are their own type and must be implemented. Are
     * very similar to Symbols in how they are described; only difference is
     * that they always have a Description field (unlike Symbols.) As a result,
     * I cannot piggyback on the Symbol type without causing problems.
     * ClassStaticBlockDefinition: TODO: A small wrapper Record around a
     * function Object. Used to keep this function from being accessible from
     * the rest of the script's code. Must be implemented.
     */

    pub enum RefBase {
        Value(JType),
        // TODO: Environment Records
        EnvRec(Todo),
        Unresolvable,
    }

    pub enum RefRefName {
        Value(JType),
        // TODO: Private Names
        Name(Todo),
    }

    #[allow(dead_code)] // TODO: unfinished
    pub struct Reference {
        base: RefBase,
        referenced_name: RefRefName,
        strict: bool,
        this_value: Option<JType>,
    }

    impl Reference {
        #[must_use]
        pub fn is_property(&self) -> bool {
            match self.base {
                RefBase::Value(_) => true,
                RefBase::EnvRec(_) | RefBase::Unresolvable => false,
            }
        }

        #[must_use]
        pub fn is_unresolvable(&self) -> bool {
            match self.base {
                RefBase::Value(_) | RefBase::EnvRec(_) => false,
                RefBase::Unresolvable => true,
            }
        }

        #[must_use]
        pub fn is_super(&self) -> bool {
            self.this_value.is_some()
        }

        #[must_use]
        pub fn is_private(&self) -> bool {
            match self.referenced_name {
                RefRefName::Value(_) => false,
                RefRefName::Name(_) => true,
            }
        }

        /// The abstract operation `get_value` takes argument `self` (a
        /// Reference) and returns a Result containing an ECMAScript language
        /// value or an error.
        /// 
        /// # Errors
        /// 
        /// Returns a `ReferenceError` if the reference is unresolvable.
        pub fn get_value(&self) -> Result<JType, Box<dyn Error>> {
            // TODO: Error Objects (Box<dyn Error> is a placeholder)
            todo!()
        }

        /// The abstract operation `put_value` takes arguments `self` (a
        /// Reference) and `val` (an ECMAScript language value) and returns
        /// a Result containing `()` or an error.
        /// 
        /// # Errors
        /// 
        /// Returns a `ReferenceError` if the reference is unresolvable and
        /// strict. Returns a `TypeError` if we are in strict mode and and
        /// couldn't put the value.
        #[allow(unused_variables)] // TODO: unfinished
        #[allow(clippy::needless_pass_by_value)] // TODO: unfinished
        pub fn put_value(&self, val: JType) -> Result<(), Box<dyn Error>> {
            if self.is_unresolvable() {
                // TODO: Error Objects (Box<dyn Error> is a placeholder)
                todo!()
            }
            todo!()
        }

        /// The abstract operation `get_this_value` takes argument `self` (a
        /// Reference) and returns an ECMAScript language value.
        /// 
        /// # Panics
        /// 
        /// Panics if `self` is not a property reference.
        #[must_use]
        pub fn get_this_value(&self) -> JType {
            assert!(self.is_property());

            if self.is_super() {
                self.this_value.clone().unwrap()
            } else if let RefBase::Value(ret) = &self.base {
                ret.clone()
            } else {
                panic!("Reference::get_this_value: the front fell off")
            }
        }

        /// The abstract operation `initialize_referenced_binding` takes
        /// arguments `self` (a Reference) and `val` (an ECMAScript language
        /// value) and returns a Result containing () or an error.
        /// 
        /// # Panics
        /// 
        /// Panics if `self` is an unresolvable reference, or if `self.base` is
        /// not an Environment Record.
        #[allow(unused_variables)] // TODO: unfinished
        #[allow(clippy::missing_errors_doc)] // we only propagate errors from elsewhere here
        #[allow(clippy::needless_pass_by_value)]  // TODO: unfinished
        #[allow(clippy::redundant_else)] // TODO: unfinished
        pub fn initialize_referenced_binding(&self, val: JType) -> Result<(), Box<dyn Error>> {
            assert!(!self.is_unresolvable());

            if let RefBase::EnvRec(_) = &self.base {
                // TODO: Environment Records
                todo!()
            } else {
                panic!("Reference::initialize_referenced_binding: base not an Environment")
            }
        }

        #[must_use]
        #[allow(unused_variables)] // TODO: unfinished
        #[allow(clippy::needless_pass_by_value)]  // TODO: unfinished
        pub fn make_private(base_val: JType, private_id: IString) -> Reference {
            // TODO: PrivateEnvironment Records
            todo!()
        }
    }

    #[derive(Debug, Clone)]
    pub struct PrivElemAccessor {
        pub getter: Option<Object>,
        pub setter: Option<Object>,
    }

    #[derive(Debug, Clone)]
    pub enum PrivElemKind {
        Field(JType),
        Method(JType),
        // must be Func Objects or None
        // Getter, Setter
        Accessor(PrivElemAccessor)
    }

    #[derive(Debug, Clone)]
    #[allow(dead_code)] // TODO: unfinished
    pub struct PrivateElement {
        pub key: Arc<PrivName>, // TODO: Private Names
        pub kind: PrivElemKind,
    }

    pub enum CFieldDefName {
        Name(Arc<PrivName>),
        Key(Key),
    }

    #[allow(dead_code)] // TODO: unfinished
    pub struct ClassFieldDef {
        pub name: CFieldDefName,
        pub init: Option<Object>,
    }

    // annoyingly similar to the Symbol type, but legally distinct
    // TODO: utility of the id is questionable since we implement eq() based on the address of the Weak references. may remove later
    #[allow(dead_code)] // TODO: unfinished
    #[derive(Debug)]
    pub struct PrivName {
        me: Weak<PrivName>,
        id: u64,
        description: IString,
    }

    impl PrivName {
        #[must_use]
        pub fn new(desc: &IString) -> Arc<Self> {
            static PRIVNAME_COUNTER: AtomicU64 = AtomicU64::new(0);

            Arc::new_cyclic(|me |
                PrivName {
                    me: me.clone(),
                    id: PRIVNAME_COUNTER.fetch_add(1, Ordering::Relaxed),
                    description: desc.clone(),
                }
            )
        }
    }

    #[allow(dead_code)] // TODO: unfinished
    pub struct ClassStaticBlockDef {
        // must be a Func Object
        body: Object,
    }

    #[allow(unused_variables)] // TODO: unfinished
    pub mod data_block {
        use std::{error::Error, sync::Arc};
        use deadflies_rust_infra::structs::ArcRw;

        #[allow(dead_code)] // TODO: unfinished
        pub struct DataBlock {
            is_shared: bool,
            vec: Vec<u8>,
        }

        /// The abstract operation `create` takes argument size (a `usize`) and
        /// returns a Result containing a Vec<u8> or a throw completion.
        /// 
        /// # Errors
        /// 
        /// If size is above the maximum safe integer of the Number type, or if
        /// the block otherwise cannot be created (e.g OOM), returns a
        /// `RangeError`. In the case of an OOM, specifically, your program
        /// will crash.
        pub fn create(size: usize) -> Result<ArcRw<DataBlock>, Box<dyn Error>> {
            // TODO: Error Objects (Box<dyn Error> is a placeholder)
            todo!()
        }

        /// The abstract operation `create_shared` takes argument size (a
        /// `usize`) and returns a Result containing a Vec<u8> or a throw
        /// completion.
        /// 
        /// # Errors
        /// 
        /// If the block cannot be created, returns a `RangeError` in theory.
        /// In practice, your program will crash due to running out of memory.
        pub fn create_shared(size: usize) -> Result<ArcRw<DataBlock>, Box<dyn Error>> {
            // TODO: Error Objects (Box<dyn Error> is a placeholder)
            todo!()
        }
        
        /// The abstract operation `copy_bytes` takes arguments `to` (a
        /// `DataBlock`), `to_ind` (a usize), from (a `DataBlock`), `from_ind`
        /// (a usize), and count (a usize.)
        /// 
        /// # Panics
        /// 
        /// Panics if we are copying between identical data blocks, if the data
        /// we are trying to read would overrun the end of `from`, or if we
        /// would overrun the end of `to` when writing.
        pub fn copy_bytes(to: &ArcRw<DataBlock>, to_ind: usize, from: &ArcRw<DataBlock>, from_ind: usize, mut count: usize) {
            assert!(!Arc::ptr_eq(to, from));
            assert!(from_ind + count <= from.read().vec.len());
            assert!(to_ind + count <= to.read().vec.len());

            while count > 0 {
                if from.read().is_shared {
                    // TODO: Agent Records
                    todo!()
                }
                count = 0; // TODO: clippy freaks out unless we add this. remove after full implementation
            }
        }
    }
}