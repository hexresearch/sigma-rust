#![feature(box_patterns)]

use ergotree_ir::mir::expr::{Expr, ToBoxedExprExt};
use ergotree_ir::serialization::constant_store::ConstantStore;
use ergotree_ir::serialization::sigma_byte_reader::SigmaByteReader;
use ergotree_ir::serialization::SerializationError;
use ergotree_ir::serialization::SigmaSerializable;

use clap::{App, Arg};
use ergo_lib::chain::transaction::Transaction;
use ergo_lib::ergotree_ir::mir::constant::ConstantPlaceholder;
use ergotree_ir::mir::and::And;
use ergotree_ir::mir::bin_op::{ArithOp, BinOp, BinOpKind, RelationOp};
use ergotree_ir::mir::block::BlockValue;
use ergotree_ir::mir::bool_to_sigma::BoolToSigmaProp;
use ergotree_ir::mir::coll_size::SizeOf;
use ergotree_ir::mir::collection::Collection;
use ergotree_ir::mir::constant::Constant;
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
// use ergotree_ir::serialization::sigma_byte_writer::SigmaByteWriter;
// use ergotree_ir::serialization::SerializationError::Misc;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use ergotree_ir::sigma_protocol::sigma_boolean::{ProveDlog, SigmaProofOfKnowledgeTree, SigmaProp};
use ergotree_ir::types::stype::SType;
use rusqlite;
use sigma_ser::vlq_encode::ReadSigmaVlqExt;
// use sigma_ser::vlq_encode::WriteSigmaVlqExt;
use std::collections::BTreeMap;
// use std::io;
use ergotree_ir::mir::coll_by_index::ByIndex;
use ergotree_ir::mir::create_prove_dh_tuple::CreateProveDhTuple;
use ergotree_ir::mir::create_provedlog::CreateProveDlog;
use ergotree_ir::mir::decode_point::DecodePoint;
use ergotree_ir::mir::extract_script_bytes::ExtractScriptBytes;
use ergotree_ir::mir::subst_const::SubstConstants;
use std::io::Cursor;
use std::convert::TryInto;

// ---------------------------------------------------------------

trait Deconstruct: Sized {
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
// impl Deconstruct for Expr {
//     fn match_and(self) -> Option<Self> {
//         match self {
//             Expr::And(And { box input }) => Some(input),
//             _ => None,
//         }
//     }
// }

fn expect(e1: &Expr, e2: &Expr) -> Option<()> {
    if e1 == e2 {
        Some(())
    } else {
        None
    }
}

fn match_const<'a>(e: &'a Expr, ty: &SType) -> Option<&'a Value> {
    match e {
        Expr::Const(Constant { tpe, v }) if tpe == ty => Some(v),
        _ => None,
    }
}

fn match_const_placeholder(e: &Expr, i: u32, ty: &SType) -> Option<()> {
    match e {
        Expr::ConstPlaceholder(ConstantPlaceholder { id, tpe }) if id == &i && tpe == ty => {
            Some(())
        }
        _ => None,
    }
}

fn match_block_value(e: &Expr) -> Option<(Vec<&Expr>, &Expr)> {
    match e {
        Expr::BlockValue(BlockValue { items, box result }) => {
            Some((items.iter().map(|x| x).collect(), result))
        }
        _ => None,
    }
}
fn match_binop(e: &Expr, op: BinOpKind) -> Option<(&Expr, &Expr)> {
    match e {
        Expr::BinOp(BinOp {
            kind,
            left: box a,
            right: box b,
        }) if kind == &op => Some((a, b)),
        _ => None,
    }
}

fn val_sigma_prop(v: &Value) -> Option<&SigmaBoolean> {
    match v {
        Value::SigmaProp(box p) => Some(p.value()),
        _ => None,
    }
}

fn match_vec3<T>(v: Vec<&T>) -> Option<(&T, &T, &T)> {
    match &v[..] {
        [a, b, c] => Some((a, b, c)),
        _ => None,
    }
}
fn match_vec2<T>(v: Vec<&T>) -> Option<(&T, &T)> {
    match &v[..] {
        [a, b] => Some((a, b)),
        _ => None,
    }
}
// ---------------------------------------------------------------
fn is_36b_script(e: &Expr) -> Option<()> {
    let v = match_const(e, &SType::SSigmaProp)?;
    match val_sigma_prop(v)? {
        SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(ProveDlog {
            ..
        })) => Some(()),
        _ => None,
    }
}

