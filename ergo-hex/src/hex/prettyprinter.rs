use pretty;
use pretty::RcDoc;

use ergo_lib::ergotree_ir::mir::bin_op::BinOp;
use ergo_lib::ergotree_ir::mir::value::CollKind;
use ergotree_ir::mir::and::And;
use ergotree_ir::mir::apply::Apply;
use ergotree_ir::mir::bin_op::{ArithOp, BinOpKind, RelationOp};
use ergotree_ir::mir::block::BlockValue;
use ergotree_ir::mir::bool_to_sigma::BoolToSigmaProp;
use ergotree_ir::mir::calc_blake2b256::CalcBlake2b256;
use ergotree_ir::mir::coll_by_index::ByIndex;
use ergotree_ir::mir::coll_forall::ForAll;
use ergotree_ir::mir::coll_size::SizeOf;
use ergotree_ir::mir::collection::Collection;
use ergotree_ir::mir::constant::{Constant, ConstantPlaceholder};
use ergotree_ir::mir::create_prove_dh_tuple::CreateProveDhTuple;
use ergotree_ir::mir::create_provedlog::CreateProveDlog;
use ergotree_ir::mir::decode_point::DecodePoint;
use ergotree_ir::mir::expr::Expr;
use ergotree_ir::mir::extract_amount::ExtractAmount;
use ergotree_ir::mir::extract_creation_info::ExtractCreationInfo;
use ergotree_ir::mir::extract_reg_as::ExtractRegisterAs;
use ergotree_ir::mir::extract_script_bytes::ExtractScriptBytes;
use ergotree_ir::mir::func_value::{FuncArg, FuncValue};
use ergotree_ir::mir::global_vars::GlobalVars;
use ergotree_ir::mir::if_op::If;
use ergotree_ir::mir::option_get::OptionGet;
use ergotree_ir::mir::or::Or;
use ergotree_ir::mir::property_call::PropertyCall;
use ergotree_ir::mir::select_field::SelectField;
use ergotree_ir::mir::sigma_and::SigmaAnd;
use ergotree_ir::mir::sigma_or::SigmaOr;
use ergotree_ir::mir::subst_const::SubstConstants;
use ergotree_ir::mir::tuple::Tuple;
use ergotree_ir::mir::val_def::ValDef;
use ergotree_ir::mir::val_use::ValUse;
use ergotree_ir::mir::value::{NativeColl, Value};
use ergotree_ir::sigma_protocol::sigma_boolean::{
    ProveDlog, SigmaBoolean, SigmaProofOfKnowledgeTree, SigmaProp,
};
use ergotree_ir::types::sfunc::SFunc;
use ergotree_ir::types::smethod::SMethod;
use ergotree_ir::types::stype::SType;
use std::fmt::Display;
use ergotree_ir::mir::upcast::Upcast;
use ergotree_ir::mir::coll_exists::Exists;

pub fn ppr<T: Pretty>(expr: &T, width: usize) -> String {
    let mut w = Vec::new();
    expr.pretty().render(width, &mut w).unwrap();
    String::from_utf8(w).unwrap()
}

pub trait Pretty {
    fn pretty_binop(&self, is_binop: bool) -> pretty::RcDoc;
    fn pretty(&self) -> pretty::RcDoc {
        self.pretty_binop(false)
    }
}

// impl Pretty for Expr {
//     fn pretty_binop(&self, bop: bool) -> pretty::RcDoc {
//         (&self).pretty_binop(bop)
//     }
// }

