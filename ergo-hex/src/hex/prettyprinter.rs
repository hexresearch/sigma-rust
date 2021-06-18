use pretty;
use pretty::RcDoc;

use ergo_lib::ergotree_ir::mir::bin_op::BinOp;
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
use ergotree_ir::mir::value::Value;
use ergotree_ir::sigma_protocol::sigma_boolean::{
    ProveDlog, SigmaBoolean, SigmaProofOfKnowledgeTree, SigmaProp,
};
use ergotree_ir::types::sfunc::SFunc;
use ergotree_ir::types::smethod::SMethod;
use ergotree_ir::types::stype::SType;

pub fn ppr<T: Pretty>(expr: &T, width: usize) -> String {
    let mut w = Vec::new();
    expr.pretty().render(width, &mut w).unwrap();
    String::from_utf8(w).unwrap()
}

pub trait Pretty {
    fn pretty(&self) -> pretty::RcDoc;
}

impl Pretty for Expr {
    fn pretty(&self) -> pretty::RcDoc {
        match self {
            Expr::Const(c) => c.pretty(),
            Expr::ConstPlaceholder(e) => e.pretty(),
            Expr::SubstConstants(e) => e.pretty(),
            Expr::ByteArrayToLong(_) => todo!(),
            Expr::ByteArrayToBigInt(_) => todo!(),
            Expr::LongToByteArray(_) => todo!(),
            Expr::Collection(e) => e.pretty(),
            Expr::Tuple(e) => e.pretty(),
            Expr::CalcBlake2b256(e) => e.pretty(),
            Expr::CalcSha256(_) => todo!(),
            Expr::Context => todo!(),
            Expr::Global => RcDoc::text("GLOBAL"),
            Expr::GlobalVars(e) => e.pretty(),
            Expr::FuncValue(e) => e.pretty(),
            Expr::Apply(e) => e.pretty(),
            Expr::MethodCall(_) => todo!(),
            Expr::ProperyCall(e) => e.pretty(),
            Expr::BlockValue(e) => e.pretty(),
            Expr::ValDef(e) => e.pretty(),
            Expr::ValUse(e) => e.pretty(),
            Expr::If(e) => e.pretty(),
            Expr::BinOp(op) => op.pretty(),
            Expr::And(e) => e.pretty(),
            Expr::Or(e) => e.pretty(),
            Expr::Xor(_) => todo!(),
            Expr::Atleast(_) => todo!(),
            Expr::LogicalNot(_) => todo!(),
            Expr::Negation(_) => todo!(),
            Expr::OptionGet(e) => e.pretty(),
            Expr::OptionIsDefined(_) => todo!(),
            Expr::OptionGetOrElse(_) => todo!(),
            Expr::ExtractAmount(e) => e.pretty(),
            Expr::ExtractRegisterAs(e) => e.pretty(),
            Expr::ExtractScriptBytes(e) => e.pretty(),
            Expr::ExtractCreationInfo(e) => e.pretty(),
            Expr::ExtractId(_) => todo!(),
            Expr::ByIndex(e) => e.pretty(),
            Expr::SizeOf(e) => e.pretty(),
            Expr::Slice(_) => todo!(),
            Expr::Fold(_) => todo!(),
            Expr::Map(_) => todo!(),
            Expr::Append(_) => todo!(),
            Expr::Filter(_) => todo!(),
            Expr::Exists(_) => todo!(),
            Expr::ForAll(e) => e.pretty(),
            Expr::SelectField(e) => e.pretty(),
            Expr::BoolToSigmaProp(e) => e.pretty(),
            Expr::Upcast(_) => todo!(),
            Expr::CreateProveDlog(e) => e.pretty(),
            Expr::CreateProveDhTuple(e) => e.pretty(),
            Expr::SigmaPropBytes(_) => todo!(),
            Expr::DecodePoint(e) => e.pretty(),
            Expr::SigmaAnd(e) => e.pretty(),
            Expr::SigmaOr(e) => e.pretty(),
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
                .append(e.pretty())
                .append(RcDoc::text(")"))
                .nest(2),
        )
        .group()
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
    fn pretty(&self) -> pretty::RcDoc {
        self.v.pretty()
    }
}
impl Pretty for Apply {
    fn pretty(&self) -> pretty::RcDoc {
        let args = RcDoc::intersperse(
            self.args.iter().map(|e| e.pretty()),
            RcDoc::text(", ").append(RcDoc::line_()),
        );
        let func = self
            .func
            .pretty()
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
    fn pretty(&self) -> pretty::RcDoc {
        // Render let expression
        let items = self
            .items
            .iter()
            .map(|e| e.pretty().nest(2).append(RcDoc::line()))
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
            .append(self.result.pretty().nest(2));
        items.append(body)
    }
}
impl Pretty for PropertyCall {
    fn pretty(&self) -> pretty::RcDoc {
        let e = self.obj.pretty();
        let method = self.method.pretty();
        e.append(RcDoc::text("["))
            .append(method)
            .append(RcDoc::text("]"))
    }
}

impl Pretty for If {
    fn pretty(&self) -> pretty::RcDoc {
        ternary(
            "IF",
            self.condition.pretty(),
            self.true_branch.pretty(),
            self.false_branch.pretty(),
        )
    }
}
impl Pretty for Tuple {
    fn pretty(&self) -> pretty::RcDoc {
        parens_comma(self.items.iter().map(|e| e.pretty()))
    }
}
impl Pretty for FuncValue {
    fn pretty(&self) -> pretty::RcDoc {
        let tpe = self.tpe.pretty();
        let args = RcDoc::text("\\(")
            .append(RcDoc::intersperse(
                self.args.iter().map(|x| x.pretty()),
                RcDoc::text(","),
            ))
            .append(RcDoc::text(") -> "))
            .append(tpe)
            .append(RcDoc::line());
        let body = self.body.pretty().nest(2);
        args.append(body)
    }
}
impl Pretty for ValDef {
    fn pretty(&self) -> pretty::RcDoc {
        RcDoc::text("LET[")
            .append(RcDoc::as_string(self.id.0))
            .append(RcDoc::text("]("))
            .append(RcDoc::line_())
            .append(self.rhs.pretty())
            .append(RcDoc::text(")"))
    }
}

impl Pretty for ValUse {
    fn pretty(&self) -> pretty::RcDoc {
        // FIXME:
        RcDoc::text("VALUSE")
    }
}

impl Pretty for ForAll {
    fn pretty(&self) -> pretty::RcDoc {
        ternary(
            "ForAll",
            self.input.pretty(),
            self.condition.pretty(),
            self.elem_tpe.pretty(),
        )
    }
}

impl Pretty for SubstConstants {
    fn pretty(&self) -> pretty::RcDoc {
        let inner = RcDoc::line_()
            .append(self.script_bytes.pretty())
            .append(RcDoc::text(","))
            .append(RcDoc::line())
            .append(self.positions.pretty())
            .append(RcDoc::text(","))
            .append(RcDoc::line())
            .append(self.new_values.pretty())
            .append(RcDoc::text(")"))
            .nest(2)
            .group();
        RcDoc::text("SubstConst(").append(inner)
    }
}

impl Pretty for ExtractRegisterAs {
    fn pretty(&self) -> pretty::RcDoc {
        let inner = RcDoc::line_()
            .append(self.input.pretty())
            .append(RcDoc::text(","))
            .append(RcDoc::line())
            .append(RcDoc::as_string(self.register_id))
            .append(RcDoc::text(","))
            .append(RcDoc::line())
            .append(self.elem_tpe.pretty())
            .append(RcDoc::text(")"))
            .nest(2)
            .group();
        RcDoc::text("ExtractRegisterAs(").append(inner)
    }
}
impl Pretty for ByIndex {
    fn pretty(&self) -> pretty::RcDoc {
        let inner = RcDoc::line_()
            .append(self.input.pretty())
            .append(RcDoc::text(","))
            .append(RcDoc::line())
            .append(self.index.pretty());
        let inner = match &self.default {
            Some(e) => inner
                .append(RcDoc::text(","))
                .append(RcDoc::line())
                .append(e.pretty())
                .append(RcDoc::text(")")),
            None => inner.append(RcDoc::text(")")),
        }
        .nest(2)
        .group();
        RcDoc::text("ByIndex(").append(inner)
    }
}
impl Pretty for Collection {
    fn pretty(&self) -> pretty::RcDoc {
        match self {
            Collection::BoolConstants(_) => todo!(),
            Collection::Exprs { elem_tpe, items } => RcDoc::text("[")
                .append(elem_tpe.pretty())
                .append(RcDoc::text("|"))
                .append(
                    RcDoc::line()
                        .append(RcDoc::intersperse(
                            items.iter().map(|x| x.pretty()),
                            RcDoc::text(",").append(RcDoc::line()),
                        ))
                        .nest(2)
                        .group(),
                ),
        }
    }
}

impl Pretty for ConstantPlaceholder {
    fn pretty(&self) -> pretty::RcDoc {
        RcDoc::text("ConstPlaceholder(")
            .append(RcDoc::as_string(self.id))
            .append(RcDoc::text(","))
            .append(self.tpe.pretty())
            .append(RcDoc::text(")"))
    }
}

impl Pretty for ExtractCreationInfo {
    fn pretty(&self) -> pretty::RcDoc {
        unary("ExtractCreationInfo", &self.input)
    }
}

impl Pretty for ExtractAmount {
    fn pretty(&self) -> pretty::RcDoc {
        unary("ExtractAmount", &self.input)
    }
}

impl Pretty for OptionGet {
    fn pretty(&self) -> pretty::RcDoc {
        unary("OptionGet", &self.input)
    }
}

impl Pretty for SizeOf {
    fn pretty(&self) -> pretty::RcDoc {
        unary("SizeOf", &self.input)
    }
}

impl Pretty for CalcBlake2b256 {
    fn pretty(&self) -> pretty::RcDoc {
        unary("blake2b", &self.input)
    }
}

impl Pretty for DecodePoint {
    fn pretty(&self) -> pretty::RcDoc {
        unary("DecodePoint", &self.input)
    }
}
impl Pretty for CreateProveDlog {
    fn pretty(&self) -> pretty::RcDoc {
        unary("CreateDLog", &self.input)
    }
}
impl Pretty for CreateProveDhTuple {
    fn pretty(&self) -> pretty::RcDoc {
        quaternary(
            "CreateDHTuple",
            self.gv.pretty(),
            self.hv.pretty(),
            self.vv.pretty(),
            self.uv.pretty(),
        )
    }
}
impl Pretty for ExtractScriptBytes {
    fn pretty(&self) -> pretty::RcDoc {
        unary("ExtractScriptBytes", &self.input)
    }
}

impl Pretty for And {
    fn pretty(&self) -> pretty::RcDoc {
        unary("And", &self.input)
    }
}
impl Pretty for Or {
    fn pretty(&self) -> pretty::RcDoc {
        unary("Or", &self.input)
    }
}

impl Pretty for SelectField {
    fn pretty(&self) -> pretty::RcDoc {
        self.input
            .pretty()
            .append(RcDoc::text("["))
            .append(RcDoc::as_string(self.field_index.zero_based_index()))
            .append("]")
            .group()
    }
}
impl Pretty for GlobalVars {
    fn pretty(&self) -> pretty::RcDoc {
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
    fn pretty(&self) -> pretty::RcDoc {
        unary("BoolToSigma", &self.input)
    }
}

impl Pretty for BinOp {
    fn pretty(&self) -> pretty::RcDoc {
        let left = self.left.pretty();
        let left = RcDoc::text("(").append(left).append(")").group();
        let right = self.right.pretty();
        let right = RcDoc::text("(").append(right).append(")").group();
        let op = self.kind.pretty();

        left.append(op).append(RcDoc::line_()).append(right).group()
    }
}

impl Pretty for BinOpKind {
    fn pretty(&self) -> pretty::RcDoc {
        match self {
            BinOpKind::Arith(op) => op.pretty(),
            BinOpKind::Relation(op) => op.pretty(),
        }
    }
}

impl Pretty for RelationOp {
    fn pretty(&self) -> pretty::RcDoc {
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
    fn pretty(&self) -> pretty::RcDoc {
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
    fn pretty(&self) -> pretty::RcDoc {
        RcDoc::text("SigmaAnd(")
            .append(
                RcDoc::line()
                    .append(RcDoc::intersperse(
                        self.items.iter().map(|x| x.pretty()),
                        RcDoc::text(",").append(RcDoc::line()),
                    ))
                    .nest(2)
                    .group(),
            )
            .append(")")
    }
}

impl Pretty for SigmaOr {
    fn pretty(&self) -> pretty::RcDoc {
        RcDoc::text("SigmaOr(")
            .append(
                RcDoc::line()
                    .append(RcDoc::intersperse(
                        self.items.iter().map(|x| x.pretty()),
                        RcDoc::text(",").append(RcDoc::line()),
                    ))
                    .nest(2)
                    .group(),
            )
            .append(")")
    }
}

impl Pretty for Value {
    fn pretty(&self) -> pretty::RcDoc {
        match self {
            Value::Boolean(_) => todo!(),
            Value::Byte(_) => todo!(),
            Value::Short(_) => todo!(),
            Value::Int(_) => todo!(),
            Value::Long(_) => todo!(),
            Value::BigInt(_) => todo!(),
            Value::GroupElement(_) => todo!(),
            Value::SigmaProp(p) => p.pretty(),
            Value::CBox(_) => todo!(),
            Value::AvlTree => todo!(),
            Value::Coll(_) => todo!(),
            Value::Tup(_) => todo!(),
            Value::Context => todo!(),
            Value::Global => todo!(),
            Value::Opt(_) => todo!(),
            Value::Lambda(_) => todo!(),
        }
    }
}

impl Pretty for SigmaProp {
    fn pretty(&self) -> pretty::RcDoc {
        self.value().pretty()
    }
}

impl Pretty for SigmaBoolean {
    fn pretty(&self) -> pretty::RcDoc {
        match self {
            SigmaBoolean::TrivialProp(_) => todo!(),
            SigmaBoolean::ProofOfKnowledge(leaf) => leaf.pretty(),
            SigmaBoolean::SigmaConjecture(_) => todo!(),
        }
    }
}

impl Pretty for SigmaProofOfKnowledgeTree {
    fn pretty(&self) -> pretty::RcDoc {
        match self {
            SigmaProofOfKnowledgeTree::ProveDhTuple(_) => todo!(),
            SigmaProofOfKnowledgeTree::ProveDlog(p) => p.pretty(),
        }
    }
}

impl Pretty for ProveDlog {
    fn pretty(&self) -> pretty::RcDoc {
        RcDoc::text("DLog(")
            .append(RcDoc::text(format!("{:?}", self.h)))
            .append(RcDoc::text(")"))
    }
}

impl Pretty for SType {
    fn pretty(&self) -> pretty::RcDoc {
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
            SType::SOption(_) => todo!(),
            SType::SColl(e) => RcDoc::text("Coll<")
                .append(e.pretty())
                .append(RcDoc::text(">")),
            // FIXME:
            SType::STuple(_) => RcDoc::text("TUPLE"),
            SType::SFunc(e) => e.pretty(),
            SType::SContext => todo!(),
            SType::SHeader => todo!(),
            SType::SPreHeader => todo!(),
            SType::SGlobal => todo!(),
        }
    }
}

impl Pretty for SFunc {
    fn pretty(&self) -> RcDoc {
        // FIXME
        RcDoc::text("FUNC")
    }
}

impl Pretty for SMethod {
    fn pretty(&self) -> RcDoc {
        // FIXME
        RcDoc::text("SMETHOD")
    }
}

impl Pretty for FuncArg {
    fn pretty(&self) -> RcDoc {
        RcDoc::as_string(self.idx.0)
            .append(RcDoc::text(":"))
            .append(self.tpe.pretty())
    }
}
