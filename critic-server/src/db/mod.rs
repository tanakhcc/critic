//! Communication with the postgres database

use sqlx::{prelude::FromRow, query_as, Pool, Postgres};

use critic_shared::{ManuscriptMeta, PageMeta, VersificationScheme};

use crate::auth::{AuthenticatedUser, NormalizedTokenResponse, UserInfo};

// include tests
#[cfg(test)]
mod test;

pub async fn migrate(pool: &Pool<Postgres>) {
    match sqlx::migrate!().run(pool).await {
        Ok(_) => {}
        Err(e) => {
            panic!("Error migrating database: {e}");
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
    /// failed to insert a page
    CannotInsertPage(sqlx::Error),
    /// failed to get a page to minify
    CannotGetMinificationCandidate(sqlx::Error),
    CannotMarkPageMinificationFailed(sqlx::Error),
    CannotMarkPageMinified(sqlx::Error),
    CannotGetPage(sqlx::Error),
    PageAlreadyExists,
    CannotUpdateManuscript(sqlx::Error),
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
            Self::CannotInsertPage(e) => {
                write!(f, "Unable to insert page: {e}")
            }
            Self::CannotGetMinificationCandidate(e) => {
                write!(f, "Unable to get next page to minify: {e}")
            }
            Self::CannotMarkPageMinificationFailed(e) => {
                write!(f, "Unable to mark page minification as failed: {e}")
            }
            Self::CannotMarkPageMinified(e) => {
                write!(f, "Unable to mark page as minified: {e}")
            }
            Self::CannotGetPage(e) => {
                write!(f, "Unable to get page: {e}")
            }
            Self::PageAlreadyExists => {
                write!(
                    f,
                    "A page with this name already exists for this manuscript."
                )
            }
            Self::CannotUpdateManuscript(e) => {
                write!(f, "Unable to update manuscript metadata: {e}")
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
        user_info.login,
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
    msname: &str,
) -> Result<critic_shared::ManuscriptMeta, DBError> {
    sqlx::query_as!(
        critic_shared::ManuscriptMeta,
        "SELECT * FROM manuscript WHERE title = $1;",
        msname
    )
    .fetch_optional(pool)
    .await
    .map_err(DBError::CannotGetManuscript)?
    .ok_or(DBError::ManuscriptDoesNotExist(msname.to_string()))
}

async fn get_manuscript_page_rows(
    pool: &Pool<Postgres>,
    msid: i64,
) -> Result<Vec<PageMeta>, DBError> {
    sqlx::query_as!(
        PageMeta,
        "SELECT page.id, manuscript.id as manuscript_id, page.name, page.verse_start, page.verse_end
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
    msname: &str,
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

pub async fn get_manuscripts(
    pool: &Pool<Postgres>,
) -> Result<Vec<critic_shared::ManuscriptMeta>, DBError> {
    get_manuscripts_by_name(pool, None).await
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
            .into_iter()
            .collect(),
    )
}

pub async fn add_page(pool: &Pool<Postgres>, pagename: &str, msname: &str) -> Result<(), DBError> {
    // get manuscript id
    let ms_meta = get_manuscript_meta(pool, msname).await?;
    let mut tx = pool
        .begin()
        .await
        .map_err(DBError::CannotStartTransaction)?;

    if sqlx::query!(
        "SELECT id FROM page WHERE manuscript = $1 AND name = $2;",
        ms_meta.id,
        pagename
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(DBError::CannotGetPage)?
    .is_some()
    {
        return Err(DBError::PageAlreadyExists);
    };

    sqlx::query!(
        "INSERT INTO page (manuscript, name) VALUES ($1, $2);",
        ms_meta.id,
        pagename,
    )
    .execute(&mut *tx)
    .await
    .map(|_| {})
    .map_err(DBError::CannotInsertPage)?;

    tx.commit().await.map_err(DBError::CannotCommitTransaction)
}

/// page information plus the name of the MS it belongs to
#[derive(FromRow, PartialEq, Clone)]
struct _PageMetaWithMsName {
    manuscript_name: String,
    id: i64,
    manuscript_id: i64,
    name: String,
    verse_start: Option<i64>,
    verse_end: Option<i64>,
}
impl From<_PageMetaWithMsName> for (String, PageMeta) {
    fn from(value: _PageMetaWithMsName) -> Self {
        (
            value.manuscript_name,
            PageMeta {
                id: value.id,
                manuscript_id: value.manuscript_id,
                name: value.name,
                verse_start: value.verse_start,
                verse_end: value.verse_end,
            },
        )
    }
}

/// Get a single page that should be minified
///
/// These have minified = false and minification_failed = false
pub async fn get_page_to_minify(
    pool: &Pool<Postgres>,
    how_many: u8,
) -> Result<Vec<(String, PageMeta)>, DBError> {
    Ok(sqlx::query_as!(_PageMetaWithMsName,
        "SELECT manuscript.title as manuscript_name, page.id, manuscript as manuscript_id, name, verse_start, verse_end
         FROM page
         INNER JOIN manuscript on page.manuscript = manuscript.id
         WHERE minified = false AND minification_failed = false
         LIMIT $1;",
         how_many as i32)
        .fetch_all(pool)
        .await
        .map_err(DBError::CannotGetMinificationCandidate)?
        .into_iter()
        .map(|p_with_msname| p_with_msname.into()).collect()
    )
}

pub async fn mark_page_minifcation_failed(
    pool: &Pool<Postgres>,
    page_id: i64,
) -> Result<(), DBError> {
    sqlx::query!(
        "UPDATE page
         SET minification_failed = true
         WHERE id = $1;",
        page_id
    )
    .execute(pool)
    .await
    .map_err(DBError::CannotMarkPageMinificationFailed)
    .map(|_| {})
}

pub async fn mark_page_minified(pool: &Pool<Postgres>, page_id: i64) -> Result<(), DBError> {
    sqlx::query!(
        "UPDATE page
         SET minified = true
         WHERE id = $1;",
        page_id
    )
    .execute(pool)
    .await
    .map_err(DBError::CannotMarkPageMinified)
    .map(|_| {})
}

pub async fn update_ms_meta(pool: &Pool<Postgres>, data: &ManuscriptMeta) -> Result<(), DBError> {
    sqlx::query!(
            "UPDATE manuscript SET title = $1, institution = $2, collection = $3, hand_desc = $4, script_desc = $5 WHERE id = $6;",
            data.title,
            data.institution,
            data.collection,
            data.hand_desc,
            data.script_desc,
            data.id,
        )
        .execute(pool)
        .await
        .map(|_| {})
        .map_err(DBError::CannotUpdateManuscript)
}