impl Pretty for Expr {
    fn pretty_binop(&self, bop: bool) -> pretty::RcDoc {
        match self {
            Expr::Const(c) => c.pretty_binop(bop),
            Expr::ConstPlaceholder(e) => e.pretty_binop(bop),
            Expr::SubstConstants(e) => e.pretty_binop(bop),
            Expr::ByteArrayToLong(_) => todo!(),
            Expr::ByteArrayToBigInt(_) => todo!(),
            Expr::LongToByteArray(_) => todo!(),
            Expr::Collection(e) => e.pretty_binop(bop),
            Expr::Tuple(e) => e.pretty_binop(bop),
            Expr::CalcBlake2b256(e) => e.pretty_binop(bop),
            Expr::CalcSha256(_) => todo!(),
            Expr::Context => todo!(),
            Expr::Global => RcDoc::text("GLOBAL"),
            Expr::GlobalVars(e) => e.pretty_binop(bop),
            Expr::FuncValue(e) => e.pretty_binop(bop),
            Expr::Apply(e) => e.pretty_binop(bop),
            Expr::MethodCall(_) => todo!(),
            Expr::ProperyCall(e) => e.pretty_binop(bop),
            Expr::BlockValue(e) => e.pretty_binop(bop),
            Expr::ValDef(e) => e.pretty_binop(bop),
            Expr::ValUse(e) => e.pretty_binop(bop),
            Expr::If(e) => e.pretty_binop(bop),
            Expr::BinOp(op) => op.pretty_binop(bop),
            Expr::And(e) => e.pretty_binop(bop),
            Expr::Or(e) => e.pretty_binop(bop),
            Expr::Xor(_) => todo!(),
            Expr::Atleast(_) => todo!(),
            Expr::LogicalNot(_) => todo!(),
            Expr::Negation(_) => todo!(),
            Expr::OptionGet(e) => e.pretty_binop(bop),
            Expr::OptionIsDefined(_) => todo!(),
            Expr::OptionGetOrElse(_) => todo!(),
            Expr::ExtractAmount(e) => e.pretty_binop(bop),
            Expr::ExtractRegisterAs(e) => e.pretty_binop(bop),
            Expr::ExtractScriptBytes(e) => e.pretty_binop(bop),
            Expr::ExtractCreationInfo(e) => e.pretty_binop(bop),
            Expr::ExtractId(_) => todo!(),
            Expr::ByIndex(e) => e.pretty_binop(bop),
            Expr::SizeOf(e) => e.pretty_binop(bop),
            Expr::Slice(_) => todo!(),
            Expr::Fold(_) => todo!(),
            Expr::Map(_) => todo!(),
            Expr::Append(_) => todo!(),
            Expr::Filter(_) => todo!(),
            Expr::Exists(e) => e.pretty_binop(bop),
            Expr::ForAll(e) => e.pretty_binop(bop),
            Expr::SelectField(e) => e.pretty_binop(bop),
            Expr::BoolToSigmaProp(e) => e.pretty_binop(bop),
            Expr::Upcast(e) => e.pretty_binop(bop),
            Expr::CreateProveDlog(e) => e.pretty_binop(bop),
            Expr::CreateProveDhTuple(e) => e.pretty_binop(bop),
            Expr::SigmaPropBytes(_) => todo!(),
            Expr::DecodePoint(e) => e.pretty_binop(bop),
            Expr::SigmaAnd(e) => e.pretty_binop(bop),
            Expr::SigmaOr(e) => e.pretty_binop(bop),
            Expr::GetVar(_) => todo!(),
            Expr::DeserializeRegister(_) => todo!(),
            Expr::DeserializeContext(_) => todo!(),
            Expr::MultiplyGroup(_) => todo!(),
        }
    }
}

fn unary<'a>(fun: &'a str, e: &'a Expr) -> pretty::RcDoc<'a> {
    RcDoc::text(fun)
        .append(RcDoc::text("("))
        .append(
            RcDoc::line_()
                .append(e.pretty_binop(false))
                .append(RcDoc::line_())
                .append(RcDoc::text(")"))
                .nest(2),
        )
        .group()
}

fn binary<'a>(name: &'a str, a: RcDoc<'a>, b: RcDoc<'a>) -> pretty::RcDoc<'a> {
    let inner = RcDoc::line_()
        .append(a)
        .append(RcDoc::text(","))
        .append(RcDoc::line())
        .append(b)
        .append(RcDoc::line_())
        .append(RcDoc::text(")"))
        .nest(2)
        .group();
    RcDoc::text(name).append(RcDoc::text("(")).append(inner)
}

