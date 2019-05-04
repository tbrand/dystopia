table! {
    audits (addr, ts) {
        addr -> Varchar,
        state -> Varchar,
        version -> Varchar,
        ts -> Int8,
    }
}

table! {
    nodes (addr) {
        addr -> Varchar,
        state -> Varchar,
        version -> Varchar,
    }
}

allow_tables_to_appear_in_same_query!(
    audits,
    nodes,
);
