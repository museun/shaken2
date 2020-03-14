use super::database;
use rusqlite::Connection;
use std::borrow::Cow;

pub struct KeyValueStore<'a> {
    table: Cow<'a, str>,
    conn: Connection,
}

impl<'a> std::fmt::Debug for KeyValueStore<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyValuStore")
            .field("table", &self.table)
            .finish()
    }
}

// TODO better error handling
impl<'a> KeyValueStore<'a> {
    fn create_table(name: &str, conn: &Connection) -> anyhow::Result<()> {
        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {} (key BLOB UNIQUE, value BLOB)",
            name
        );

        conn.execute(&sql, rusqlite::NO_PARAMS)
            .map(|_| ())
            .map_err(Into::into)
    }

    pub fn fetch(table: &'a str) -> anyhow::Result<Self> {
        database::get_global_connection()
            .and_then(|conn| Self::create_table(table, &conn).map(|_| conn))
            .map(|conn| Self {
                conn,
                table: table.into(),
            })
    }

    #[allow(dead_code)]
    pub fn in_memory(table: impl Into<Cow<'a, str>>) -> anyhow::Result<Self> {
        let table = table.into();
        let conn = database::get_in_memory(&table)?;
        Self::create_table(&table, &conn)?;
        Ok(Self { conn, table })
    }

    pub fn get<K: ?Sized, V>(&self, key: &K) -> Option<V>
    where
        K: serde::Serialize + std::fmt::Debug,
        for<'de> V: serde::Deserialize<'de>,
    {
        let k = serde_cbor::to_vec(&key).expect("valid key repr");
        self.conn
            .query_row_named(
                &format!("SELECT value FROM {} WHERE key = :key", &self.table),
                rusqlite::named_params![":key": &k],
                |row| {
                    let data = &row.get_unwrap::<_, Vec<u8>>("value");
                    let val = serde_cbor::from_slice(data).expect("valid cbor");
                    Ok(val)
                },
            )
            .map_err(|err| {
                log::warn!("cannot get key: {:?} -> {}", key, err);
                err
            })
            .ok()
    }

    pub fn set<K: ?Sized, V: ?Sized>(&self, key: &K, val: &V) -> anyhow::Result<()>
    where
        K: serde::Serialize + std::fmt::Debug,
        V: serde::Serialize,
    {
        let k = serde_cbor::to_vec(&key).expect("valid key repr");
        let v = serde_cbor::to_vec(&val).expect("valid value repr");

        self.conn
            .execute_named(
                &format!(
                    "REPLACE INTO {} (key, value) values (:key, :value)",
                    &self.table
                ),
                rusqlite::named_params! {
                    ":key": &k,
                    ":value": &v,
                },
            )
            .map(|_| ())
            .map_err(|err| {
                log::warn!("cannot set key: {:?} -> ", err);
                anyhow::Error::from(err)
            })
    }

    #[allow(dead_code)]
    pub fn remove<K: ?Sized>(&self, key: &K) -> bool
    where
        K: serde::Serialize + std::fmt::Debug,
    {
        let k = serde_cbor::to_vec(&key).expect("valid key repr");
        match self.conn.execute_named(
            &format!("DELETE FROM {} WHERE key = :key", &self.table),
            rusqlite::named_params! {
                ":key": &k
            },
        ) {
            Err(..) => {
                log::debug!("error while removing key: {:?}", key);
                false
            }
            Ok(0) => {
                log::debug!("no row was removed for key: {:?}", key);
                false
            }
            Ok(..) => {
                log::trace!("removed key: {:?}", key);
                true
            }
        }
    }

    // iter
    // select key, value from _table_
    //
    // values
    // select value from _table_
    //
    // keys
    // select key from _table_
    //
    // len
    // select count(*) from _table_
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn key_value() {
        #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq)]
        struct Foo {
            name: String,
            size: usize,
        }

        let kv = KeyValueStore::fetch("testing").unwrap();
        kv.set(
            &42,
            &Foo {
                name: "this is a name".into(),
                size: 42,
            },
        )
        .unwrap();

        kv.set(&"asdf", &42).unwrap();

        assert_eq!(
            kv.get::<_, Foo>(&42).unwrap(),
            Foo {
                name: "this is a name".to_string(),
                size: 42,
            }
        );

        assert_eq!(kv.get::<_, i64>(&"asdf").unwrap(), 42);

        assert!(kv.remove(&"asdf"));
        assert!(kv.get::<_, i64>(&"asdf").is_none());
        assert!(!kv.remove(&"asdf"));

        kv.set(&"asdf", &42).unwrap();
        kv.set(&"asdf", &42).unwrap();
    }
}
