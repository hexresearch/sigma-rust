use std::convert::{TryFrom, TryInto};
use ergo_lib::chain::transaction::Transaction;
use ergotree_ir::mir::expr::{Expr, ToBoxedExprExt};
use ergotree_ir::mir::and::And;
use ergotree_ir::mir::bin_op::{ArithOp, BinOp, BinOpKind, RelationOp};
use ergotree_ir::mir::block::BlockValue;
use ergotree_ir::mir::bool_to_sigma::BoolToSigmaProp;
use ergotree_ir::mir::coll_size::SizeOf;
use ergotree_ir::mir::collection::Collection;
use ergotree_ir::mir::constant::{Constant,ConstantPlaceholder};
use ergotree_ir::mir::extract_creation_info::ExtractCreationInfo;
use ergotree_ir::mir::extract_reg_as::ExtractRegisterAs;
use ergotree_ir::mir::global_vars::GlobalVars;
use ergotree_ir::mir::option_get::OptionGet;
use ergotree_ir::mir::select_field::{SelectField, TupleFieldIndex};
use ergotree_ir::mir::sigma_and::SigmaAnd;
use ergotree_ir::mir::sigma_or::SigmaOr;
use ergotree_ir::mir::val_def::ValId;
use ergotree_ir::mir::val_use::ValUse;
use ergotree_ir::mir::value::Value;
use ergotree_ir::mir::coll_by_index::ByIndex;
use ergotree_ir::mir::create_prove_dh_tuple::CreateProveDhTuple;
use ergotree_ir::mir::create_provedlog::CreateProveDlog;
use ergotree_ir::mir::decode_point::DecodePoint;
use ergotree_ir::mir::extract_script_bytes::ExtractScriptBytes;
use ergotree_ir::mir::subst_const::SubstConstants;
use ergotree_ir::types::stype::SType;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;

pub trait Deconstruct: Sized {
    fn match_and(self) -> Option<Self>;
    fn match_sigma_and(self) -> Option<Vec<Self>>;
    fn match_sigma_or(self) -> Option<Vec<Self>>;
    fn match_option_get(self) -> Option<Self>;
    fn match_extract_creation_info(self) -> Option<Self>;
    fn match_select_field(self, idx: u8) -> Option<Self>;
    fn match_bool_to_sigma(self) -> Option<Self>;
    fn match_collection(self, ty: &SType) -> Option<Vec<Self>>;
    fn match_valuse(self, id: u32, ty: SType) -> Option<()>;
    fn match_dlog(self) -> Option<Self>;
    fn match_dhtuple(self) -> Option<(Self, Self, Self, Self)>;
    fn match_extract_register(self, id: i8, ty: SType) -> Option<Self>;
    fn match_global_var(self, var: GlobalVars) -> Option<()>;
}

impl Deconstruct for &Expr {
    fn match_and(self) -> Option<Self> {
        match self {
            Expr::And(And { box input }) => Some(input),
            _ => None,
        }
    }
    fn match_sigma_and(self) -> Option<Vec<Self>> {
        match self {
            Expr::SigmaAnd(SigmaAnd { items }) => Some(items.iter().map(|x| x).collect()),
            _ => None,
        }
    }
    fn match_option_get(self) -> Option<Self> {
        match self {
            Expr::OptionGet(OptionGet { box input }) => Some(input),
            _ => None,
        }
    }
    fn match_sigma_or(self) -> Option<Vec<Self>> {
        match self {
            Expr::SigmaOr(SigmaOr { items }) => Some(items.iter().map(|x| x).collect()),
            _ => None,
        }
    }
    fn match_extract_creation_info(self) -> Option<Self> {
        match self {
            Expr::ExtractCreationInfo(ExtractCreationInfo { box input }) => Some(input),
            _ => None,
        }
    }
    fn match_select_field(self, idx: u8) -> Option<Self> {
        let idx: TupleFieldIndex = idx.try_into().unwrap();
        match self {
            Expr::SelectField(SelectField {
                                  input: box expr,
                                  field_index: i,
                              }) if i == &idx => Some(expr),
            _ => None,
        }
    }
    fn match_bool_to_sigma(self) -> Option<Self> {
        match self {
            Expr::BoolToSigmaProp(BoolToSigmaProp { box input }) => Some(input),
            _ => None,
        }
    }
    fn match_collection(self, ty: &SType) -> Option<Vec<Self>> {
        match self {
            Expr::Collection(Collection::Exprs { elem_tpe, items }) if elem_tpe == ty => {
                Some(items.iter().map(|x| x).collect())
            }
            _ => None,
        }
    }

