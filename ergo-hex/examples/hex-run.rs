#![feature(box_patterns)]
#![allow(non_camel_case_types)]
use ergotree_ir::mir::expr::Expr;
use ergotree_ir::serialization::constant_store::ConstantStore;
use ergotree_ir::serialization::sigma_byte_reader::SigmaByteReader;
use ergotree_ir::serialization::SerializationError;
use ergotree_ir::serialization::SigmaSerializable;
use clap::{App, Arg};
use ergo_lib::chain::transaction::Transaction;
use ergotree_ir::mir::bin_op::{ArithOp, BinOpKind, RelationOp};
use ergotree_ir::mir::extract_creation_info::ExtractCreationInfo;
use ergotree_ir::mir::global_vars::GlobalVars;
use ergotree_ir::mir::select_field::{SelectField};
use ergotree_ir::mir::sigma_and::SigmaAnd;
use ergotree_ir::serialization::sigma_byte_writer::SigmaByteWriter;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use ergotree_ir::sigma_protocol::sigma_boolean::{ProveDlog, SigmaProofOfKnowledgeTree};
use ergotree_ir::types::stype::SType;
use rusqlite;
use sigma_ser::vlq_encode::ReadSigmaVlqExt;
use sigma_ser::vlq_encode::WriteSigmaVlqExt;
use std::collections::BTreeMap;
use std::collections::HashMap;
// use std::io;
use ergo_lib::chain::{blake2b256_hash, Digest32};
use std::convert::{TryFrom, TryInto};
use std::io::Cursor;

use ergo_hex::hex::errors::SErr;
use ergo_hex::hex::matcher::*;
use ergo_hex::hex::sql::Query;

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

// --------------------------------------------------------------------------------
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum TxType {
    TxP2PK_36,
    TxP2PK_54,
    Tx105,
    Tx198,
    Tx228,
    Tx251,
    Tx261,
    Tx415,
    Tx450,
}

struct Classifier {
    pub named: HashMap<TxType, u32>,
    pub sized: BTreeMap<usize, u32>,
    hash_b105: Digest32,
    hash_b198: Digest32,
    hash_b228: Digest32,
    hash_b251: Digest32,
    hash_b261: Digest32,
    hash_b415: Digest32,
    hash_b450: Digest32,
}

impl Classifier {
    fn new() -> Classifier {
        fn mk(s: &str) -> Digest32 {
            Digest32::try_from(s.to_string()).unwrap()
        }
        let hash_b105 = mk("922e50aa537eacdad7f0584dd703426d8f376d7623422afc3b212bd2f674e2b2");
        let hash_b198 = mk("441438d8b1a847e8cc6f8c19e43e6c63e516cf1e1db45f246c1b6e84f70e685f");
        let hash_b228 = mk("ac08c57ae9c761ffd023cf64d666228ca15f2e46f9b74171a445efa50b65d58c");
        let hash_b251 = mk("c5e5cd8afc994181ea5f4b32e2aad6b2af2acbf797fdcb5e47a7bacb9d9c9386");
        let hash_b261 = mk("d7e0ee6e86a01384211ad7d0f726364a1951a9b48be6663a6cab25f89287cb87");
        let hash_b450 = mk("25b9da55e8c2ef009c0196aa2caa1b9bba2755a9f2627b4d64c2928aad6e63af");
        let hash_b415 = mk("f7dfa8929aebc1a8f4b26ab5642f08daf28c93c7c73d4ea45da045b4e181ae73");
        Classifier {
            named: HashMap::<TxType, u32>::new(),
            sized: BTreeMap::new(),
            hash_b105,
            hash_b198,
            hash_b228,
            hash_b251,
            hash_b261,
            hash_b415,
            hash_b450,
        }
    }

    fn bump_named(&mut self, k: TxType) {
        (*self.named.entry(k).or_insert(0)) += 1;
    }
}
impl Check for Classifier {
    fn name(&self) -> &'static str {
        "Classifier"
    }

    fn check(&mut self, txdata: &TxData) {
        for tx in txdata.block.iter().flatten() {
            for out in &tx.output_candidates {
                // FIXME: Extremely ugly and dangerous
                let expr = &**(out.ergo_tree.tree.as_ref().unwrap().root.as_ref().unwrap());
                // Here we compute size of full ergo tree but test for equality only script itself
                let h = blake2b256_hash(&expr.sigma_serialize_bytes());
                let n = out.ergo_tree.sigma_serialize_bytes().len();


                if is_36b_script(&expr).is_some() {
                    self.bump_named(TxType::TxP2PK_36);
                } else if is_54b_script(&expr).is_some() {
                    self.bump_named(TxType::TxP2PK_54);
                } else if n == 105 && h == self.hash_b105 {
                    self.bump_named(TxType::Tx105);
                } else if n == 198 && h == self.hash_b198 {
                    self.bump_named(TxType::Tx198);
                } else if n == 228 && h == self.hash_b228 {
                    self.bump_named(TxType::Tx228);
                } else if n == 251 && h == self.hash_b251 {
                    self.bump_named(TxType::Tx251);
                } else if n == 261 && h == self.hash_b261 {
                    self.bump_named(TxType::Tx261);
                } else if n == 415 && h == self.hash_b415 {
                    self.bump_named(TxType::Tx415);
                } else if n == 450 && h == self.hash_b450 {
                    self.bump_named(TxType::Tx450);
                } else {
                    *(self.sized.entry(n).or_insert(0)) += 1;
                }
            }
        }
    }

    fn finalize(&self) {
        let tot_named = self.named.iter().map(|(_, n)| *n).sum::<u32>();
        let tot_sized =self.sized.iter().map(|(_, n)| *n).sum::<u32>();
        let tot: u32 = tot_named + tot_sized;
        println!("TOT        = {}", tot);
        println!("Classified = {:.2}%", ((tot_named as f64) / (tot as f64))*100.0);
        // Named
        {
            let mut named = self.named.iter().collect::<Vec<_>>();
            named.sort_by_key(|(k, _)| *k);
            for (nm, n) in named {
                let f: f64 = (*n as f64) / (tot as f64);
                println!("{:?} {:6} ({:.2}%)", nm, n, f * 100.0);
            }
        }
        // Sized
        {
            let mut sized = self.sized.iter().collect::<Vec<_>>();
            sized.sort_by_key(|(_, n)| std::cmp::Reverse(**n));
            for (sz, n) in sized.iter() {
                let f: f64 = (**n as f64) / (tot as f64);
                if f > 0.5e-2 {
                    println!("{:4} {:6} ({:.2}%)", sz, n, f * 100.0);
                }
            }
        }
    }
}


