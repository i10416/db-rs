## Query and Execution Plan
In general, a database client interacts with a database by a query language
such as SQL. The database is responsible for transforming a query tree into
an execution plan tree to retrieve/mutate data from underlying storage
and return a result set to the client.

It is a kind of "compilation" from a query tree to an execution plan tree.
The conversion does not always form 1:1 relationship and the database engine
is responsible for determining the "best" execution tree from possible N execution tree.
