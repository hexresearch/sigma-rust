use ergotree_ir::serialization::constant_store::ConstantStore;
use ergotree_ir::serialization::sigma_byte_reader::SigmaByteReader;
use ergotree_ir::serialization::SerializationError;
use ergotree_ir::serialization::SigmaSerializable;

use clap::{App, Arg};
use ergo_lib::chain::transaction::Transaction;
use rusqlite;
use sigma_ser::peekable_reader::PeekableReader;
use sigma_ser::vlq_encode::{ReadSigmaVlqExt,WriteSigmaVlqExt};
use std::io::Cursor;
use ergotree_ir::serialization::sigma_byte_writer::SigmaByteWriter;
use ergotree_ir::serialization::SerializationError::Misc;

fn parse_block(bytes: &[u8]) -> Result<String, SerializationError> {
    let mut buf = bytes.to_owned();
    let cursor = Cursor::new(&mut buf[..]);
    let pr = PeekableReader::new(cursor);
    let mut sr = SigmaByteReader::new(pr, ConstantStore::empty());
    let r = &mut sr;

    // Parse block
    let (n_tx, after_fork) = {
        let n = r.get_u32()?;
        if n == 10000002 {
            (r.get_u32()?, true)
        } else {
            (n,false)
        }
    };
    let txs = {
        let mut txs = Vec::with_capacity(n_tx as usize);
        for _ in 0..n_tx {
            txs.push(Transaction::sigma_parse(r)?);
        }
        txs
    };
    // Write block back
    let mut data = Vec::new();
    let mut w = SigmaByteWriter::new(&mut data, None);
    if after_fork {
        w.put_u32(10000002)?;
    }
    w.put_u32(n_tx)?;
    for tx in txs {
        tx.sigma_serialize(&mut w)?;
    }
    if data != buf {
        // println!("{:?}", buf);
        // println!("{:?}", data);
        Err(Misc("Roundtrip failed".into()))?
    }
    //self.sigma_serialize(&mut w)?;
    Ok("AZAZA".to_string())
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

fn run_block_scan(blk_iter: rusqlite::Rows) -> rusqlite::Result<()> {
    let blk_iter =
        blk_iter.mapped(|row| Ok((row.get::<usize, u32>(0)?, row.get::<usize, Vec<u8>>(1)?)));
    for row in blk_iter {
        let (h, blk) = row?;
        // println!("H={}", h);
        // parse_block(&blk).unwrap();
        match parse_block(&blk) {
            Ok(_) => (),
            Err(e) => println!("H={} : {:?}", h, &e),
        };
    }
    Ok(())
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
        .get_matches();
    match matches.value_of("height") {
        Some(h) => with_db(|conn| query_height(conn, h.parse().unwrap(), run_block_scan)).unwrap(),
        None =>
            match matches.value_of("h-lt") {
                Some(h) => with_db(|conn| query_height_lt(conn, h.parse().unwrap(), run_block_scan)).unwrap(),
                None => with_db(|conn| query_plain(conn, run_block_scan)).unwrap(),
            },
    }
}
