use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args = std::env::args().skip(1).collect::<Vec<_>>();
  if args.len() != 2 {
    eprintln!(
      "Usage: export_sql <path-to-itemcache.wdb> <output-item_template.sql>\n\
       Example: export_sql \"E:\\\\World of Warcraft 2.4.3\\\\Cache\\\\WDB\\\\enUS\\\\itemcache.wdb\" \"item_template.sql\""
    );
    std::process::exit(2);
  }

  let itemcache = PathBuf::from(&args[0]);
  let output = PathBuf::from(&args[1]);

  let count = tbc_wdb_parser_lib::itemcache_export::export_itemcache_to_cmangos_item_template_sql(
    &itemcache,
    &output,
    false,
  )?;
  println!("Exported {} items to {}", count, output.display());
  Ok(())
}

