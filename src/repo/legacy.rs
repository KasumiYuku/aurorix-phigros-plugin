use std::sync::Arc;

use aurorix_db::PluginDb;
use sqlx::PgPool;

use super::{Repo, RepoResult, SCHEMA_VERSION};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LegacyReport {
    pub bindings: i64,
    pub profiles: i64,
    pub records: i64,
    pub adopted: bool,
}

impl LegacyReport {
    pub fn total(&self) -> i64 {
        self.bindings + self.profiles + self.records
    }

    pub fn summary(&self) -> String {
        format!(
            "旧数据 {} 条（绑定 {} · 档案 {} · 成绩 {}）",
            self.total(),
            self.bindings,
            self.profiles,
            self.records
        )
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LegacyError {
    #[error("数据库错误：{0}")]
    Db(#[from] aurorix_db::DbError),
    #[error("SQL 错误：{0}")]
    Sql(#[from] sqlx::Error),
    #[error("旧库版本 {found} 高于插件支持的 {supported}")]
    TooNew { found: i64, supported: i64 },
}

pub type LegacyResult<T> = Result<T, LegacyError>;

pub async fn import_once(db: &Arc<PluginDb>) -> LegacyResult<LegacyReport> {
    let repo = Repo::new(db.clone());
    let pool: &PgPool = repo.pool();
    let version = repo.schema_version().await.ok().flatten();
    if let Some(found) = version {
        if found > SCHEMA_VERSION {
            return Err(LegacyError::TooNew {
                found,
                supported: SCHEMA_VERSION,
            });
        }
    }
    let bindings = count(pool, "phg_binding").await?;
    let profiles = count(pool, "phg_profile").await?;
    let records = count(pool, "phg_record").await?;
    let adopted = bindings > 0 || profiles > 0 || records > 0;
    if version.is_none() && adopted {
        repo.set_schema_version(SCHEMA_VERSION)
            .await
            .map_err(|error| match error {
                super::RepoError::Db(inner) => LegacyError::Db(inner),
                super::RepoError::Sql(inner) => LegacyError::Sql(inner),
                other => LegacyError::Sql(sqlx::Error::Protocol(other.to_string())),
            })?;
    }
    Ok(LegacyReport {
        bindings,
        profiles,
        records,
        adopted,
    })
}

async fn count(pool: &PgPool, table: &str) -> LegacyResult<i64> {
    let sql = format!("SELECT count(*) FROM {table}");
    let row: (i64,) = sqlx::query_as(&sql).fetch_one(pool).await?;
    Ok(row.0)
}

pub async fn schema_version(db: &Arc<PluginDb>) -> RepoResult<Option<i64>> {
    Repo::new(db.clone()).schema_version().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_totals_and_text() {
        let report = LegacyReport {
            bindings: 12,
            profiles: 11,
            records: 3400,
            adopted: true,
        };
        assert_eq!(report.total(), 3423);
        assert!(report.summary().contains("绑定 12"));
        assert!(!LegacyReport::default().adopted);
    }
}