fn is_54b_a(e: &Expr) -> Option<()> {
    let e = e.match_bool_to_sigma()?;
    let (a, b) = match_binop(e, BinOpKind::Relation(RelationOp::Ge))?;
    expect(a, &Expr::GlobalVars(GlobalVars::Height))?;
    let (b_1, b_2) = match_binop(b, BinOpKind::Arith(ArithOp::Plus))?;
    expect(
        b_1,
        &Expr::SelectField(SelectField {
            input: Expr::ExtractCreationInfo(ExtractCreationInfo {
                input: Expr::GlobalVars(GlobalVars::SelfBox).into(),
            })
            .into(),
            field_index: 1.try_into().unwrap(),
        }),
    )?;
    match_const_placeholder(b_2, 0, &SType::SInt)?;
    Some(())
}

fn is_54b_script(e: &Expr) -> Option<()> {
    match e {
        Expr::SigmaAnd(SigmaAnd { items }) =>
            if items.len() == 2 {
                let a = &items.as_vec()[0];
                let b = &items.as_vec()[1];
                is_54b_a(a)?;
                match_const_placeholder(b, 1, &SType::SSigmaProp)
            } else {
                None
            },
        _ => None,
    }
}

fn is_105b_script(e: &Expr) -> Option<()> {
    let e = e
        .match_bool_to_sigma()?
        .match_and()?
        .match_collection(&SType::SBoolean)?;
    let (a, b, c) = match_vec3(e)?;
    // A
    {
        let (a_1, a_2) = match_binop(a, BinOpKind::Relation(RelationOp::Eq))?;
        expect(a_1, &GlobalVars::Height.into())?;
        let a_2 = a_2.match_select_field(1)?.match_extract_creation_info()?;
        expect(
            a_2,
            &ByIndex {
                input: GlobalVars::Outputs.to_expr(),
                index: ConstantPlaceholder {
                    id: 0,
                    tpe: SType::SInt,
                }
                .to_expr(),
                default: None,
            }
            .into(),
        )?;
    }
    // B
    {
        let (b_1, b_2) = match_binop(b, RelationOp::Eq.into())?;
        expect(
            b_1,
            &ExtractScriptBytes {
                input: ByIndex {
                    input: GlobalVars::Outputs.to_expr(),
                    index: ConstantPlaceholder {
                        id: 1,
                        tpe: SType::SInt,
                    }
                    .to_expr(),
                    default: None,
                }
                .to_expr(),
            }
            .to_expr(),
        )?;
        expect(
            b_2,
            &SubstConstants {
                script_bytes: ConstantPlaceholder {
                    id: 2,
                    tpe: SType::SColl(SType::SByte.into()),
                }
                .to_expr(),
                positions: ConstantPlaceholder {
                    id: 3,
                    tpe: SType::SColl(SType::SInt.into()),
                }
                .to_expr(),
                new_values: Collection::Exprs {
                    elem_tpe: SType::SSigmaProp,
                    items: vec![CreateProveDlog {
                        input: DecodePoint {
                            input: GlobalVars::MinerPubKey.to_expr(),
                        }
                        .to_expr(),
                    }
                    .into()],
                }
                .to_expr(),
            }
            .to_expr(),
        )?;
    }
    // C
    {
        let (c_1, c_2) = match_binop(c, BinOpKind::Relation(RelationOp::Eq))?;
        expect(
            c_1,
            &SizeOf {
                input: Box::new(GlobalVars::Outputs.into()),
            }
            .into(),
        )?;
        match_const_placeholder(c_2, 4, &SType::SInt)?;
    }
    Some(())
}

