use std::fs;

use mdcheck::model::DocModel;

fn main() -> anyhow::Result<()> {
    let path = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("usage: mdcheck-dump <file.md>"))?;
    let source = fs::read_to_string(&path)?;
    let doc = DocModel::parse(&source);

    for error in &doc.load_errors {
        eprintln!("load: {error}");
    }
    print!("{}", doc.dump_tree());
    Ok(())
}
