use ergo_hex::hex::errors::SErr;
use ergo_hex::hex::matcher::{match_const, val_sigma_prop};
use ergo_hex::hex::parse;
use ergo_hex::hex::sql;
use ergo_lib::chain::blake2b256_hash;
use ergotree_ir::mir::expr::Expr;
use ergotree_ir::serialization::SigmaSerializable;
use ergotree_ir::sigma_protocol::sigma_boolean::{SigmaBoolean, SigmaProofOfKnowledgeTree};
use ergotree_ir::types::stype::SType;

fn is_36b_script(e: &Expr) -> Option<()> {
    let v = match_const(e, &SType::SSigmaProp)?;
    match val_sigma_prop(v)? {
        SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(_)) => Some(()),
        _ => None,
    }
}


fn main() -> Result<(), SErr> {
    let mut db = sql::open("../ergvein/blocks.sqlite")?;
    let dbtx = db.transaction()?;
    {
        let mut stmt_store =
            dbtx.prepare("INSERT OR IGNORE INTO transaction_list VALUES (NULL, ?,?,?,?)")?;
        let mut stmt_store_out =
            dbtx.prepare("INSERT OR IGNORE INTO outputs_list VALUES (NULL, ?,?,?,?,?,?,?)")?;
        let mut stmt = dbtx.prepare("SELECT height, txs FROM blocks")?;
        let rows = stmt
            .query([])?
            .mapped(|row| Ok((row.get::<usize, u32>(0)?, row.get::<usize, Vec<u8>>(1)?)));
        for row in rows {
            let (h, bytes) = row?;
            if h % 1000 == 0 {
                println!("H={}", h)
            }
            //
            let (_, txs) = parse::parse_block(&bytes)?;
            for (tx, tx_n) in txs.iter().zip(0..) {
                let tx_bytes = tx.sigma_serialize_bytes();
                let tid = *blake2b256_hash(&tx_bytes).0;
                stmt_store.execute(rusqlite::params![h, tx_n, <Vec<u8>>::from(tid), tx_bytes])?;
                let tx_id = (*dbtx).last_insert_rowid();
                for (out, n_out) in tx.output_candidates.iter().zip(0..) {
                    // FIXME: Extremely ugly and dangerous
                    let parsed_tree = out.ergo_tree.tree.as_ref().unwrap();
                    let consts = &parsed_tree.constants;
                    let expr = &**parsed_tree.root.as_ref().unwrap();
                    let expr_bytes = expr.sigma_serialize_bytes();
                    let eid = *blake2b256_hash(&expr_bytes).0;
                    stmt_store_out.execute(rusqlite::params![
                        tx_id,
                        n_out,
                        consts.sigma_serialize_bytes(),
                        consts.len(),
                        expr_bytes,
                        <Vec<u8>>::from(eid),
                        is_36b_script(expr).is_some()
                    ])?;
                }
            }
        }
    }
    dbtx.commit()?;
    Ok(())
}
