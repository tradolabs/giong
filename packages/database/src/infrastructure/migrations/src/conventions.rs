use sha1::{Digest, Sha1};

pub fn foreign_key_name(
    table: &str,
    columns: &[&str],
    referenced_table: &str,
    referenced_columns: &[&str],
) -> String {
    foreign_key_name_with_hash(table, columns, referenced_table, referenced_columns, true)
}

pub fn foreign_key_name_with_hash(
    table: &str,
    columns: &[&str],
    referenced_table: &str,
    referenced_columns: &[&str],
    hash: bool,
) -> String {
    let source = generate_identifiers(table, columns);
    let reference = generate_identifiers(referenced_table, referenced_columns);
    let readable = format!("FK_{source}_{reference}");
    constraint_name(readable, &source, hash)
}

pub fn unique_constraint_name(table: &str, columns: &[&str]) -> String {
    unique_constraint_name_with_hash(table, columns, true)
}

pub fn unique_constraint_name_with_hash(table: &str, columns: &[&str], hash: bool) -> String {
    let source = generate_identifiers(table, columns);
    constraint_name(format!("UQ_{source}"), &source, hash)
}

fn constraint_name(readable: String, hash_key: &str, hash: bool) -> String {
    if !hash {
        return readable;
    }
    const HASH_HEX_LENGTH: usize = 27;
    let hash: String =
        Sha1::digest(hash_key.as_bytes()).iter().map(|byte| format!("{byte:02x}")).collect();
    format!("{readable}_{}", &hash[..HASH_HEX_LENGTH])
}

fn generate_identifiers(table: &str, columns: &[&str]) -> String {
    let table = table.rsplit('.').next().unwrap_or(table);
    let mut columns = columns.to_vec();
    columns.sort_unstable();
    format!("{table}_{}", columns.join("_"))
}
