pub mod parsing;

#[derive(Debug, PartialEq)]
pub enum Query {
    Select(SelectStatement),
}

#[derive(Debug, PartialEq)]
pub struct SelectStatement {
    // always select all for now
    // cols: ...,
    pub table: String, // only one table is allowed for from clause
}
