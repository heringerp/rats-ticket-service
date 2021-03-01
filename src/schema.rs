table! {
    accounts (id) {
        id -> Nullable<Integer>,
        email -> Text,
        firstname -> Nullable<Text>,
        lastname -> Text,
        password -> Text,
    }
}

table! {
    tickets (id) {
        id -> Nullable<Integer>,
        description -> Text,
        approved -> Bool,
        approver -> Integer,
        requestor -> Integer,
        filename -> Text,
    }
}

allow_tables_to_appear_in_same_query!(accounts, tickets,);
