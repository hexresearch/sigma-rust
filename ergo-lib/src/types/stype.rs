//! SType hierarchy

use std::convert::TryFrom;
use std::convert::TryInto;
use std::slice::Iter;

use impl_trait_for_tuples::impl_for_tuples;

use crate::chain::ergo_box::ErgoBox;
use crate::serialization::types::TypeCode;
use crate::sigma_protocol::dlog_group::EcPoint;
use crate::sigma_protocol::sigma_boolean::ProveDlog;
use crate::sigma_protocol::sigma_boolean::SigmaBoolean;
use crate::sigma_protocol::sigma_boolean::SigmaProofOfKnowledgeTree;
use crate::sigma_protocol::sigma_boolean::SigmaProp;

use super::scontext::SContext;
use super::sfunc::SFunc;
use super::stype_companion::STypeCompanion;
use super::stype_param::STypeVar;

/// Every type descriptor is a tree represented by nodes in SType hierarchy.
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum SType {
    /// Type variable (generic)
    STypeVar(STypeVar),
    /// TBD
    SAny,
    /// Boolean
    SBoolean,
    /// Signed byte
    SByte,
    /// Signed short (16-bit)
    SShort,
    /// Signed int (32-bit)
    SInt,
    /// Signed long (64-bit)
    SLong,
    /// 256-bit integer
    SBigInt,
    /// Discrete logarithm prime-order group element [`EcPoint`]
    SGroupElement,
    /// Proposition which can be proven and verified by sigma protocol.
    SSigmaProp,
    /// ErgoBox value
    SBox,
    /// AVL tree value
    SAvlTree,
    /// Optional value
    SOption(Box<SType>),
    /// Collection of elements of the same type
    SColl(Box<SType>),
    /// Tuple (elements can have different types)
    STuple(TupleItems<SType>),
    /// Function (signature)
    SFunc(Box<SFunc>),
    /// Context object ("CONTEXT" in ErgoScript)
    SContext(SContext),
}

impl SType {
    /// Type code used in serialization of SType values.
    pub fn type_code(&self) -> TypeCode {
        match self {
            SType::SAny => todo!(),
            SType::SBoolean => TypeCode::SBOOLEAN,
            SType::SByte => TypeCode::SBYTE,
            SType::SShort => TypeCode::SSHORT,
            SType::SInt => TypeCode::SINT,
            SType::SLong => TypeCode::SLONG,
            SType::SBigInt => TypeCode::SBIGINT,
            SType::SGroupElement => TypeCode::SGROUP_ELEMENT,
            SType::SSigmaProp => TypeCode::SSIGMAPROP,
            SType::SBox => todo!(),
            SType::SAvlTree => todo!(),
            SType::SOption(_) => TypeCode::OPTION,
            SType::SColl(_) => TypeCode::COLLECTION,
            SType::STuple(_) => TypeCode::TUPLE,
            SType::SFunc(_) => todo!(),
            SType::SContext(_) => todo!(),
            SType::STypeVar(_) => todo!(),
        }
    }

    /// Get STypeCompanion instance associated with this SType
    pub fn type_companion(&self) -> Option<Box<STypeCompanion>> {
        todo!()
    }
}

/// Tuple items with bounds check (2..=255)
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TupleItems<T>(Vec<T>);

#[allow(clippy::len_without_is_empty)]
impl<T> TupleItems<T> {
    // pub fn into_vec(self) -> Vec<T> {
    //     self.0
    // }

    /// Create a pair
    pub fn pair(t1: T, t2: T) -> TupleItems<T> {
        TupleItems(vec![t1, t2])
    }

    /// Get the length (quantity)
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Get an iterator
    pub fn iter(&self) -> Iter<T> {
        self.0.iter()
    }

    /// Get a slice
    pub fn as_slice(&self) -> &[T] {
        self.0.as_slice()
    }
}

/// Out of bounds items quantity error
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct STupleItemsOutOfBoundsError();

impl<T> TryFrom<Vec<T>> for TupleItems<T> {
    type Error = STupleItemsOutOfBoundsError;

    fn try_from(items: Vec<T>) -> Result<Self, Self::Error> {
        if items.len() >= 2 && items.len() <= 255 {
            Ok(TupleItems(items))
        } else {
            Err(STupleItemsOutOfBoundsError())
        }
    }
}

/// Conversion to SType
pub trait LiftIntoSType {
    /// get SType
    fn stype() -> SType;
}

impl<T: LiftIntoSType> LiftIntoSType for Vec<T> {
    fn stype() -> SType {
        SType::SColl(Box::new(T::stype()))
    }
}

impl LiftIntoSType for bool {
    fn stype() -> SType {
        SType::SBoolean
    }
}

impl LiftIntoSType for i8 {
    fn stype() -> SType {
        SType::SByte
    }
}

impl LiftIntoSType for i16 {
    fn stype() -> SType {
        SType::SShort
    }
}

impl LiftIntoSType for i32 {
    fn stype() -> SType {
        SType::SInt
    }
}

impl LiftIntoSType for i64 {
    fn stype() -> SType {
        SType::SLong
    }
}

impl LiftIntoSType for ErgoBox {
    fn stype() -> SType {
        SType::SBox
    }
}

impl LiftIntoSType for SigmaBoolean {
    fn stype() -> SType {
        SType::SSigmaProp
    }
}

impl LiftIntoSType for SigmaProofOfKnowledgeTree {
    fn stype() -> SType {
        SType::SSigmaProp
    }
}

impl LiftIntoSType for SigmaProp {
    fn stype() -> SType {
        SType::SSigmaProp
    }
}

impl LiftIntoSType for ProveDlog {
    fn stype() -> SType {
        SType::SSigmaProp
    }
}

impl LiftIntoSType for EcPoint {
    fn stype() -> SType {
        SType::SGroupElement
    }
}

impl<T: LiftIntoSType> LiftIntoSType for Option<T> {
    fn stype() -> SType {
        SType::SOption(Box::new(T::stype()))
    }
}

#[impl_for_tuples(2, 4)]
impl LiftIntoSType for Tuple {
    fn stype() -> SType {
        let v: Vec<SType> = [for_tuples!(  #( Tuple::stype() ),* )].to_vec();
        SType::STuple(v.try_into().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn primitive_type() -> BoxedStrategy<SType> {
        prop_oneof![
            Just(SType::SBoolean),
            Just(SType::SByte),
            Just(SType::SShort),
            Just(SType::SInt),
            Just(SType::SLong),
            Just(SType::SBigInt),
            Just(SType::SGroupElement),
            Just(SType::SSigmaProp),
        ]
        .boxed()
    }

    impl Arbitrary for SType {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            primitive_type()
                .prop_recursive(
                    4,  // no more than this branches deep
                    64, // total elements target
                    16, // each collection max size
                    |elem| {
                        prop_oneof![
                            prop::collection::vec(elem.clone(), 2..=4)
                                .prop_map(|elems| SType::STuple(elems.try_into().unwrap())),
                            elem.clone().prop_map(|tpe| SType::SColl(Box::new(tpe))),
                            elem.prop_map(|tpe| SType::SOption(Box::new(tpe))),
                        ]
                    },
                )
                .boxed()
        }
    }
}
