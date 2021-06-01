use ergotree_ir::mir::constant::TryExtractFrom;
use ergotree_ir::mir::slice::Slice;
use ergotree_ir::mir::value::CollKind;
use ergotree_ir::mir::value::Value;
use ergotree_ir::types::stype::SType;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for Slice {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        let from_v = self.from.eval(env, ctx)?;
        let until_v = self.until.eval(env, ctx)?;
        let (normalized_input_vals, elem_tpe): (Vec<Value>, SType) = match input_v {
            Value::Coll(coll) => Ok((coll.as_vec(), coll.elem_tpe().clone())),
            _ => Err(EvalError::UnexpectedValue(format!(
                "expected Slice input to be Value::Coll, got: {0:?}",
                input_v
            ))),
        }?;

        let from_i: usize = (usize::try_extract_from(from_v))?;
        let until_i: usize = (usize::try_extract_from(until_v))?;

        Ok(Value::Coll(CollKind::from_vec(
            elem_tpe.clone(),
            (&normalized_input_vals[from_i..until_i]).to_vec(),
        )?))
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use crate::eval::tests::try_eval_out_wo_ctx;

    use super::*;

    // use ergotree_ir::ir_ergo_box::IrBoxId;
    // use ergotree_ir::mir::bin_op::BinOp;
    // use ergotree_ir::mir::bin_op::RelationOp;
    use ergotree_ir::mir::expr::Expr;
    // use ergotree_ir::mir::extract_amount::ExtractAmount;
    // use ergotree_ir::mir::func_value::FuncArg;
    // use ergotree_ir::mir::func_value::FuncValue;
    use ergotree_ir::mir::property_call::PropertyCall;
    // use ergotree_ir::mir::val_use::ValUse;
    use ergotree_ir::types::scontext;

    use proptest::prelude::*;

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(16))]

        #[test]
        fn eval(vec in any::<Vec<i32>>(), from_i in 0..10i32, until_i in 0..10i32) {
            let expr: Expr = Slice::new(vec.clone().into(), from_i.into(), until_i.into())
                .unwrap().into();
            let res = try_eval_out_wo_ctx::<Vec<i32>>(&expr);
            prop_assert_eq!(res, Ok(vec));
        }
    }
}
