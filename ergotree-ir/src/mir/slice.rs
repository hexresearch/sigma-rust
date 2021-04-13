// FIXME: Doc
use super::expr::Expr;
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

/// Logical OR op on collection of SBoolean values
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Slice {
    /// Input collection
    pub input: Box<Expr>,
    /// FIXME: Doc
    pub from: Box<Expr>,
    /// FIXME: Doc
    pub until: Box<Expr>,
}

impl Slice {
    pub(crate) const OP_CODE: OpCode = OpCode::SLICE;

    /// Type
    pub fn tpe(&self) -> SType {
        self.input.tpe()
    }

    pub(crate) fn op_code(&self) -> OpCode {
        Self::OP_CODE
    }
}

impl SigmaSerializable for Slice {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.input.sigma_serialize(w)?;
        self.from.sigma_serialize(w)?;
        self.until.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let input = Expr::sigma_parse(r)?.into();
        let from = Expr::sigma_parse(r)?.into();
        let until = Expr::sigma_parse(r)?.into();
        Ok(Self{input, from, until})
    }
}

// #[cfg(feature = "arbitrary")]
// /// Arbitrary impl
// mod arbitrary {
//     use crate::mir::expr::arbitrary::ArbExprParams;
//
//     use super::*;
//
//     use proptest::prelude::*;
//
//     impl Arbitrary for Slice {
//         type Strategy = BoxedStrategy<Self>;
//         type Parameters = usize;
//
//         fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
//             any_with::<Expr>(ArbExprParams {
//                 tpe: SType::SColl(SType::SBoolean.into()),
//                 depth: args,
//             })
//             .prop_map(|input| Self {
//                 input: input.into(),
//             })
//             .boxed()
//         }
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::mir::expr::Expr;
//     use crate::serialization::sigma_serialize_roundtrip;
//     use proptest::prelude::*;
//
//     proptest! {
//
//         #[test]
//         fn ser_roundtrip(v in any_with::<Or>(1)) {
//             let expr: Expr = v.into();
//             prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
//         }
//
//     }
// }