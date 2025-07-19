//! Communication with the postgres database

use sqlx::{query_as, Pool, Postgres};

use critic_shared::{PageMeta, VersificationScheme};

use crate::auth::{AuthenticatedUser, NormalizedTokenResponse, UserInfo};

// include tests
#[cfg(test)]
mod test;

pub async fn migrate(pool: &Pool<Postgres>) {
    match sqlx::migrate!().run(pool).await {
        Ok(_) => {}
        Err(e) => {
            panic!("Eeror migrating database: {e}");
        }
    }
}

#[derive(Debug)]
pub enum DBError {
    CannotStartTransaction(sqlx::Error),
    CannotCommitTransaction(sqlx::Error),
    CannotRollbackTransaction(sqlx::Error),
    CannotInsertOrUpdateUsersession(sqlx::Error),
    CannotGetUsersession(sqlx::Error),
    CannotGetManuscript(sqlx::Error),
    /// The manuscript we looked for simply does not exist
    ManuscriptDoesNotExist(String),
    /// Unable to add a manuscript
    CannotAddManuscript(sqlx::Error),
    /// Unable to get versification schemes
    CannotGetVersificationSchemes(sqlx::Error),
}
impl core::fmt::Display for DBError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::CannotStartTransaction(e) => {
                write!(f, "Unable to start transaction: {e}")
            }
            Self::CannotCommitTransaction(e) => {
                write!(f, "Unable to commit transaction: {e}")
            }
            Self::CannotRollbackTransaction(e) => {
                write!(f, "Unable to rollback transaction: {e}")
            }
            Self::CannotInsertOrUpdateUsersession(e) => {
                write!(f, "Unable to insert or update usersession: {e}")
            }
            Self::CannotGetUsersession(e) => {
                write!(f, "Unable to get usersession: {e}")
            }
            Self::CannotGetManuscript(e) => {
                write!(f, "Unable to get manuscript: {e}")
            }
            Self::ManuscriptDoesNotExist(msname) => {
                write!(f, "This manuscript does not exist: {msname}")
            }
            Self::CannotAddManuscript(e) => {
                write!(f, "Unable to add manuscript: {e}")
            }
            Self::CannotGetVersificationSchemes(e) => {
                write!(f, "Unable to get versification schemes: {e}")
            }
        }
    }
}
impl std::error::Error for DBError {}

pub async fn insert_or_update_user_session(
    pool: &Pool<Postgres>,
    user_info: UserInfo,
    token_res: NormalizedTokenResponse,
) -> Result<AuthenticatedUser, DBError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(DBError::CannotStartTransaction)?;

    let authenticated_user = query_as!(
        AuthenticatedUser,
        "insert into user_session (id, username, access_token, refresh_token, expires_at)
            values ($1, $2, $3, $4, $5)
            on conflict(username) do update
            set access_token = excluded.access_token,
            refresh_token = excluded.refresh_token,
            expires_at = excluded.expires_at,
            id = excluded.id
            returning *",
        user_info.id,
        user_info.username,
        token_res.access_token,
        token_res.refresh_token,
        token_res.expires_at,
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(DBError::CannotInsertOrUpdateUsersession)?;

    tx.commit()
        .await
        .map_err(DBError::CannotCommitTransaction)?;

    Ok(authenticated_user)
}

async fn get_manuscript_meta(
    pool: &Pool<Postgres>,
    msname: String,
) -> Result<critic_shared::ManuscriptMeta, DBError> {
    sqlx::query_as!(
        critic_shared::ManuscriptMeta,
        "SELECT * FROM manuscript WHERE title = $1;",
        msname
    )
    .fetch_optional(pool)
    .await
    .map_err(DBError::CannotGetManuscript)?
    .ok_or(DBError::ManuscriptDoesNotExist(msname))
}

async fn get_manuscript_page_rows(
    pool: &Pool<Postgres>,
    msid: i64,
) -> Result<Vec<PageMeta>, DBError> {
    sqlx::query_as!(
        PageMeta,
        "SELECT page.id, page.name, page.verse_start, page.verse_end
            FROM manuscript
            INNER JOIN page on page.manuscript = manuscript.id
            WHERE manuscript.id = $1
            ;",
        msid
    )
    .fetch_all(pool)
    .await
    .map_err(DBError::CannotGetManuscript)
}

/// Get the metainformation for a manuscript from the db
pub async fn get_manuscript(
    pool: &Pool<Postgres>,
    msname: String,
) -> Result<critic_shared::Manuscript, DBError> {
    let meta = get_manuscript_meta(pool, msname).await?;
    let pages = get_manuscript_page_rows(pool, meta.id).await?;
    Ok(critic_shared::Manuscript { meta, pages })
}

/// Get the metainformation for all manuscripts, excluding the page information
pub async fn get_manuscripts_by_name(
    pool: &Pool<Postgres>,
    msname: Option<String>,
) -> Result<Vec<critic_shared::ManuscriptMeta>, DBError> {
    if let Some(name) = msname {
        sqlx::query_as!(
            critic_shared::ManuscriptMeta,
            "SELECT * FROM manuscript WHERE title LIKE $1;",
            format!("%{name}%")
        )
        .fetch_all(pool)
        .await
        .map_err(DBError::CannotGetManuscript)
    } else {
        sqlx::query_as!(critic_shared::ManuscriptMeta, "SELECT * FROM manuscript;",)
            .fetch_all(pool)
            .await
            .map_err(DBError::CannotGetManuscript)
    }
}

pub async fn add_manuscript(pool: &Pool<Postgres>, msname: String) -> Result<(), DBError> {
    sqlx::query!("INSERT INTO manuscript (title) VALUES ($1);", msname)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(DBError::CannotAddManuscript)
}

pub async fn get_versification_schemes(
    pool: &Pool<Postgres>,
) -> Result<Vec<VersificationScheme>, DBError> {
    Ok(
        query_as!(VersificationScheme, "SELECT * FROM versification_scheme;")
            .fetch_all(pool)
            .await
            .map_err(DBError::CannotGetVersificationSchemes)?
            .into_iter().collect(),
    )
}
