//! Global variables

use crate::has_opcode::HasOpCode;
use crate::serialization::op_code::OpCode;
use crate::types::stype::SType;

#[derive(PartialEq, Eq, Debug, Clone)]
/// Predefined global variables
pub enum GlobalVars {
    /// Tx inputs
    Inputs,
    /// Tx outputs
    Outputs,
    /// Current blockchain height
    Height,
    /// ErgoBox instance, which script is being evaluated
    SelfBox,
    /// When interpreted evaluates to a ByteArrayConstant built from Context.minerPubkey
    MinerPubKey,
}

impl GlobalVars {
    /// Type
    pub fn tpe(&self) -> SType {
        match self {
            GlobalVars::Inputs => SType::SColl(Box::new(SType::SBox)),
            GlobalVars::Outputs => SType::SColl(Box::new(SType::SBox)),
            GlobalVars::Height => SType::SInt,
            GlobalVars::SelfBox => SType::SBox,
            GlobalVars::MinerPubKey => SType::SColl(Box::new(SType::SByte)),
        }
    }
}

impl HasOpCode for GlobalVars {
    /// Op code (serialization)
    fn op_code(&self) -> OpCode {
        match self {
            GlobalVars::SelfBox => OpCode::SELF_BOX,
            GlobalVars::Inputs => OpCode::INPUTS,
            GlobalVars::Outputs => OpCode::OUTPUTS,
            GlobalVars::Height => OpCode::HEIGHT,
            GlobalVars::MinerPubKey => OpCode::MINER_PUBKEY,
        }
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use super::*;
    use proptest::prelude::*;

    impl Arbitrary for GlobalVars {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            use GlobalVars::*;
            prop_oneof![
                Just(Inputs),
                Just(Outputs),
                Just(Height),
                Just(SelfBox),
                Just(MinerPubKey)
            ]
            .boxed()
        }
    }
}