    fn match_valuse(self, id: u32, ty: SType) -> Option<()> {
        match self {
            Expr::ValUse(ValUse { val_id, tpe }) if val_id == &ValId(id) && tpe == &ty => Some(()),
            _ => None,
        }
    }

    fn match_dlog(self) -> Option<Self> {
        match self {
            Expr::CreateProveDlog(CreateProveDlog { box input }) => Some(input),
            _ => None,
        }
    }
    fn match_dhtuple(self) -> Option<(Self, Self, Self, Self)> {
        match self {
            Expr::CreateProveDhTuple(CreateProveDhTuple {
                                         box gv,
                                         box hv,
                                         box uv,
                                         box vv,
                                     }) => Some((gv, hv, vv, uv)),
            _ => None,
        }
    }

    fn match_extract_register(self, id: i8, ty: SType) -> Option<Self> {
        match self {
            Expr::ExtractRegisterAs(ExtractRegisterAs {
                                        box input,
                                        register_id,
                                        elem_tpe,
                                    }) if register_id == &id && elem_tpe == &ty => Some(input),
            _ => None,
        }
    }

    fn match_global_var(self, var: GlobalVars) -> Option<()> {
        match self {
            Expr::GlobalVars(v) if v == &var => Some(()),
            _ => None,
        }
    }
}

pub fn expect(e1: &Expr, e2: &Expr) -> Option<()> {
    if e1 == e2 {
        Some(())
    } else {
        None
    }
}

pub fn match_const<'a>(e: &'a Expr, ty: &SType) -> Option<&'a Value> {
    match e {
        Expr::Const(Constant { tpe, v }) if tpe == ty => Some(v),
        _ => None,
    }
}

pub fn match_const_placeholder(e: &Expr, i: u32, ty: &SType) -> Option<()> {
    match e {
        Expr::ConstPlaceholder(ConstantPlaceholder { id, tpe }) if id == &i && tpe == ty => {
            Some(())
        }
        _ => None,
    }
}

pub fn match_block_value(e: &Expr) -> Option<(Vec<&Expr>, &Expr)> {
    match e {
        Expr::BlockValue(BlockValue { items, box result }) => {
            Some((items.iter().map(|x| x).collect(), result))
        }
        _ => None,
    }
}

pub fn match_binop(e: &Expr, op: BinOpKind) -> Option<(&Expr, &Expr)> {
    match e {
        Expr::BinOp(BinOp {
                        kind,
                        left: box a,
                        right: box b,
                    }) if kind == &op => Some((a, b)),
        _ => None,
    }
}

pub fn val_sigma_prop(v: &Value) -> Option<&SigmaBoolean> {
    match v {
        Value::SigmaProp(box p) => Some(p.value()),
        _ => None,
    }
}

pub fn match_vec3<T>(v: Vec<&T>) -> Option<(&T, &T, &T)> {
    match &v[..] {
        [a, b, c] => Some((a, b, c)),
        _ => None,
    }
}

pub fn match_vec2<T>(v: Vec<&T>) -> Option<(&T, &T)> {
    match &v[..] {
        [a, b] => Some((a, b)),
        _ => None,
    }
}