fn ternary<'a>(name: &'a str, a: RcDoc<'a>, b: RcDoc<'a>, c: RcDoc<'a>) -> pretty::RcDoc<'a> {
    let inner = RcDoc::line_()
        .append(a)
        .append(RcDoc::text(","))
        .append(RcDoc::line())
        .append(b)
        .append(RcDoc::text(","))
        .append(RcDoc::line())
        .append(c)
        .append(RcDoc::line_())
        .append(RcDoc::text(")"))
        .nest(2)
        .group();
    RcDoc::text(name).append(RcDoc::text("(")).append(inner)
}

fn quaternary<'a>(
    name: &'a str,
    a: RcDoc<'a>,
    b: RcDoc<'a>,
    c: RcDoc<'a>,
    d: RcDoc<'a>,
) -> pretty::RcDoc<'a> {
    let inner = RcDoc::line_()
        .append(a)
        .append(RcDoc::text(","))
        .append(RcDoc::line())
        .append(b)
        .append(RcDoc::text(","))
        .append(RcDoc::line())
        .append(c)
        .append(RcDoc::text(","))
        .append(RcDoc::line())
        .append(d)
        .append(RcDoc::line_())
        .append(RcDoc::text(")"))
        .nest(2)
        .group();
    RcDoc::text(name).append(RcDoc::text("(")).append(inner)
}

fn comma<'a, I, A>(items: I) -> RcDoc<'a, A>
where
    I: Iterator,
    I::Item: Into<pretty::BuildDoc<'a, RcDoc<'a, A>, A>>,
    A: Clone,
{
    RcDoc::intersperse(items, RcDoc::text(", ").append(RcDoc::line_())).group()
}

fn parens_comma<'a, I, A>(items: I) -> RcDoc<'a, A>
where
    I: Iterator,
    I::Item: Into<pretty::BuildDoc<'a, RcDoc<'a, A>, A>>,
    A: Clone,
{
    RcDoc::text("(")
        .append(RcDoc::line_())
        .append(comma(items).append(RcDoc::text(")")).nest(2))
        .group()
}

impl Pretty for Constant {
    fn pretty_binop(&self, bop: bool) -> pretty::RcDoc {
        self.v.pretty_binop(bop)
    }
}
impl Pretty for Apply {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        let args = RcDoc::intersperse(
            self.args.iter().map(|e| e.pretty_binop(false)),
            RcDoc::text(", ").append(RcDoc::line_()),
        );
        let func = self
            .func
            .pretty_binop(false)
            .append(RcDoc::text("; "))
            .append(RcDoc::line_())
            .append(args)
            .nest(2)
            .group();

        RcDoc::text("CALL(")
            .append(RcDoc::line_())
            .append(func)
            .append(RcDoc::text(")"))
            .group()
    }
}

impl Pretty for BlockValue {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        // Render let expression
        let items = self
            .items
            .iter()
            .map(|e| e.pretty_binop(false).nest(2).append(RcDoc::line()))
            .zip(0..)
            .map(|(e, n)| {
                RcDoc::text("let [")
                    .append(RcDoc::as_string(n))
                    .append(RcDoc::text("]"))
                    .append(RcDoc::line())
                    .append(e)
            });
        // .collect::<Vec<(RcDoc, usize)>>();
        let items = RcDoc::concat(items);
        let body = RcDoc::text("in")
            .append(RcDoc::line())
            .append(self.result.pretty_binop(false).nest(2));
        items.append(body)
    }
}
impl Pretty for PropertyCall {
    fn pretty_binop(&self, bop: bool) -> pretty::RcDoc {
        let e = self.obj.pretty_binop(bop);
        let method = self.method.name();
        e.append(RcDoc::text("%"))
            .append(RcDoc::text(method))
    }
}

impl Pretty for If {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        ternary(
            "IF",
            self.condition.pretty_binop(false),
            self.true_branch.pretty_binop(false),
            self.false_branch.pretty_binop(false),
        )
    }
}
impl Pretty for Tuple {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        parens_comma(self.items.iter().map(|e| e.pretty()))
    }
}
impl Pretty for FuncValue {
    fn pretty_binop(&self, bop: bool) -> pretty::RcDoc {
        let tpe = self.tpe.pretty_binop(bop);
        let args = RcDoc::text("\\(")
            .append(RcDoc::intersperse(
                self.args.iter().map(|x| x.pretty_binop(bop)),
                RcDoc::text(","),
            ))
            .append(RcDoc::text(") -> "))
            .append(tpe)
            .append(RcDoc::line());
        let body = self.body.pretty_binop(bop).nest(2);
        args.append(body)
    }
}
impl Pretty for ValDef {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        RcDoc::text("LET[")
            .append(RcDoc::as_string(self.id.0))
            .append(RcDoc::text("]("))
            .append(RcDoc::line_())
            .append(self.rhs.pretty_binop(false))
            .append(RcDoc::text(")"))
    }
}

