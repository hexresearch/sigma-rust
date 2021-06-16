#![feature(box_patterns)]

mod prettyprinter;
mod testset;

use ergotree_ir::mir::expr::{Expr, ToBoxedExprExt};
use ergotree_ir::serialization::constant_store::ConstantStore;
use ergotree_ir::serialization::sigma_byte_reader::SigmaByteReader;
use ergotree_ir::serialization::SerializationError;
use ergotree_ir::serialization::SigmaSerializable;
use pretty;
use prettyprinter::Pretty;

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
use ergo_lib::chain::{blake2b256_hash, Digest32};
use ergotree_ir::mir::coll_by_index::ByIndex;
use ergotree_ir::mir::create_prove_dh_tuple::CreateProveDhTuple;
use ergotree_ir::mir::create_provedlog::CreateProveDlog;
use ergotree_ir::mir::decode_point::DecodePoint;
use ergotree_ir::mir::extract_script_bytes::ExtractScriptBytes;
use ergotree_ir::mir::subst_const::SubstConstants;
use std::convert::{TryFrom, TryInto};
use std::io::Cursor;

use rusqlite::ToSql;
use testset::errors::SErr;
use testset::matcher::*;

// ---------------------------------------------------------------

// impl Deconstruct for Expr {
//     fn match_and(self) -> Option<Self> {
//         match self {
//             Expr::And(And { box input }) => Some(input),
//             _ => None,
//         }
//     }
// }

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
        Expr::SigmaAnd(SigmaAnd { items }) => {
            if items.len() == 2 {
                let a = &items.as_vec()[0];
                let b = &items.as_vec()[1];
                is_54b_a(a)?;
                match_const_placeholder(b, 1, &SType::SSigmaProp)
            } else {
                None
            }
        }
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
            let (rb1, rb2) = match_binop(rb, RelationOp::Or.into())?;
            let (rb11, rb12) = match_binop(rb1, RelationOp::Or.into())?;
            let (rb111, rb112) = match_binop(rb11, RelationOp::And.into())?;
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
    pub n415: u32,
    pub n450: u32,
}
impl NTx {
    fn new() -> NTx {
        NTx {
            n36: 0,
            n54: 0,
            n105: 0,
            n198: 0,
            n415: 0,
            n450: 0,
        }
    }

    fn tot(&self) -> usize {
        (self.n36 + self.n54 + self.n105 + self.n198 + self.n415 + self.n450) as usize
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
        print(("B415", self.n415));
        print(("B450", self.n450));
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

    let hash_b198 = Digest32::try_from(
        "441438d8b1a847e8cc6f8c19e43e6c63e516cf1e1db45f246c1b6e84f70e685f".to_string(),
    )
    .unwrap();
    let hash_b450 = Digest32::try_from(
        "25b9da55e8c2ef009c0196aa2caa1b9bba2755a9f2627b4d64c2928aad6e63af".to_string(),
    )
    .unwrap();
    let hash_b415 = Digest32::try_from(
        "f7dfa8929aebc1a8f4b26ab5642f08daf28c93c7c73d4ea45da045b4e181ae73".to_string(),
    )
    .unwrap();

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
            } else if n == 198 {
                let h = blake2b256_hash(&expr.sigma_serialize_bytes());
                if h == hash_b198 {
                    ntx.n198 += 1;
                }
            } else if n == 415 {
                let h = blake2b256_hash(&expr.sigma_serialize_bytes());
                if h == hash_b415 {
                    ntx.n415 += 1;
                }
            } else if n == 450 {
                // let mut w = Vec::new();
                // expr.pretty().render(80, &mut w).unwrap();
                // println!("{}", String::from_utf8(w).unwrap());
                //
                let h = blake2b256_hash(&expr.sigma_serialize_bytes());
                if h == hash_b450 {
                    ntx.n450 += 1;
                }
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

struct Query<'a> {
    stmt: rusqlite::Statement<'a>,
    param: Vec<Box<dyn rusqlite::ToSql>>,
}

impl<'a> Query<'a> {
    pub fn newP(conn: &'a rusqlite::Connection, sql: &str) -> rusqlite::Result<Query<'a>> {
        let stmt = conn.prepare(&sql)?;
        Ok(Query {
            stmt,
            param: Vec::new(),
        })
    }

    pub fn new(
        conn: &'a rusqlite::Connection,
        sql: &str,
        param: Vec<Box<dyn rusqlite::ToSql>>,
    ) -> rusqlite::Result<Query<'a>> {
        let stmt = conn.prepare(&sql)?;
        Ok(Query { stmt, param })
    }

    pub fn run(&mut self) -> rusqlite::Result<rusqlite::Rows> {
        let params = self.param.iter().map(|x| &**x).collect::<Vec<_>>();
        let params: &[&dyn ToSql] = params.as_ref();
        self.stmt.query(params)
    }
}

fn query_plain(conn: &rusqlite::Connection) -> rusqlite::Result<Query> {
    Query::newP(
        conn,
        "SELECT height, txs FROM blocks \
                WHERE txs IS NOT NULL \
                ORDER BY height desc",
    )
}

fn query_height(conn: &rusqlite::Connection, h: i32) -> rusqlite::Result<Query> {
    Query::new(
        conn,
        "SELECT height, txs FROM blocks \
                  WHERE txs IS NOT NULL AND height = ? \
                  ORDER BY height desc",
        vec![Box::new(h)],
    )
}

fn query_height_lt(conn: &rusqlite::Connection, h: i32) -> rusqlite::Result<Query> {
    Query::new(
        conn,
        "SELECT height, txs FROM blocks \
                  WHERE txs IS NOT NULL AND height < ? \
                  ORDER BY height desc",
        vec![Box::new(h)],
    )
}

fn query_height_gt(conn: &rusqlite::Connection, h: i32) -> rusqlite::Result<Query> {
    Query::new(
        conn,
        "SELECT height, txs FROM blocks \
                  WHERE txs IS NOT NULL AND height > ? \
                  ORDER BY height desc",
        vec![Box::new(h)],
    )
}

fn main() -> Result<(), SErr> {
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
    let mk_query: Box<Fn(&rusqlite::Connection) -> rusqlite::Result<Query>> =
        if let Some(h) = matches.value_of("height") {
            let h = h.parse().unwrap();
            Box::new(move |conn| query_height(conn, h))
        } else if let Some(h) = matches.value_of("h-lt") {
            let h = h.parse().unwrap();
            Box::new(move |conn| query_height_lt(conn, h))
        } else if let Some(h) = matches.value_of("h-gt") {
            let h = h.parse().unwrap();
            Box::new(move |conn| query_height_gt(conn, h))
        } else {
            Box::new(|conn| query_plain(conn))
        };
    // Run program
    let conn = rusqlite::Connection::open("../ergvein/blocks.sqlite")?;
    let mut query = mk_query(&conn)?;
    run_block_scan(query.run()?)?;
    Ok(())
}
