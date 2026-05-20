#![allow(clippy::use_self)]
use spacetimedb::{DbContext, Local, Table, table, table::TableInternal};

#[table(accessor=auto_inc)]
struct AutoInc {
    #[primary_key]
    table_id: u32,
    current_id: u32,
}

pub trait TableAutoIncable {
    const TABLE_ID: u32;
}

pub(crate) trait AutoIncWrite {
    fn auto_inc<T: TableInternal>(&self, _: T) -> u32;
}

impl<Db: DbContext<DbView = Local>> AutoIncWrite for Db {
    fn auto_inc<T: TableInternal>(&self, _: T) -> u32 {
        let table_id = T::table_id().0;
        if let Some(mut table) = self.db().auto_inc().table_id().find(table_id) {
            table.current_id += 1;
            let current_id = table.current_id;
            self.db().auto_inc().table_id().update(table);
            current_id
        } else {
            self.db().auto_inc().insert(AutoInc {
                table_id,
                current_id: 1,
            });
            1
        }
    }
}
