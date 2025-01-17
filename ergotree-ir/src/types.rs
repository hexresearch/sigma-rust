//! Sigma types

pub mod stype;

/// Box object type companion
pub mod sbox;
/// Collection object type companion
pub mod scoll;
pub mod scontext;
/// Function signature type
pub mod sfunc;
/// Global methods
pub mod sglobal;
/// Header's methods
pub mod sheader;
/// Object method(property) signature type
pub mod smethod;
/// PreHeader's methods
pub mod spreheader;
/// Tuple type
pub mod stuple;
/// Type companion for an object
pub mod stype_companion;
/// Type parameters for generic signatures
pub mod stype_param;
/// Types unification
pub mod type_unify;
