use anyhow::Context;
use lact_schema::{Request, Response};
use schemars::schema_for;
use std::fs::File;
use std::io::BufWriter;

fn main() -> anyhow::Result<()> {
    {
        let schema = schema_for!(Response);
        let file = BufWriter::new(
            File::create("schema/response.json")
                .context("Could not create schema/response.json")?,
        );
        serde_json::to_writer_pretty(file, &schema).context("Could not write schema")?;
    }

    {
        let schema = schema_for!(Request);
        let file = BufWriter::new(
            File::create("schema/request.json").context("Could not create schema/request.json")?,
        );
        serde_json::to_writer_pretty(file, &schema).context("Could not write schema")?;
    }

    Ok(())
}
