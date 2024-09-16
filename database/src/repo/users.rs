use chrono::{NaiveDateTime, Utc};
use diesel::{
    query_dsl::methods::{FilterDsl, SelectDsl},
    result::Error,
    ExpressionMethods, RunQueryDsl, SelectableHelper,
};
use models::User;

use crate::{
    establish_connection,
    models::{self, NewUser},
    schema::{self, users},
};

pub fn get_all_users() -> Vec<User> {
    use schema::users::dsl::*;

    let connection = &mut establish_connection();
    let results = users
        .load::<User>(connection)
        .expect("Error loading users.");

    results
}

pub fn get_user_by_name(username_str: &str) -> Result<Option<User>, Error> {
    use schema::users::dsl::*;
    let connection = &mut establish_connection();

    match users
        .filter(username.eq(username_str))
        .select((id, username, password_hash, created_at, updated_at))
        .first::<User>(connection)
    {
        Ok(user) => Ok(Some(user)),
        Err(diesel::result::Error::NotFound) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn create_new_user<'a>(username: &'a str, password_hash: &'a str) -> Result<(), Error> {
    let conn = &mut establish_connection();

    let now: NaiveDateTime = Utc::now().naive_utc();

    let new_user = NewUser {
        username,
        password_hash,
        created_at: Some(now),
        updated_at: Some(now),
    };

    diesel::insert_into(users::table)
        .values(&new_user)
        .returning(User::as_returning())
        .get_result(conn)?;
    // .expect("Error saving new user");

    Ok(())
}
