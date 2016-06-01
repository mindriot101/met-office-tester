extern crate sqlite;
extern crate chrono;

use self::chrono::*;

pub struct Database {
    connection: sqlite::Connection,
}

impl Database {
    pub fn new(path: &str) -> Database {
        Database { connection: sqlite::open(path).unwrap() }
    }

    pub fn insert(&mut self, dt: &DateTime<UTC>, temperature: &i64, upload_time: &DateTime<UTC>) {
        let mut statement = self.connection
            .prepare("insert into predictions (dt, temperature, upload_time)
            values (?, ?, ?)")
            .unwrap();
        statement.bind(1, dt.timestamp()).unwrap();
        statement.bind(2, *temperature).unwrap();
        statement.bind(3, upload_time.timestamp()).unwrap();
        loop {
            match statement.next() {
                Ok(sqlite::State::Done) => break,
                _ => (),
            }
        }
    }

    pub fn drop_tables(&mut self) {
        self.connection.execute("drop table if exists predictions").unwrap();
    }

    pub fn create_tables(&mut self) {
        self.connection
            .execute("create table if not exists predictions (
                id integer primary key,
                dt timestamp not null,
                temperature integer not null,
                upload_time timestamp not null
            )")
            .unwrap();
    }
}


