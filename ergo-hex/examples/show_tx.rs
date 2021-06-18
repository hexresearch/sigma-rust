use clap::{App, Arg};
use ergo_hex::hex::errors::SErr;
use ergo_hex::hex::prettyprinter::ppr;
use ergotree_ir::mir::expr::Expr;
use ergotree_ir::serialization::SigmaSerializable;

fn main() -> Result<(), SErr> {
    let matches = App::new("Ergo parse")
        .arg(
            Arg::with_name("is36b")
                .long("is36b")
                .help("Show some 36B transaction"),
        )
        .arg(
            Arg::with_name("hash")
                .long("hash")
                .value_name("HASH")
                .help("Show script with given hash")
                .takes_value(true),
        )
        .get_matches();
    //
    let db = rusqlite::Connection::open("../ergvein/blocks.sqlite")?;
    if matches.is_present("is36b") {
        let mut stmt = db.prepare("SELECT script FROM outputs_list WHERE is_36b LIMIT 1")?;
        let rows = stmt.query_map([], |row| row.get::<usize, Vec<u8>>(0))?;
        for script in rows {
            let script = script?;
            let script = Expr::sigma_parse_bytes(&script)?;
            println!("{}", ppr(&script, 80));
        }
    } else if let Some(h) = matches.value_of("hash") {
        let mut stmt = db.prepare("SELECT script FROM outputs_list WHERE hex(script_hash) = ? LIMIT 1")?;
        let rows = stmt.query_map([h], |row| row.get::<usize, Vec<u8>>(0))?;
        for script in rows {
            let script = script?;
            let script = Expr::sigma_parse_bytes(&script)?;
            println!("{}", ppr(&script, 80));
        }
    }
    Ok(())
}
