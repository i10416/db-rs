use nom::{
    IResult, Parser,
    bytes::complete::{tag_no_case, take_while1},
    character::complete::multispace1,
};

use crate::{Query, SelectStatement};

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
        .map(|(_, _, _, _, _, _, ident)| Query::Select(SelectStatement { from_table: ident }))
        .parse(s)
}

pub(crate) fn identifier<'a>(i: RawSpan<'a>) -> ParseResult<'a, String> {
    take_while1(|c: char| c.is_alphanumeric())
        .map(|a: &str| a.to_string())
        .parse(i)
}

#[cfg(test)]
mod tests {
    use crate::{Query, SelectStatement, parsing::nom::parse};

    #[test]
    fn parse_select() {
        let q = "SELECT * FROM t1";
        let (_, q) = parse(q).unwrap();
        assert_eq!(q, Query::Select(SelectStatement { from_table: "t1".into() }))
    }
}

pub type RawSpan<'a> = &'a str;

pub type ParseResult<'a, T> = IResult<RawSpan<'a>, T>;
