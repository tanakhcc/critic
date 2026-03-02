//! Communication with the postgres database

use sqlx::{Pool, Postgres, QueryBuilder, prelude::FromRow, query_as};

use critic_shared::{
    LanguageMetadata, ManuscriptMeta, ModelMetadata, ModelType, OwnStatus, PageMeta, PageTodo,
    Point, RetrainOptions, SegmentedPage, VersificationScheme,
};

use crate::auth_types::{AuthenticatedUser, NormalizedTokenResponse, UserInfo};

pub mod auth_types;

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
    CannotGetModel(sqlx::Error),
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
    CannotUpdatePage(sqlx::Error),
    CannotGetPagesByQuery(sqlx::Error),
    CannotGetEditorInitialValue(sqlx::Error),
    CannotInsertTranscription(sqlx::Error),
    CannotPublish(sqlx::Error),
    CannotAddModel(sqlx::Error),
    CannotUpdateModel(sqlx::Error),
    CannotGetLanguage(sqlx::Error),
    CannotAddLanguage(sqlx::Error),
    CannotUpdateLanguage(sqlx::Error),
    CannotUpdateDefaultLanguage(sqlx::Error),
    CannotGetDefaultLanguage(sqlx::Error),
    CannotGetUnindexedChunks(sqlx::Error),
    CannotUpdateChunks(sqlx::Error),
    CannotGetOcr(sqlx::Error),
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
            Self::CannotUpdatePage(e) => {
                write!(f, "Unable to update page metadata: {e}")
            }
            Self::CannotGetPagesByQuery(e) => {
                write!(f, "Unable to get pages from query: {e}")
            }
            Self::CannotGetEditorInitialValue(e) => {
                write!(
                    f,
                    "Unable to get the seeding values for editor initial state: {e}"
                )
            }
            Self::CannotInsertTranscription(e) => {
                write!(f, "Unable to insert transcription: {e}")
            }
            Self::CannotPublish(e) => {
                write!(f, "Unable to publish a transcription: {e}")
            }
            Self::CannotGetModel(e) => {
                write!(f, "Unable to get model: {e}")
            }
            Self::CannotAddModel(e) => {
                write!(f, "Unable to add model: {e}")
            }
            Self::CannotUpdateModel(e) => {
                write!(f, "Unable to update model: {e}")
            }
            Self::CannotGetLanguage(e) => {
                write!(f, "Unable to get language: {e}")
            }
            Self::CannotAddLanguage(e) => {
                write!(f, "Unable to add language: {e}")
            }
            Self::CannotUpdateLanguage(e) => {
                write!(f, "Unable to update language: {e}")
            }
            Self::CannotUpdateDefaultLanguage(e) => {
                write!(f, "Unable to update the project default language: {e}")
            }
            Self::CannotGetDefaultLanguage(e) => {
                write!(f, "Unable to get the project default language: {e}")
            }
            Self::CannotGetUnindexedChunks(e) => {
                write!(f, "Unable to get next unindexed chunks: {e}")
            }
            Self::CannotUpdateChunks(e) => {
                write!(f, "Unable to update chunks: {e}")
            }
            Self::CannotGetOcr(e) => {
                write!(f, "Unable to get OCR record: {e}")
            }
        }
    }
}
impl std::error::Error for DBError {}

