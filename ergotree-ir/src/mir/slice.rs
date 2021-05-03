// FIXME: Doc
use super::expr::Expr;
use super::expr::InvalidArgumentError;
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

/// Selects an interval of elements
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Slice {
    /// Input collection
    pub input: Box<Expr>,
    /// The lowest index to include from this collection
    pub from: Box<Expr>,
    /// The lowest index to EXCLUDE from this collection
    pub until: Box<Expr>,
}

impl Slice {
    pub(crate) const OP_CODE: OpCode = OpCode::SLICE;

    /// Create new object, returns an error if any of the requirements failed
    pub fn new(input: Expr, from: Expr, until: Expr) -> Result<Self, InvalidArgumentError> {

        let _input_elem_tpe: SType = *match input.post_eval_tpe() {
            SType::SColl(elem_type) => Ok(elem_type),
            _ => Err(InvalidArgumentError(format!(
                "Expected Slice input to be SColl, got {0:?}",
                input.tpe()
            ))),
        }?;

        let _from_ok = match from.post_eval_tpe() {
            SType::SInt => Ok(()),
            _ => Err(InvalidArgumentError(format!(
                "Expected Slice from to be int-like, got {0:?}",
                from.tpe()
            ))),
        }?;

        let _until_ok = match until.post_eval_tpe() {
            SType::SInt => Ok(()),
            _ => Err(InvalidArgumentError(format!(
                "Expected Slice until to be int-like, got {0:?}",
                until.tpe()
            ))),
        }?;

        Ok(Slice {
            input: input.into(),
            from: from.into(),
            until: until.into(),
        })
    }

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
        Ok(Self { input, from, until })
    }
}

#[cfg(feature = "arbitrary")]
/// Arbitrary impl
mod arbitrary {
    use super::*;
    use crate::mir::expr::arbitrary::ArbExprParams;
    use proptest::prelude::*;

    impl Arbitrary for Slice {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = usize;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            (
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SColl(SType::SInt.into()),
                    depth: args,
                }),
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SInt,
                    depth: 0,
                }),
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SInt,
                    depth: 0,
                }),
            )
                .prop_map(|(input, from, until)| Self {
                    input: input.into(),
                    from: from.into(),
                    until: until.into(),
                })
                .boxed()
        }
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use super::*;
    use crate::mir::expr::Expr;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<Slice>()) {
            dbg!(&v);
            let expr: Expr = v.into(); prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }

    }
}
