use nom::{
    IResult, Parser,
    bytes::complete::{tag_no_case, take_while1},
    character::complete::multispace1,
};

#[derive(Debug, PartialEq)]
pub enum Query {
    Select(SelectStatement),
}

type RawSpan<'a> = &'a str;

type ParseResult<'a, T> = IResult<RawSpan<'a>, T>;

#[derive(Debug, PartialEq)]
pub struct SelectStatement {
    // always select all for now
    // cols: ...,
    table: String, // only one table is allowed for from clause
}
pub fn parse<'a>(s: RawSpan<'a>) -> ParseResult<'a, Query> {
    (
        tag_no_case("select"),
        multispace1,
        nom::character::char('*'),
        multispace1,
        tag_no_case("from"),
        multispace1,
        identifier,
    )
        .map(|(_, _, _, _, _, _, ident)| Query::Select(SelectStatement { table: ident }))
        .parse(s)
}

pub(crate) fn identifier<'a>(i: RawSpan<'a>) -> ParseResult<'a, String> {
    take_while1(|c: char| c.is_alphanumeric())
        .map(|a: &str| a.to_string())
        .parse(i)
}

#[cfg(test)]
mod tests {
    use crate::{Query, SelectStatement, parse};

    #[test]
    fn parse_select() {
        let q = "SELECT * FROM t1";
        let (_, q) = parse(q).unwrap();
        assert_eq!(q, Query::Select(SelectStatement { table: "t1".into() }))
    }
}
