use ergo_lib::chain::transaction::Transaction;
use ergotree_ir::serialization::constant_store::ConstantStore;
use ergotree_ir::serialization::sigma_byte_reader::SigmaByteReader;
use ergotree_ir::serialization::SerializationError;
use ergotree_ir::serialization::SigmaSerializable;
use sigma_ser::vlq_encode::ReadSigmaVlqExt;
use std::io::Cursor;

pub fn parse_block(binary: &Vec<u8>) -> Result<(bool, Vec<Transaction>), SerializationError> {
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