impl Pretty for ValUse {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        binary("ValUse", RcDoc::as_string(self.val_id.0),
        self.tpe.pretty())
    }
}

impl Pretty for ForAll {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        ternary(
            "ForAll",
            self.input.pretty_binop(false),
            self.condition.pretty_binop(false),
            self.elem_tpe.pretty_binop(false),
        )
    }
}

impl Pretty for Exists {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        ternary(
            "Exists",
            self.input.pretty_binop(false),
            self.condition.pretty_binop(false),
            self.elem_tpe.pretty_binop(false),
        )
    }
}

impl Pretty for SubstConstants {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        ternary(
            "SubstConst",
            self.script_bytes.pretty_binop(false),
            self.positions.pretty_binop(false),
            self.new_values.pretty_binop(false),
        )
    }
}

impl Pretty for ExtractRegisterAs {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        let inner = RcDoc::line_()
            .append(self.input.pretty_binop(false))
            .append(RcDoc::text(","))
            .append(RcDoc::line())
            .append(RcDoc::as_string(self.register_id))
            .append(RcDoc::text(","))
            .append(RcDoc::line())
            .append(self.elem_tpe.pretty_binop(false))
            .append(RcDoc::text(")"))
            .nest(2)
            .group();
        RcDoc::text("ExtractRegisterAs(").append(inner)
    }
}
impl Pretty for ByIndex {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        match &self.default {
            Some(def) => ternary(
                "ByIndex",
                self.input.pretty_binop(false),
                self.index.pretty_binop(false),
                def.pretty_binop(false),
            ),
            None => binary(
                "ByIndex",
                self.input.pretty_binop(false),
                self.index.pretty_binop(false),
            ),
        }
    }
}

fn simple_coll<'a, T: Display>(ty: &'a SType, elems: &'a [T]) -> RcDoc<'a> {
    RcDoc::text("[")
        .append(ty.pretty_binop(false))
        .append(RcDoc::text("'|"))
        .append(
            RcDoc::line_()
                .append(comma(elems.iter().map(|x| RcDoc::as_string(x))))
                .append(RcDoc::text("]"))
                .nest(2)
                .group(),
        )
}

fn typed_coll<'a, T: Pretty>(ty: &'a SType, elems: &'a Vec<T>) -> RcDoc<'a> {
    RcDoc::text("[")
        .append(ty.pretty_binop(false))
        .append(RcDoc::text("|"))
        .append(
            RcDoc::line_()
                .append(comma(elems.iter().map(|x| x.pretty())))
                .append(RcDoc::text("]"))
                .nest(2)
                .group(),
        )
}

impl Pretty for Collection {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        match self {
            Collection::BoolConstants(bools) => simple_coll(&SType::SBoolean, bools),
            Collection::Exprs { elem_tpe, items } => typed_coll(elem_tpe, items),
        }
    }
}

impl Pretty for ConstantPlaceholder {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        RcDoc::text("ConstPlaceholder(")
            .append(RcDoc::as_string(self.id))
            .append(RcDoc::text(","))
            .append(self.tpe.pretty_binop(false))
            .append(RcDoc::text(")"))
    }
}

impl Pretty for ExtractCreationInfo {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        unary("ExtractCreationInfo", &self.input)
    }
}

impl Pretty for Upcast {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        unary("Upcast", &self.input)
    }
}

impl Pretty for ExtractAmount {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        unary("ExtractAmount", &self.input)
    }
}

impl Pretty for OptionGet {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        unary("OptionGet", &self.input)
    }
}

impl Pretty for SizeOf {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        unary("SizeOf", &self.input)
    }
}

impl Pretty for CalcBlake2b256 {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        unary("blake2b", &self.input)
    }
}

