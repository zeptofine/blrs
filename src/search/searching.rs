use crate::info::{BasicBuildInfo, BlendBuild};

use super::query::{OrdPlacement, VersionSearchQuery, WildPlacement};

pub struct BInfoMatcher<'a> {
    versions: &'a [BasicBuildInfo],
}

impl<'a> BInfoMatcher<'a> {
    fn new(versions: &'a [BasicBuildInfo]) -> Self {
        BInfoMatcher { versions }
    }

    pub fn match_all(&self, query: &VersionSearchQuery) -> Vec<&BasicBuildInfo> {
        let vs = self
            .versions
            .iter()
            .filter(|build| match query.build_hash.clone() {
                WildPlacement::Any => true,
                WildPlacement::Exact(hash) => build.build_hash().is_some_and(|bh| bh == hash),
            })
            .filter(|build| match query.branch.clone() {
                WildPlacement::Any => true,
                WildPlacement::Exact(branch) => build.branch().is_some_and(|br| br == branch),
            })
            .collect::<Vec<&BasicBuildInfo>>();

        let vs = match query.major {
            OrdPlacement::Any => vs,
            _ => query
                .major
                .search(&(vs.iter().map(|v| &v.version.major).collect::<Vec<_>>()))
                .into_iter()
                .map(|i| vs[i])
                .collect(),
        };
        let vs = match query.minor {
            OrdPlacement::Any => vs,
            _ => query
                .minor
                .search(&(vs.iter().map(|v| &v.version.minor).collect::<Vec<_>>()))
                .into_iter()
                .map(|i| vs[i])
                .collect(),
        };
        let vs = match query.patch {
            OrdPlacement::Any => vs,
            _ => query
                .patch
                .search(&(vs.iter().map(|v| &v.version.patch).collect::<Vec<_>>()))
                .into_iter()
                .map(|i| vs[i])
                .collect(),
        };

        let vs = match query.commit_dt {
            OrdPlacement::Any => vs,
            _ => query
                .commit_dt
                .search(&(vs.iter().map(|v| &v.commit_dt).collect::<Vec<_>>()))
                .into_iter()
                .map(|i| vs[i])
                .collect(),
        };

        vs
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use chrono::DateTime;
    use semver::Version;

    use crate::info::BasicBuildInfo;

    use super::BInfoMatcher;

    static builds: LazyLock<Vec<BasicBuildInfo>> = LazyLock::new(|| {
        vec![
            BasicBuildInfo {
                version: Version::parse("1.2.3+stable").unwrap(),
                commit_dt: DateTime::parse_from_rfc3339("2020-05-04T00:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            },
            BasicBuildInfo {
                version: Version::parse("1.2.2+stable").unwrap(),
                commit_dt: DateTime::parse_from_rfc3339("2020-04-02T00:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            },
            BasicBuildInfo {
                version: Version::parse("1.2.1+daily").unwrap(),
                commit_dt: DateTime::parse_from_rfc3339("2020-03-01T00:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            },
            BasicBuildInfo {
                version: Version::parse("1.2.4+stable").unwrap(),
                commit_dt: DateTime::parse_from_rfc3339("2020-06-03T00:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            },
            BasicBuildInfo {
                version: Version::parse("3.6.14+lts").unwrap(),
                commit_dt: DateTime::parse_from_rfc3339("2024-07-16T00:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            },
            BasicBuildInfo {
                version: Version::parse("4.2.0+stable").unwrap(),
                commit_dt: DateTime::parse_from_rfc3339("2024-07-16T00:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            },
            BasicBuildInfo {
                version: Version::parse("4.3.0+daily").unwrap(),
                commit_dt: DateTime::parse_from_rfc3339("2024-07-30T00:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            },
            BasicBuildInfo {
                version: Version::parse("4.3.0+daily").unwrap(),
                commit_dt: DateTime::parse_from_rfc3339("2024-07-28T00:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            },
            BasicBuildInfo {
                version: Version::parse("4.3.1+daily").unwrap(),
                commit_dt: DateTime::parse_from_rfc3339("2024-07-20T00:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            },
        ]
    });

    #[test]
    fn test_binfo_matcher() {
        let bs = builds.clone();
        let matcher = BInfoMatcher::new(&bs);
    }
}
