# Naming Conventions

This document outlines the guidelines that govern how builder methods are named, etc.

## Builder methods

### Clauses

e.g. A method(s) for adding a GROUP BY clause to a SELECT statement.

- There are often multiple methods that achieve the same result, each targeting a
  different trade-off between ergonomics & flexibility.

e.g.
- `group_by(predicates: ...)`
  - Easiest to use. Accepts iterator of predicates.
  - Methods as this level accept straightforward arguments, specific to what type of clause, etc., is being 
    incorporated into the builder.
- `group_by_clause(build_fn: ...)`
  - Accepts a builder fn, which is given a default, un-finalized, GroupBy clause.
  - May enforce some defaults, but otherwise lets caller configure the rest of the clause.
  - Denoted by `_clause` suffix.
- `with_group_by_clause(clause: ...)`
  - Most flexible. Accepts a finalized GroupBy clause.
  - Denoted by `with_` prefix and `_clause` suffix.