pub async fn get_user(
    pool: &Pool<Postgres>,
    user_id: &i32,
) -> Result<Option<AuthenticatedUser>, DBError> {
    sqlx::query_as!(
        AuthenticatedUser,
        "select * from user_session where id = $1",
        user_id,
    )
    .fetch_optional(&*pool)
    .await
    .map_err(|e| DBError::CannotGetUsersession(e))
}

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
        "SELECT
        id,
        title,
        institution,
        collection,
        hand_desc,
        script_desc,
        COALESCE(
            manuscript.language,
            (SELECT language FROM default_language
             WHERE id = 1)) as language
        FROM manuscript WHERE title = $1;",
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
        "SELECT page.id, manuscript.id as manuscript_id, page.name, page.verse_start, page.verse_end,
            COALESCE(
                page.language,
                manuscript.language,
                (SELECT language FROM default_language
                 WHERE id = 1)) as language
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
            // this is a view, so sqlx cannot infer that id and title are NOT NULL
            r#"SELECT id as "id!", title as "title!", institution, collection, hand_desc, script_desc, language FROM manuscript_with_language WHERE title LIKE $1;"#,
            format!("%{name}%")
        )
        .fetch_all(pool)
        .await
        .map_err(DBError::CannotGetManuscript)
    } else {
        sqlx::query_as!(critic_shared::ManuscriptMeta,
            r#"SELECT id as "id!", title as "title!", institution, collection, hand_desc, script_desc, language FROM manuscript_with_language;"#)
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

pub async fn add_manuscript(
    pool: &Pool<Postgres>,
    msname: &str,
    language: Option<i64>,
) -> Result<(), DBError> {
    sqlx::query!(
        "INSERT INTO manuscript (title, language) VALUES ($1, $2);",
        msname,
        language,
    )
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
    language: Option<i64>,
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
                language: value.language,
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
    Ok(sqlx::query_as!(
        _PageMetaWithMsName,
        "SELECT
            manuscript.title as manuscript_name,
            page.id,
            manuscript as manuscript_id,
            name,
            verse_start,
            verse_end,
            COALESCE(
                 manuscript.language,
                 page.language,
                 (SELECT language FROM default_language
                  WHERE id = 1))
             as language
         FROM page
         INNER JOIN manuscript on page.manuscript = manuscript.id
         WHERE minified = false AND minification_failed = false
         LIMIT $1;",
        how_many as i32
    )
    .fetch_all(pool)
    .await
    .map_err(DBError::CannotGetMinificationCandidate)?
    .into_iter()
    .map(|p_with_msname| p_with_msname.into())
    .collect())
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
            "UPDATE manuscript SET title = $1, institution = $2, collection = $3, hand_desc = $4, script_desc = $5, language = $6 WHERE id = $7;",
            data.title,
            data.institution,
            data.collection,
            data.hand_desc,
            data.script_desc,
            data.language,
            data.id,
        )
        .execute(pool)
        .await
        .map(|_| {})
        .map_err(DBError::CannotUpdateManuscript)
}

struct QueryTerm<'a> {
    qtype: QueryType,
    qstr: &'a str,
}
/// The different things we can search for.
enum QueryType {
    ManuscriptEqual,
    ManuscriptContains,
    LanguageEqual,
    LanguageContains,
    PageEqual,
    PageContains,
}

/// Decompose a query such as
/// ```text
/// ms=IIB17+ lang=hbo-Hebr page:3
/// ```
fn decompose_query(query: &str) -> Vec<QueryTerm> {
    let mut res = Vec::<QueryTerm>::new();
    for item in query.split_whitespace() {
        match item {
            // TODO: allow quoted terms like ms:'Babylonicus Petropolitanus'
            // This requires a proper lexer, and I am to lazy for that right now
            //
            // Good first Issue if you want to build one.
            s if s.starts_with("ms:") => {
                res.push(QueryTerm {
                    qtype: QueryType::ManuscriptContains,
                    qstr: &s[3..],
                });
            }
            s if s.starts_with("ms=") => {
                res.push(QueryTerm {
                    qtype: QueryType::ManuscriptEqual,
                    qstr: &s[3..],
                });
            }
            s if s.starts_with("lang:") => {
                res.push(QueryTerm {
                    qtype: QueryType::LanguageContains,
                    qstr: &s[5..],
                });
            }
            s if s.starts_with("lang=") => {
                res.push(QueryTerm {
                    qtype: QueryType::LanguageEqual,
                    qstr: &s[5..],
                });
            }
            s if s.starts_with("page:") => {
                res.push(QueryTerm {
                    qtype: QueryType::PageContains,
                    qstr: &s[5..],
                });
            }
            s if s.starts_with("page=") => {
                res.push(QueryTerm {
                    qtype: QueryType::PageEqual,
                    qstr: &s[5..],
                });
            }
            _ => {}
        }
    }
    res
}

/// turn a query into a free standing SQL condition expression like `name = foo`
fn query_term_to_sql_filter<'a>(
    QueryTerm { qtype, qstr }: QueryTerm<'a>,
    mut current_query: QueryBuilder<'a, Postgres>,
) -> QueryBuilder<'a, Postgres> {
    match qtype {
        QueryType::ManuscriptEqual => {
            current_query.push(" manuscript.title = ");
            current_query.push_bind(qstr);
        }
        QueryType::ManuscriptContains => {
            current_query.push(" manuscript.title LIKE CONCAT('%', ");
            current_query.push_bind(qstr);
            current_query.push(", '%')");
        }
        QueryType::LanguageEqual => {
            current_query.push(" manuscript.lang = ");
            current_query.push_bind(qstr);
        }
        QueryType::LanguageContains => {
            current_query.push(" manuscript.lang LIKE CONCAT('%', ");
            current_query.push_bind(qstr);
            current_query.push(", '%')");
        }
        QueryType::PageEqual => {
            current_query.push(" page.name = ");
            current_query.push_bind(qstr);
        }
        QueryType::PageContains => {
            current_query.push(" page.name LIKE CONCAT('%', ");
            current_query.push_bind(qstr);
            current_query.push(", '%')");
        }
    };
    current_query
}

