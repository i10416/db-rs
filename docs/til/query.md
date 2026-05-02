## Query and Execution Plan
In general, a database client interacts with a database by a query language
such as SQL. The database is responsible for transforming a query tree into
an execution plan tree to retrieve/mutate data from underlying storage
and return a result set to the client.

It is a kind of "compilation" from a query tree to an execution plan tree.
The conversion does not always form 1:1 relationship and the database engine
is responsible for determining the "best" execution tree from possible N execution tree.


Pseudocode

```rs
// the naivest query tree representing
// `SELECT * FROM _ WHERE pk = .. AND col = value`
enum Query {
  SelectAll {
    cond: WherePkEq,
    filter: Filter
  }
}

struct WherePkEq {
  value: [[u8]]
}

struct Filter {
  colidx: usize,
  value: [[u8]]
}

```

```rs
let q = Query::SelectAll {
  cond: WherePkEq {
    value: [[...]],
  }
}

let e = compile(q);
// =>
// SeqScan {
//   table_meta_page_id: ...,
//   search_mode: SearchMode::Key(encode([[...]])),
//   while_cond: (keys) => keys[0] == [...]
// }

```