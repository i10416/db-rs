pub mod parsing;

#[derive(Debug, PartialEq)]
pub enum Query {
    Select(SelectStatement),
}

#[derive(Debug, PartialEq)]
pub struct SelectStatement {
    // always select all columns in this implementation
    // cols: ...
    // ---
    // only one table is allowed for from clause in this implementation
    pub from_table: String,
    // where_clause: Option<_>
}