impl Pretty for DecodePoint {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        unary("DecodePoint", &self.input)
    }
}
impl Pretty for CreateProveDlog {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        unary("CreateDLog", &self.input)
    }
}
impl Pretty for CreateProveDhTuple {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        quaternary(
            "CreateDHTuple",
            self.gv.pretty_binop(false),
            self.hv.pretty_binop(false),
            self.vv.pretty_binop(false),
            self.uv.pretty_binop(false),
        )
    }
}
impl Pretty for ExtractScriptBytes {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        unary("ExtractScriptBytes", &self.input)
    }
}

impl Pretty for And {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        unary("And", &self.input)
    }
}
impl Pretty for Or {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        unary("Or", &self.input)
    }
}

impl Pretty for SelectField {
    fn pretty_binop(&self, bop: bool) -> pretty::RcDoc {
        self.input
            .pretty_binop(bop)
            .append(RcDoc::text("["))
            .append(RcDoc::as_string(self.field_index.zero_based_index()))
            .append("]")
            .group()
    }
}
impl Pretty for GlobalVars {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        match self {
            GlobalVars::Inputs => RcDoc::text("INPUTS"),
            GlobalVars::Outputs => RcDoc::text("OUTPUTS"),
            GlobalVars::Height => RcDoc::text("HEIGHT"),
            GlobalVars::SelfBox => RcDoc::text("SELF"),
            GlobalVars::MinerPubKey => RcDoc::text("MINER_PK"),
        }
    }
}
impl Pretty for BoolToSigmaProp {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        unary("BoolToSigma", &self.input)
    }
}

impl Pretty for BinOp {
    fn pretty_binop(&self, bop: bool) -> pretty::RcDoc {
        let left = self.left.pretty_binop(true);
        let left = if bop {
            RcDoc::text("(").append(left).append(")")
        } else {
            left
        };
        let right = self.right.pretty_binop(true);
        let right = if bop {
            RcDoc::text("(").append(right).append(")")
        } else {
            right
        };
        let op = self.kind.pretty_binop(true);
        left.append(RcDoc::text(" "))
            .append(op)
            .append(RcDoc::text(" "))
            .append(RcDoc::line_().append(right).nest(2))
            .group()
    }
}

impl Pretty for BinOpKind {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        match self {
            BinOpKind::Arith(op) => op.pretty_binop(false),
            BinOpKind::Relation(op) => op.pretty_binop(false),
        }
    }
}

impl Pretty for RelationOp {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        match self {
            RelationOp::Eq => RcDoc::text("=="),
            RelationOp::NEq => RcDoc::text("/="),
            RelationOp::Ge => RcDoc::text(">="),
            RelationOp::Gt => RcDoc::text(">"),
            RelationOp::Le => RcDoc::text("<="),
            RelationOp::Lt => RcDoc::text("<"),
            RelationOp::And => RcDoc::text("&&"),
            RelationOp::Or => RcDoc::text("||"),
        }
    }
}

impl Pretty for ArithOp {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        match self {
            ArithOp::Plus => RcDoc::text("+"),
            ArithOp::Minus => RcDoc::text("-"),
            ArithOp::Multiply => RcDoc::text("*"),
            ArithOp::Divide => RcDoc::text("/"),
            ArithOp::Max => RcDoc::text("`max`"),
            ArithOp::Min => RcDoc::text("`min`"),
            ArithOp::BitOr => RcDoc::text("|"),
            ArithOp::BitAnd => RcDoc::text("&"),
            ArithOp::BitXor => RcDoc::text("^"),
        }
    }
}

impl Pretty for SigmaAnd {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        let items = comma(self.items.iter().map(|x| x.pretty()));
        RcDoc::text("SigmaAnd[")
            .append(
                RcDoc::line_()
                    .append(items)
                    .append(RcDoc::line_())
                    .append(RcDoc::text("]"))
                    .nest(2)
                    .group())
    }
}

impl Pretty for SigmaOr {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        let items = comma(self.items.iter().map(|x| x.pretty()));
        RcDoc::text("SigmaOr[")
            .append(
                RcDoc::line_()
                    .append(items)
                    .append(RcDoc::line_())
                    .append(RcDoc::text("]"))
                    .nest(2)
                    .group())
    }
}