// --------------------------------------------------------------------------------

trait Check {
    fn name(&self) -> &'static str;
    fn check(&mut self, tx: &TxData);
    fn finalize(&self) {}
}

// --------------------------------------------------------------------------------

/// Transactions correctly roundtrips
struct Roundtrip;

impl Check for Roundtrip {
    fn name(&self) -> &'static str {
        "Roundtrip"
    }
    fn check(&mut self, tx: &TxData) {
        if let Ok(txs) = &tx.block {
            let mut data = Vec::new();
            let mut w = SigmaByteWriter::new(&mut data, None);
            if tx.after_fork {
                w.put_u32(10000002).unwrap();
            }
            w.put_u32(txs.len() as u32).unwrap();
            for tx in txs {
                tx.sigma_serialize(&mut w).unwrap();
            }
            if data != tx.binary {
                println!("H={} Roundtrip failed", tx.h);
            }
        }
    }
}

// --------------------------------------------------------------------------------

/// Transactions data
struct TxData {
    pub h: u32,
    pub binary: Vec<u8>,
    pub block: Result<Vec<Transaction>, SerializationError>,
    pub after_fork: bool,
}

impl TxData {
    /// Create new TX data object
    pub fn new(h: u32, binary: Vec<u8>) -> TxData {
        let res = Self::parse_block(&binary);
        let after_fork = match &res {
            Ok((a, _)) => *a,
            Err(_) => false,
        };
        let block = res.map(|(_, b)| b);
        TxData {
            h,
            binary,
            block,
            after_fork,
        }
    }

    /// Test that we parsed thing correctly
    pub fn check_parse(&self) -> () {
        if let Err(e) = &self.block {
            println!("H={} : {:?}", self.h, e);
        }
    }

    fn parse_block(binary: &Vec<u8>) -> Result<(bool, Vec<Transaction>), SerializationError> {
        let cursor = Cursor::new(&binary);
        let mut r = SigmaByteReader::new(cursor, ConstantStore::empty());
        // Parse number of transactions
        let (n_tx, after_fork) = {
            let n = r.get_u32()?;
            if n == 10000002 {
                (r.get_u32()?, true)
            } else {
                (n, false)
            }
        };
        //
        let mut txs: Vec<Transaction> = Vec::with_capacity(n_tx as usize);
        for _ in 0..n_tx {
            let tx = Transaction::sigma_parse(&mut r)?;
            txs.push(tx);
        }
        Ok((after_fork, txs))
    }
}

fn query_plain(conn: &rusqlite::Connection) -> rusqlite::Result<Query> {
    Query::new_(
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
        .arg(
            Arg::with_name("roundtrip")
                .long("roundtrip")
                .help("Check that TX encoding roudntrips"),
        )
        .arg(
            Arg::with_name("classify")
                .long("classify")
                .help("Check that TX encoding roudntrips"),
        )
        .get_matches();
    // Feature flags
    let mut checks: Vec<Box<dyn Check>> = Vec::new();
    if matches.is_present("roundtrip") {
        checks.push(Box::new(Roundtrip));
    }
    if matches.is_present("classify") {
        checks.push(Box::new(Classifier::new()));
    }
    // How to query database
    let mk_query: Box<dyn Fn(&rusqlite::Connection) -> rusqlite::Result<Query>> =
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
    // Announces
    for c in &checks {
        println!("- Checking {}", c.name());
    }
    // Run program
    let conn = rusqlite::Connection::open("../ergvein/blocks.sqlite")?;
    let mut query = mk_query(&conn)?;
    let rows = query
        .run()?
        .mapped(|row| Ok((row.get::<usize, u32>(0)?, row.get::<usize, Vec<u8>>(1)?)));
    for row in rows {
        let (h, bs) = row?;
        let tx_data = TxData::new(h, bs);
        // Start checking stuff
        tx_data.check_parse();
        for c in &mut checks {
            c.check(&tx_data);
        }
    }
    for c in &checks {
        c.finalize();
    }
    Ok(())
}