const DEFAULT_PAGINATION_SIZE: i32 = 50;

#[derive(FromRow, Debug)]
struct _GetPagesByQueryRow {
    manuscript_name: String,
    page_name: String,
    verse_start: Option<String>,
    verse_end: Option<String>,
    transcriptions_published: i64,
    transcriptions_started: i64,
    transcriptions_by_this_user: i64,
    published_by_this_user: i64,
}

pub async fn get_pages_by_query(
    pool: &Pool<Postgres>,
    query: &str,
    this_username: &str,
    page: i32,
) -> Result<Vec<PageTodo>, DBError> {
    let decomposed_query = decompose_query(query);
    let mut builder = QueryBuilder::new(
        "SELECT
            manuscript.title as manuscript_name,
            page.id,
            page.name as page_name,
            verse_start,
            verse_end,
            count(*) FILTER (WHERE transcription.id is not NULL) as transcriptions_started,
            count(*) FILTER (WHERE transcription.published) as transcriptions_published,
            count(*) FILTER (WHERE transcription.username = ",
    );
    // couting transcriptions started by this user
    builder.push_bind(this_username);
    builder.push(
        ") as transcriptions_by_this_user,
            count(*) FILTER (WHERE transcription.username = ",
    );
    // counting published transcriptions by this user separately
    builder.push_bind(this_username);
    builder.push(
        " AND transcription.published) as published_by_this_user
         FROM page
         INNER JOIN manuscript on page.manuscript = manuscript.id
         LEFT OUTER JOIN transcription on page.id = transcription.page
         LEFT OUTER JOIN reconciliation on page.id = reconciliation.page
         WHERE
         ",
    );
    // user specified search filters
    for query in decomposed_query {
        builder = query_term_to_sql_filter(query, builder);
        builder.push(" AND ");
    }
    // exclude MSS with reconciliation already in progress
    builder.push(" reconciliation.id is NULL");

    builder.push(" GROUP BY (manuscript_name, page.id, page_name, verse_start, verse_end) ");

    // exclude MSS with two or more transcriptions, but always show MSS where the user has started
    // a transcription
    // y you has no WHERE on output column aliases @sql??
    builder.push(" HAVING (count(*) FILTER (WHERE transcription.published) < 2) OR (count(*) FILTER (WHERE transcription.username = ");
    builder.push_bind(this_username);
    builder.push(") = 1) ");

    builder.push(" ORDER BY transcriptions_published DESC, transcriptions_started ASC ");
    builder.push(" LIMIT ");
    builder.push_bind(DEFAULT_PAGINATION_SIZE);
    builder.push(" OFFSET ");
    builder.push_bind(page * DEFAULT_PAGINATION_SIZE);
    builder.push(";");

    let page_query_rows = builder
        .build_query_as::<_GetPagesByQueryRow>()
        .fetch_all(pool)
        .await
        .map_err(DBError::CannotGetPagesByQuery)?;

    let mut res = Vec::<PageTodo>::new();
    for item in page_query_rows {
        res.push(PageTodo {
            manuscript_name: item.manuscript_name,
            page_name: item.page_name,
            verse_start: item.verse_start,
            verse_end: item.verse_end,
            // this will always be nonnegative, because it is Count(*) from SQL
            // It should never be high (in practice), and we certainly don't care if this is wrong
            transcriptions_started: item.transcriptions_started.try_into().unwrap_or(u8::MAX),
            transcriptions_published: item
                .transcriptions_published
                .try_into()
                .expect("Query filters for more transcriptions published"),
            this_user_status: if item.transcriptions_by_this_user == 1
                && item.published_by_this_user == 1
            {
                OwnStatus::Published
            } else if item.transcriptions_by_this_user == 1 {
                OwnStatus::Started
            } else {
                OwnStatus::None
            },
        });
    }
    Ok(res)
}

pub struct EditorInitialValue {
    pub meta: ManuscriptMeta,
    pub user_has_started: bool,
    pub verse_start: Option<i64>,
    pub verse_end: Option<i64>,
}

struct _EditorIVSeed {
    manuscript_id: i64,
    institution: Option<String>,
    collection: Option<String>,
    hand_desc: Option<String>,
    script_desc: Option<String>,
    default_language: Option<i64>,
    verse_start: Option<i64>,
    verse_end: Option<i64>,
    transcriptions_by_this_user: Option<i64>,
}

/// Get the initial value for a transcription editor
pub async fn get_editor_initial_value(
    pool: &Pool<Postgres>,
    msname: &str,
    pagename: &str,
    this_username: &str,
) -> Result<EditorInitialValue, DBError> {
    let seed = sqlx::query_as!(
        _EditorIVSeed,
        "SELECT
            manuscript.id as manuscript_id,
            manuscript.institution,
            manuscript.collection,
            manuscript.hand_desc,
            manuscript.script_desc,
            COALESCE(
                 manuscript.language,
                 page.language,
                 (SELECT language FROM default_language
                  WHERE id = 1))
             as default_language,
            page.verse_start,
            page.verse_end,
            COUNT(*) FILTER (WHERE transcription.username = $3) as transcriptions_by_this_user
        FROM
            page
        INNER JOIN manuscript
            ON manuscript.id = page.manuscript
        LEFT OUTER JOIN transcription
            ON page.id = transcription.page
        WHERE manuscript.title = $1 AND page.name = $2
        GROUP BY (manuscript.id, manuscript.institution, manuscript.collection, manuscript.hand_desc, manuscript.script_desc, manuscript.language, page.verse_start, page.verse_end, page.language)
        ;",
        msname,
        pagename,
        this_username
    )
    .fetch_one(pool)
    .await
    .map_err(DBError::CannotGetEditorInitialValue)?;
    Ok(EditorInitialValue {
        user_has_started: seed.transcriptions_by_this_user.unwrap_or_default() > 0,
        verse_start: seed.verse_start,
        verse_end: seed.verse_end,
        meta: ManuscriptMeta {
            id: seed.manuscript_id,
            title: msname.to_string(),
            institution: seed.institution,
            collection: seed.collection,
            hand_desc: seed.hand_desc,
            script_desc: seed.script_desc,
            language: seed.default_language,
        },
    })
}

pub async fn add_transcription(
    pool: &Pool<Postgres>,
    msname: &str,
    pagename: &str,
    username: &str,
) -> Result<(), DBError> {
    sqlx::query!(
        "
        INSERT INTO transcription
            (page, username)
        VALUES
            ((SELECT page.id
                FROM page
                INNER JOIN manuscript
                    ON page.manuscript = manuscript.id
                WHERE manuscript.title = $1 AND page.name = $2),
             $3)
        ON CONFLICT DO NOTHING;",
        msname,
        pagename,
        username
    )
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(DBError::CannotInsertTranscription)
}

/// set this transcription as published
pub async fn publish_transcription(
    pool: &Pool<Postgres>,
    msname: &str,
    pagename: &str,
    username: &str,
) -> Result<(), DBError> {
    sqlx::query!(
        "UPDATE transcription
        SET published = true
        FROM page p, manuscript m
        WHERE p.id = transcription.page
            AND m.id = p.manuscript
            AND m.title = $1
            AND p.name = $2
            AND transcription.username = $3
        ",
        msname,
        pagename,
        username
    )
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(DBError::CannotPublish)
}

/// Get a list of all models of the given type
pub async fn get_models(
    pool: &Pool<Postgres>,
    model_type: ModelType,
) -> Result<Vec<ModelMetadata>, DBError> {
    match model_type {
        ModelType::Segmentation => Ok(sqlx::query!(
            r#"SELECT id, name, retrain_every_days, retrain_keep_versions FROM segmentation_model"#,
        )
        .fetch_all(pool)
        .await
        .map_err(DBError::CannotGetModel)?
        .into_iter()
        .map(|raw| ModelMetadata {
            id: raw.id,
            name: raw.name,
            model_type: ModelType::Segmentation,
            retrain_options: if let Some(every_days) = raw.retrain_every_days {
                Some(RetrainOptions {
                    every_days: every_days.try_into().unwrap_or_default(),
                    keep_versions: raw.retrain_keep_versions.map(|as_i32| {
                        <i32 as TryInto<u16>>::try_into(as_i32.clone()).unwrap_or_default()
                    }),
                })
            } else {
                None
            },
        })
        .collect()),
        ModelType::Recognition => Ok(sqlx::query!(
            r#"SELECT id, name, retrain_every_days, retrain_keep_versions FROM recognition_model"#,
        )
        .fetch_all(pool)
        .await
        .map_err(DBError::CannotGetModel)?
        .into_iter()
        .map(|raw| ModelMetadata {
            id: raw.id,
            name: raw.name,
            model_type: ModelType::Recognition,
            retrain_options: if let Some(every_days) = raw.retrain_every_days {
                Some(RetrainOptions {
                    every_days: every_days.try_into().unwrap_or_default(),
                    keep_versions: raw.retrain_keep_versions.map(|as_i32| {
                        <i32 as TryInto<u16>>::try_into(as_i32.clone()).unwrap_or_default()
                    }),
                })
            } else {
                None
            },
        })
        .collect()),
    }
}

/// Get a list of all models of the given type
pub async fn get_model_by_id(
    pool: &Pool<Postgres>,
    id: i64,
    model_type: ModelType,
) -> Result<ModelMetadata, DBError> {
    match model_type {
        ModelType::Recognition => sqlx::query!(
            r#"SELECT id, name, retrain_every_days, retrain_keep_versions FROM recognition_model
                WHERE id = $1
                "#,
            id,
        )
        .fetch_one(pool)
        .await
        .map(|raw| ModelMetadata {
            id: raw.id,
            name: raw.name,
            model_type: ModelType::Recognition,
            retrain_options: if let Some(every_days) = raw.retrain_every_days {
                Some(RetrainOptions {
                    every_days: every_days.try_into().unwrap_or_default(),
                    keep_versions: raw.retrain_keep_versions.map(|as_i32| {
                        <i32 as TryInto<u16>>::try_into(as_i32.clone()).unwrap_or_default()
                    }),
                })
            } else {
                None
            },
        })
        .map_err(DBError::CannotGetModel),
        ModelType::Segmentation => sqlx::query!(
            r#"SELECT id, name, retrain_every_days, retrain_keep_versions FROM segmentation_model
                WHERE id = $1
                "#,
            id,
        )
        .fetch_one(pool)
        .await
        .map(|raw| ModelMetadata {
            id: raw.id,
            name: raw.name,
            model_type: ModelType::Segmentation,
            retrain_options: if let Some(every_days) = raw.retrain_every_days {
                Some(RetrainOptions {
                    every_days: every_days.try_into().unwrap_or_default(),
                    keep_versions: raw.retrain_keep_versions.map(|as_i32| {
                        <i32 as TryInto<u16>>::try_into(as_i32.clone()).unwrap_or_default()
                    }),
                })
            } else {
                None
            },
        })
        .map_err(DBError::CannotGetModel),
    }
}

pub async fn add_model_with_default_options(
    pool: &Pool<Postgres>,
    new_name: &str,
    model_type: ModelType,
) -> Result<i64, DBError> {
    match model_type {
        ModelType::Recognition => sqlx::query!(
            r#"INSERT INTO recognition_model (name) VALUES ($1) returning id"#,
            new_name,
        )
        .fetch_one(pool)
        .await
        .map(|record| record.id)
        .map_err(DBError::CannotAddModel),
        ModelType::Segmentation => sqlx::query!(
            r#"INSERT INTO segmentation_model (name) VALUES ($1) returning id"#,
            new_name,
        )
        .fetch_one(pool)
        .await
        .map(|record| record.id)
        .map_err(DBError::CannotAddModel),
    }
}

pub async fn update_model(
    pool: &Pool<Postgres>,
    id: i64,
    model_type: ModelType,
    retraining_opts: &Option<RetrainOptions>,
) -> Result<(), DBError> {
    match model_type {
        ModelType::Segmentation => {
            sqlx::query!(
                "UPDATE segmentation_model set retrain_every_days = $1, retrain_keep_versions = $2 WHERE id = $3",
                retraining_opts.as_ref().map(|ro| ro.every_days as i32),
                retraining_opts
                    .as_ref()
                    .map(|ro| ro.keep_versions.map(|x| x as i32))
                    .flatten(),
                id
            )
            .execute(pool)
            .await
            .map(|_| ())
            .map_err(DBError::CannotUpdateModel)
        }
        ModelType::Recognition => {
            sqlx::query!(
                "UPDATE recognition_model set retrain_every_days = $1, retrain_keep_versions = $2 WHERE id = $3",
                retraining_opts.as_ref().map(|ro| ro.every_days as i32),
                retraining_opts
                    .as_ref()
                    .map(|ro| ro.keep_versions.map(|x| x as i32))
                    .flatten(),
                id
            )
            .execute(pool)
            .await
            .map(|_| ())
            .map_err(DBError::CannotUpdateModel)
        }
    }
}

pub async fn get_languages(pool: &Pool<Postgres>) -> Result<Vec<LanguageMetadata>, DBError> {
    Ok(sqlx::query!("SELECT * FROM language")
        .fetch_all(pool)
        .await
        .map_err(DBError::CannotGetLanguage)?
        .into_iter()
        .map(|row| LanguageMetadata {
            id: row.id,
            name: row.name,
            segmentation_model_id: row.segmentation_model,
            recognition_model_id: row.recognition_model,
            equality_alphabet: row.equality_alphabet,
        })
        .collect())
}

pub async fn get_language_by_name(
    pool: &Pool<Postgres>,
    language: &str,
) -> Result<Option<LanguageMetadata>, DBError> {
    Ok(
        sqlx::query!("SELECT * FROM language WHERE name = $1", language)
            .fetch_optional(pool)
            .await
            .map_err(DBError::CannotGetLanguage)?
            .map(|row| LanguageMetadata {
                id: row.id,
                name: row.name,
                segmentation_model_id: row.segmentation_model,
                recognition_model_id: row.recognition_model,
                equality_alphabet: row.equality_alphabet,
            }),
    )
}

pub async fn get_language_by_id(
    pool: &Pool<Postgres>,
    language_id: i64,
) -> Result<Option<LanguageMetadata>, DBError> {
    Ok(
        sqlx::query!("SELECT * FROM language WHERE id = $1", language_id)
            .fetch_optional(pool)
            .await
            .map_err(DBError::CannotGetLanguage)?
            .map(|row| LanguageMetadata {
                id: row.id,
                name: row.name,
                segmentation_model_id: row.segmentation_model,
                recognition_model_id: row.recognition_model,
                equality_alphabet: row.equality_alphabet,
            }),
    )
}

pub async fn add_language_with_default_options(
    pool: &Pool<Postgres>,
    language: &str,
) -> Result<(), DBError> {
    sqlx::query!("INSERT INTO language (name) VALUES ($1)", language)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(DBError::CannotAddLanguage)
}

pub async fn update_language(
    pool: &Pool<Postgres>,
    language: &str,
    segmentation_model_id: Option<i64>,
    recognition_model_id: Option<i64>,
    equality_alphabet: Option<String>,
) -> Result<(), DBError> {
    sqlx::query!(
        "UPDATE language SET segmentation_model = $1, recognition_model = $2, equality_alphabet = $3 WHERE name = $4",
        segmentation_model_id,
        recognition_model_id,
        equality_alphabet,
        language
    )
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(DBError::CannotUpdateLanguage)
}

pub async fn update_default_language(
    pool: &Pool<Postgres>,
    language_id: Option<i64>,
) -> Result<(), DBError> {
    sqlx::query!("UPDATE default_language SET language = $1;", language_id)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(DBError::CannotUpdateDefaultLanguage)
}

pub async fn get_default_language(
    pool: &Pool<Postgres>,
) -> Result<Option<LanguageMetadata>, DBError> {
    sqlx::query_as!(
        LanguageMetadata,
        "SELECT
            language.id,
            language.name,
            language.segmentation_model as segmentation_model_id,
            language.recognition_model as recognition_model_id,
            language.equality_alphabet as equality_alphabet
         FROM language
         INNER JOIN default_language on language.id = default_language.language
         LIMIT 1;"
    )
    .fetch_optional(pool)
    .await
    .map_err(DBError::CannotGetDefaultLanguage)
}

/// Set the language for a page
pub async fn update_page_language(
    pool: &Pool<Postgres>,
    language_id: Option<i64>,
    ms_name: &str,
    page_name: &str,
) -> Result<(), DBError> {
    sqlx::query!("UPDATE page SET language = $1 FROM manuscript WHERE page.manuscript = manuscript.id AND manuscript.title = $2 AND page.name = $3;", language_id, ms_name, page_name)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(DBError::CannotUpdatePage)
}

/// Get the language for a page
pub async fn get_page_language(
    pool: &Pool<Postgres>,
    ms_name: &str,
    page_name: &str,
) -> Result<Option<i64>, DBError> {
    sqlx::query!("SELECT page.language FROM page INNER JOIN manuscript ON manuscript.id = page.manuscript WHERE manuscript.title = $1 AND page.name = $2;", ms_name, page_name)
        .fetch_one(pool)
        .await
        .map(|row| row.language)
        .map_err(DBError::CannotGetLanguage)
}

/// A chunk of the base corpus, representing one row in the DB
///
/// Note that we expect the base corpus to be ingested into the DB by an external tool.
/// You can consider [this example for the MapM
/// data](https://github.com/curatorsigma/mapm-to-critic).
#[derive(Debug, FromRow)]
pub struct BaseCorpusChunk {
    pub id: i64,
    /// The language used in this chunk
    pub language: i64,
    /// the critic-tei-xml for this chunk
    pub content: String,
    /// The versification scheme that is relevant for this chunk
    pub versification_scheme: i64,
    /// the first verse (given versification-scheme-agnostic) in this chunk
    pub verse_start: i64,
    /// the last verse (given versification-scheme-agnostic) in this chunk
    pub verse_end: i64,
}
impl From<_BaseCorpusChunkWithEqualityAlphabet> for (BaseCorpusChunk, String) {
    fn from(value: _BaseCorpusChunkWithEqualityAlphabet) -> Self {
        (
            BaseCorpusChunk {
                id: value.id,
                language: value.language,
                content: value.content,
                versification_scheme: value.versification_scheme,
                verse_start: value.verse_start,
                verse_end: value.verse_end,
            },
            value.equality_alphabet,
        )
    }
}

/// A chunk of the base corpus, representing one row in the DB
///
/// Note that we expect the base corpus to be ingested into the DB by an external tool.
/// You can consider [this example for the MapM
/// data](https://github.com/curatorsigma/mapm-to-critic).
#[derive(Debug, FromRow)]
struct _BaseCorpusChunkWithEqualityAlphabet {
    id: i64,
    language: i64,
    content: String,
    versification_scheme: i64,
    verse_start: i64,
    verse_end: i64,
    equality_alphabet: String,
}

pub async fn get_unindexed_chunks(
    pool: &Pool<Postgres>,
    number_of_chunks: i64,
) -> Result<Vec<BaseCorpusChunk>, DBError> {
    sqlx::query_as!(
        BaseCorpusChunk,
        r#"SELECT
            id as "id!",
            language as "language!",
            content as "content!",
            versification_scheme as "versification_scheme!",
            verse_start as "verse_start!",
            verse_end as "verse_end!"
        FROM base_corpus
        WHERE indexed = false LIMIT $1;"#,
        number_of_chunks,
    )
    .fetch_all(&*pool)
    .await
    .map_err(DBError::CannotGetUnindexedChunks)
}

pub async fn get_unindexed_chunks_with_equality_alphabet(
    pool: &Pool<Postgres>,
    number_of_chunks: i64,
) -> Result<Vec<(BaseCorpusChunk, String)>, DBError> {
    sqlx::query_as!(
        _BaseCorpusChunkWithEqualityAlphabet,
        r#"SELECT
            base_corpus.id as "id!",
            language as "language!",
            language.equality_alphabet as "equality_alphabet!",
            content as "content!",
            versification_scheme as "versification_scheme!",
            verse_start as "verse_start!",
            verse_end as "verse_end!"
        FROM base_corpus
        INNER JOIN language ON language.id = base_corpus.language
        WHERE indexed = false LIMIT $1;"#,
        number_of_chunks,
    )
    .fetch_all(&*pool)
    .await
    .map(|v| v.into_iter().map(|c| c.into()).collect::<Vec<_>>())
    .map_err(DBError::CannotGetUnindexedChunks)
}

pub async fn set_chunks_indexed(pool: &Pool<Postgres>, chunks: &[i64]) -> Result<(), DBError> {
    sqlx::query!(
        "UPDATE base_corpus SET indexed = true WHERE id = ANY($1);",
        chunks
    )
    .execute(&*pool)
    .await
    .map(|_| ())
    .map_err(DBError::CannotUpdateChunks)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaselineTask {
    pub manuscript: String,
    pub page: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OcrTask {
    pub manuscript: String,
    pub page: String,
    pub baselines: Vec<(Point, Point)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KrakenTask {
    Baseline(BaselineTask),
    Ocr(OcrTask),
}

pub async fn get_next_kraken_task(pool: &Pool<Postgres>) -> Result<Option<KrakenTask>, DBError> {
    match sqlx::query!("SELECT manuscript.title, page.name FROM page INNER JOIN manuscript on manuscript.id = page.manuscript WHERE minified AND should_baseline;")
        .fetch_optional(&*pool)
        .await
        .map_err(DBError::CannotGetPage)?
        .map(|r| BaselineTask{ manuscript: r.title, page: r.name, })
        {
            None => {
                match sqlx::query!(
                        "SELECT manuscript.title, page.name, page.id FROM page INNER JOIN manuscript on manuscript.id = page.manuscript WHERE minified AND should_ocr;"
                    )
                    .fetch_optional(&*pool)
                    .await
                    .map_err(DBError::CannotGetPage)?
                {
                    None => {
                        Ok(None)
                    }
                    Some(x) => {
                        let manuscript = x.title;
                        let page = x.name;
                        let page_id = x.id;
                        let baselines = sqlx::query!(
                            "SELECT baseline FROM line INNER JOIN region ON line.region = region.id WHERE page = $1",
                            page_id,
                        )
                            .fetch_all(&*pool).await.map_err(DBError::CannotGetOcr)?.into_iter().map(|r|
                                (
                                    Point { x: r.baseline.start_x.round() as u32, y: r.baseline.start_y.round() as u32 },
                                    Point { x: r.baseline.end_x.round() as u32, y: r.baseline.end_y.round() as u32 },
                                )).collect::<Vec<_>>();
                        Ok(Some(KrakenTask::Ocr(OcrTask { manuscript, page, baselines })))
                    }
                }
            }
            Some(x) =>{
                Ok(Some(KrakenTask::Baseline(x)))
            }
        }
}

/// Get the model to use when dealing with a specified page.
///
/// Note that this does not return an actual file, only the metadata from which you can calculate
/// the correct base path to get the mlmodel to use (or an older model)
pub async fn get_model_for_page(
    pool: &Pool<Postgres>,
    pagename: &str,
    model_type: ModelType,
) -> Result<ModelMetadata, DBError> {
    match model_type {
        ModelType::Segmentation => {
            sqlx::query!(
                "SELECT segmentation_model.id, segmentation_model.name, segmentation_model.retrain_every_days, segmentation_model.retrain_keep_versions
                 FROM language
                 INNER JOIN segmentation_model ON segmentation_model.id = language.segmentation_model
                 WHERE language.id = (
                    SELECT
                        COALESCE(page.language,
                                manuscript.language,
                                (SELECT language
                                    FROM default_language WHERE id = 1))
                    FROM page
                    INNER JOIN manuscript on manuscript.id = page.manuscript
                    WHERE page.name = $1);",
                pagename,
                )
                .fetch_one(&*pool)
                .await
                .map(|raw| ModelMetadata {
                    id: raw.id,
                    name: raw.name,
                    model_type: ModelType::Segmentation,
                    retrain_options: if let Some(every_days) = raw.retrain_every_days {
                        Some(RetrainOptions {
                            every_days: every_days.try_into().unwrap_or_default(),
                            keep_versions: raw.retrain_keep_versions.map(|as_i32| {
                                <i32 as TryInto<u16>>::try_into(as_i32.clone()).unwrap_or_default()
                            }),
                        })
                    } else {
                        None
                    },
                })
                .map_err(DBError::CannotGetModel)
            }
        ModelType::Recognition => {
            sqlx::query!(
                "SELECT recognition_model.id, recognition_model.name, recognition_model.retrain_every_days, recognition_model.retrain_keep_versions
                 FROM language
                 INNER JOIN recognition_model ON recognition_model.id = language.recognition_model
                 WHERE language.id = (
                    SELECT
                        COALESCE(page.language,
                                manuscript.language,
                                (SELECT language
                                    FROM default_language WHERE id = 1))
                    FROM page
                    INNER JOIN manuscript on manuscript.id = page.manuscript
                    WHERE page.name = $1);",
                pagename,
                )
                .fetch_one(&*pool)
                .await
                .map(|raw| ModelMetadata {
                    id: raw.id,
                    name: raw.name,
                    model_type: ModelType::Recognition,
                    retrain_options: if let Some(every_days) = raw.retrain_every_days {
                        Some(RetrainOptions {
                            every_days: every_days.try_into().unwrap_or_default(),
                            keep_versions: raw.retrain_keep_versions.map(|as_i32| {
                                <i32 as TryInto<u16>>::try_into(as_i32.clone()).unwrap_or_default()
                            }),
                        })
                    } else {
                        None
                    },
                })
                .map_err(DBError::CannotGetModel)
            }
    }
}

pub async fn insert_segmentation(
    pool: &Pool<Postgres>,
    manuscript_name: &str,
    page_name: &str,
    segmentation: SegmentedPage,
) -> Result<(), DBError> {
    for region in segmentation.regions.into_iter() {
        // insert the region
        // insert the baselines
    }
    todo!()
}