impl Pretty for Value {
    fn pretty_binop(&self, bop: bool) -> pretty::RcDoc {
        match self {
            Value::Boolean(b) => RcDoc::as_string(b),
            Value::Byte(_) => todo!(),
            Value::Short(_) => todo!(),
            Value::Int(i) => RcDoc::as_string(i).append(RcDoc::text(":SInt")),
            Value::Long(i) => RcDoc::as_string(i).append(RcDoc::text(":SLong")),
            Value::BigInt(_) => todo!(),
            Value::GroupElement(_) => todo!(),
            Value::SigmaProp(p) => p.pretty_binop(bop),
            Value::CBox(_) => todo!(),
            Value::AvlTree => todo!(),
            Value::Coll(coll) => match coll {
                CollKind::NativeColl(NativeColl::CollByte(bytes)) => {
                    RcDoc::text("[BYTES|").append(RcDoc::text(base16::encode_lower(
                        &bytes.iter().map(|x| *x as u8).collect::<Vec<_>>(),
                    )))
                }
                CollKind::WrappedColl { elem_tpe, items } => typed_coll(elem_tpe, &items),
            },
            Value::Tup(_) => todo!(),
            Value::Context => todo!(),
            Value::Global => todo!(),
            Value::Opt(_) => todo!(),
            Value::Lambda(_) => todo!(),
        }
    }
}

impl Pretty for SigmaProp {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        self.value().pretty_binop(false)
    }
}

impl Pretty for SigmaBoolean {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        match self {
            SigmaBoolean::TrivialProp(_) => todo!(),
            SigmaBoolean::ProofOfKnowledge(leaf) => leaf.pretty_binop(false),
            SigmaBoolean::SigmaConjecture(_) => todo!(),
        }
    }
}

impl Pretty for SigmaProofOfKnowledgeTree {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        match self {
            SigmaProofOfKnowledgeTree::ProveDhTuple(_) => todo!(),
            SigmaProofOfKnowledgeTree::ProveDlog(p) => p.pretty_binop(false),
        }
    }
}

impl Pretty for ProveDlog {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        RcDoc::text("DLog(")
            .append(RcDoc::text(format!("{:?}", self.h)))
            .append(RcDoc::text(")"))
    }
}

impl Pretty for SType {
    fn pretty_binop(&self, _bop: bool) -> pretty::RcDoc {
        match self {
            SType::STypeVar(_) => todo!(),
            SType::SAny => todo!(),
            SType::SBoolean => RcDoc::text("SBool"),
            SType::SByte => RcDoc::text("SByte"),
            SType::SShort => RcDoc::text("SShortInt"),
            SType::SInt => RcDoc::text("SInt"),
            SType::SLong => RcDoc::text("SLongInt"),
            SType::SBigInt => RcDoc::text("SBigInt"),
            SType::SGroupElement => RcDoc::text("SGroupElem"),
            SType::SSigmaProp => RcDoc::text("SSigmaProp"),
            SType::SBox => RcDoc::text("SBox"),
            SType::SAvlTree => todo!(),
            SType::SOption(e) => RcDoc::text("Option<").append(e.pretty()).append(RcDoc::text(">")),
            SType::SColl(e) => RcDoc::text("Coll<")
                .append(e.pretty_binop(false))
                .append(RcDoc::text(">")),
            // FIXME:
            SType::STuple(_) => RcDoc::text("TUPLE"),
            SType::SFunc(e) => e.pretty_binop(false),
            SType::SContext => todo!(),
            SType::SHeader => todo!(),
            SType::SPreHeader => todo!(),
            SType::SGlobal => todo!(),
        }
    }
}

impl Pretty for SFunc {
    fn pretty_binop(&self, _bop: bool) -> RcDoc {
        // FIXME
        RcDoc::text("FUNC")
    }
}

impl Pretty for FuncArg {
    fn pretty_binop(&self, _bop: bool) -> RcDoc {
        RcDoc::as_string(self.idx.0)
            .append(RcDoc::text(":"))
            .append(self.tpe.pretty_binop(false))
    }
}
