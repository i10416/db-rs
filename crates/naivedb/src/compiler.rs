use naivedb_api::{Query, SelectStatement};
use naivedb_kernel::disk::PageId;

use crate::query::{PlanNode, SeqScan, TupleSearchMode};

pub fn compiler(q: Query) -> Box<dyn PlanNode> {
    match q {
        // select everything from the table
        Query::Select(SelectStatement { table }) => Box::new(SeqScan {
            table_meta_page_id: look_up_page_id_by_table_name(&table),
            search_mode: TupleSearchMode::Start,
            while_cond: &|_| true,
        }),
    }
}

fn look_up_page_id_by_table_name(_: &str) -> PageId {
    todo!()
}
