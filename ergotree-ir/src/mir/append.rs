// FIXME: Doc
use super::expr::Expr;
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

/// Append two collections
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Append {
    /// Input collection
    pub input_a: Box<Expr>,
    /// Another collection
    pub input_b: Box<Expr>,
}

impl Append {
    pub(crate) const OP_CODE: OpCode = OpCode::APPEND;

    /// Type
    pub fn tpe(&self) -> SType {
        self.input_a.tpe()
    }

    pub(crate) fn op_code(&self) -> OpCode {
        Self::OP_CODE
    }
}

impl SigmaSerializable for Append {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.input_a.sigma_serialize(w)?;
        self.input_b.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let input_a = Expr::sigma_parse(r)?.into();
        let input_b = Expr::sigma_parse(r)?.into();
        Ok(Self{input_a, input_b})
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