fn is_198b_script(e: &Expr) -> Option<()> {
    let (vals, res) = match_block_value(e)?;
    dbg!(vals.len());
    // Parse result
    {
        let res = res.match_sigma_and()?;
        let (ra, rb) = match_vec2(res)?;
        {
            let ra = ra.match_sigma_or()?;
            let (ra1, ra2) = match_vec2(ra)?;
            let () = ra1.match_dlog()?.match_valuse(1, SType::SGroupElement)?;
            let (gv, hv, vv, uv) = ra2.match_dhtuple()?;
            let () = hv
                .match_option_get()?
                .match_extract_register(4, SType::SGroupElement)?
                .match_global_var(GlobalVars::SelfBox)?;
            let () = vv.match_valuse(1, SType::SGroupElement)?;
            let () = uv
                .match_option_get()?
                .match_extract_register(6, SType::SGroupElement)?
                .match_global_var(GlobalVars::SelfBox)?;
            // dbg!(gv);
        }
        {
            let rb = rb.match_bool_to_sigma()?;
            let (rb1,rb2) = match_binop(rb,RelationOp::Or.into())?;
            let (rb11,rb12) = match_binop(rb1, RelationOp::Or.into())?;
            let (rb111,rb112) = match_binop(rb11, RelationOp::And.into())?;
            dbg!(rb111);
            dbg!(rb112);
        }

    }
    //
    todo!()
}
// --------------------------------------------------------------------------------

#[derive(Debug)]
struct NTx {
    pub n36: u32,
    pub n54: u32,
    pub n105: u32,
    pub n198: u32,
}
impl NTx {
    fn new() -> NTx {
        NTx {
            n36: 0,
            n54: 0,
            n105: 0,
            n198: 0,
        }
    }

    fn tot(&self) -> usize {
        (self.n36 + self.n54 + self.n105 + self.n198) as usize
    }

    fn dbg(&self, tot: usize) {
        let tot = tot as f64;
        let print = |(s, n): (&str, u32)| {
            println!("{} = {} / {:.2}%", s, n, (n as f64) / tot * 100.0);
        };
        print(("B36 ", self.n36));
        print(("B54 ", self.n54));
        print(("B105", self.n105));
        print(("B198", self.n198));
        println!("Classified = {:.2}%", (self.tot() as f64) / tot * 100.0);
    }
}

fn parse_block(
    bytes: &[u8],
    hist: &mut BTreeMap<usize, usize>,
    ntx: &mut NTx,
) -> Result<String, SerializationError> {
    let mut buf = bytes.to_owned();
    let cursor = Cursor::new(&mut buf[..]);
    let mut r = SigmaByteReader::new(cursor, ConstantStore::empty());

    // Parse block
    let (n_tx, after_fork) = {
        let n = r.get_u32()?;
        if n == 10000002 {
            (r.get_u32()?, true)
        } else {
            (n, false)
        }
    };
    let txs = {
        let mut txs: Vec<Transaction> = Vec::with_capacity(n_tx as usize);
        for _ in 0..n_tx {
            let tx = Transaction::sigma_parse(&mut r)?;
            txs.push(tx);
        }
        txs
    };
    for tx in txs.iter().skip(1) {
        for out in &tx.output_candidates {
            let expr = &**(out.ergo_tree.tree.as_ref().unwrap().root.as_ref().unwrap());
            let n = out.ergo_tree.sigma_serialize_bytes().len();
            if is_36b_script(&expr).is_some() {
                ntx.n36 += 1;
            } else if is_54b_script(&expr).is_some() {
                ntx.n54 += 1;
            } else if is_105b_script(&expr).is_some() {
                ntx.n105 += 1;
            } else if n == 198 && is_198b_script(&expr).is_some() {
                ntx.n198 += 1;
            } else {
                // if n == 54 {
                //     dbg!(is_54b_script(&expr));
                //     println!("{:#?}", &expr);
                // }
                *(hist.entry(n).or_insert(0)) += 1;
            }
            // if n == 36 {
            //
            // }
        }
    }
    /*
       // Write block back
       let data = {
           let mut data = Vec::new();
           let mut w = SigmaByteWriter::new(&mut data, None);
           if after_fork {
               w.put_u32(10000002)?;
           }
           w.put_u32(n_tx)?;
           for tx in txs {
               tx.sigma_serialize(&mut w)?;
           }
           data
       };
       if data != buf {
           Err(Misc("Roundtrip failed".into()))?
       }
    */
    Ok("AZAZA".to_string())
}

