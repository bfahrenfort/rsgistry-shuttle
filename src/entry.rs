use serde::{Deserialize, Serialize};
use sqlx::{database::HasArguments, query::QueryAs, FromRow, Postgres};

// This is exactly how your Entry looks in the database migrations, less the ID
// - Use Option<SomeType> for fields that can be NULL
// - Others will be NOT NULL
// You can probably use this type to send your requests from your API consumers as well!
// - ...obviously minus the mixin junk
#[derive(Serialize, Deserialize, Debug)]
#[mixin::declare]
pub struct Entry {
    program_name: String,
    doctype: String,
    url: Option<String>,
}

// Internal database type, basically all of the above Entry plus the automatically-generated fields
// You may need to adjust this depending on your schema, but it's not likely
#[mixin::insert(Entry)]
#[derive(Serialize, FromRow)]
pub struct EntryWithID {
    pub id: i32,
}

//
impl Entry {
    pub fn bind<'q>(
        self: &'q Entry,
        query: QueryAs<'q, Postgres, EntryWithID, <Postgres as HasArguments>::Arguments>,
    ) -> QueryAs<'q, Postgres, EntryWithID, <Postgres as HasArguments>::Arguments> {
        query
            .bind(&self.program_name)
            .bind(&self.doctype)
            .bind(&self.url)
    }
}
