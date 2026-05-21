use spacetimedb::{DbContext, Local, Table, reducer, table, table::TableInternal};

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
    fn auto_inc<T: TableInternal>(&self) -> u32;
    fn auto_inc_migration<T>(&self, current_max: u32)
    where
        T: TableInternal;
}

impl<Db: DbContext<DbView = Local>> AutoIncWrite for Db {
    fn auto_inc<T: TableInternal>(&self) -> u32 {
        let table_id = match T::TABLE_NAME {
            "tab_raw_server_config_v2" => 1,
            _ => {
                log::error!("Table case not covered!");
                panic!()
            }
        };
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

    fn auto_inc_migration<T: TableInternal>(&self, current_max: u32) {
        let table_id = match T::TABLE_NAME {
            "tab_raw_server_config_v2" => 1,
            _ => {
                log::error!("Table case not covered!");
                panic!()
            }
        };
        if let Some(mut table) = self.db().auto_inc().table_id().find(table_id) {
            log::error!("Trying to do a auto inc migration but table already exists");
            panic!()
        }
        self.db()
            .auto_inc()
            .try_insert(AutoInc {
                table_id,
                current_id: current_max,
            })
            .unwrap();
    }
}