fn run_block_scan(blk_iter: rusqlite::Rows) -> rusqlite::Result<()> {
    let blk_iter =
        blk_iter.mapped(|row| Ok((row.get::<usize, u32>(0)?, row.get::<usize, Vec<u8>>(1)?)));
    let mut hist = BTreeMap::new();
    let mut ntx = NTx::new();
    for row in blk_iter {
        let (h, blk) = row?;
        // println!("H={}", h);
        // parse_block(&blk).unwrap();
        match parse_block(&blk, &mut hist, &mut ntx) {
            Ok(_) => (),
            Err(e) => println!("H={} : {:?}", h, &e),
        };
    }
    // Sort in reverse popularity
    let mut freq = hist.iter().collect::<Vec<_>>();
    freq.sort_by_key(|(_, n)| std::cmp::Reverse(**n));
    // Compute total
    let tot: usize = freq.iter().map(|(_, n)| *n).sum::<usize>() + ntx.tot();
    println!("TOT = {}", tot);
    ntx.dbg(tot);
    for (sz, n) in freq.iter() {
        let f: f64 = (**n as f64) / (tot as f64);
        if f > 1e-2 {
            println!("{:4} {} ({:.2})", sz, n, f * 100.0);
        }
    }
    Ok(())
}

fn with_db<F, A>(fun: F) -> rusqlite::Result<A>
where
    F: Fn(rusqlite::Connection) -> rusqlite::Result<A>,
{
    let conn = rusqlite::Connection::open("../ergvein/blocks.sqlite")?;
    fun(conn)
}

fn query_plain<F, A>(conn: rusqlite::Connection, fun: F) -> rusqlite::Result<A>
where
    F: Fn(rusqlite::Rows) -> rusqlite::Result<A>,
{
    fun(conn
        .prepare(
            "SELECT height, txs FROM blocks \
                WHERE txs IS NOT NULL \
                ORDER BY height desc",
        )?
        .query([])?)
}

fn query_height<F, A>(conn: rusqlite::Connection, h: i32, fun: F) -> rusqlite::Result<A>
where
    F: Fn(rusqlite::Rows) -> rusqlite::Result<A>,
{
    fun(conn
        .prepare(
            "SELECT height, txs FROM blocks \
                  WHERE txs IS NOT NULL AND height = ? \
                  ORDER BY height desc",
        )?
        .query(rusqlite::params![h])?)
}

fn query_height_lt<F, A>(conn: rusqlite::Connection, h: i32, fun: F) -> rusqlite::Result<A>
where
    F: Fn(rusqlite::Rows) -> rusqlite::Result<A>,
{
    fun(conn
        .prepare(
            "SELECT height, txs FROM blocks \
                  WHERE txs IS NOT NULL AND height < ? \
                  ORDER BY height desc",
        )?
        .query(rusqlite::params![h])?)
}

fn query_height_gt<F, A>(conn: rusqlite::Connection, h: i32, fun: F) -> rusqlite::Result<A>
where
    F: Fn(rusqlite::Rows) -> rusqlite::Result<A>,
{
    fun(conn
        .prepare(
            "SELECT height, txs FROM blocks \
                  WHERE txs IS NOT NULL AND height > ? \
                  ORDER BY height desc",
        )?
        .query(rusqlite::params![h])?)
}

fn main() {
    let matches = App::new("Ergo parse")
        .arg(
            Arg::with_name("height")
                .short("h")
                .long("height")
                .value_name("H")
                .help("Read only block with given height")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("h-lt")
                .long("lt")
                .value_name("H")
                .help("Read only block with height smaller than given")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("h-gt")
                .long("gt")
                .value_name("H")
                .help("Read only block with height greater than given")
                .takes_value(true),
        )
        .get_matches();
    if let Some(h) = matches.value_of("height") {
        with_db(|conn| query_height(conn, h.parse().unwrap(), run_block_scan)).unwrap();
    } else if let Some(h) = matches.value_of("h-lt") {
        with_db(|conn| query_height_lt(conn, h.parse().unwrap(), run_block_scan)).unwrap();
    } else if let Some(h) = matches.value_of("h-gt") {
        with_db(|conn| query_height_gt(conn, h.parse().unwrap(), run_block_scan)).unwrap();
    } else {
        with_db(|conn| query_plain(conn, run_block_scan)).unwrap();
    }
}
