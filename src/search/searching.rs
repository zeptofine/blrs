use std::fmt::Debug;

use crate::{info::BasicBuildInfo, search::OrdPlacement};

use super::query::{VersionSearchQuery, WildPlacement};

/// A matcher meant for searching through a list of builds (Used in tandem with [`VersionSearchQuery`]).
pub struct BInfoMatcher<'a, BI, N>
where
    BI: AsRef<BasicBuildInfo>,
    N: Eq + AsRef<str>,
{
    versions: &'a [(BI, N)],
}

impl<'a, BI, N> BInfoMatcher<'a, BI, N>
where
    BI: AsRef<BasicBuildInfo> + Debug,
    N: Eq + AsRef<str> + Debug,
{
    /// Creates a new instance of the matcher.
    pub fn new(versions: &'a [(BI, N)]) -> Self {
        BInfoMatcher { versions }
    }

    /// Finds all the `BI`s that are matched by query: [`VersionSearchQuery`].
    pub fn find_all(&self, query: &VersionSearchQuery) -> Vec<&(BI, N)> {
        let vs: Vec<(&BasicBuildInfo, &(BI, N))> = self
            .versions
            .iter()
            .filter_map(|x| {
                let (bi, nick) = &x;
                let build: &BasicBuildInfo = bi.as_ref();

                if let WildPlacement::Exact(r) = &query.repository {
                    if nick.as_ref() != *r {
                        return None;
                    }
                }
                if let WildPlacement::Exact(hash) = &query.build_hash {
                    if *build.ver.build_hash() != *hash {
                        return None;
                    }
                }
                if let WildPlacement::Exact(branch) = &query.branch {
                    if *build.ver.branch() != *branch {
                        return None;
                    }
                }

                Some((build, x))
            })
            .collect();

        let vs = match query.major {
            OrdPlacement::Any => vs,
            _ => query.major.find(
                &(vs.iter()
                    .map(|(v, _)| &v.version().major)
                    .collect::<Vec<_>>()),
                |idx| vs[idx],
            ),
        };
        let vs = match query.minor {
            OrdPlacement::Any => vs,
            _ => query.minor.find(
                &(vs.iter()
                    .map(|(v, _)| &v.version().minor)
                    .collect::<Vec<_>>()),
                |idx| vs[idx],
            ),
        };
        let vs = match query.patch {
            OrdPlacement::Any => vs,
            _ => query.patch.find(
                &(vs.iter()
                    .map(|(v, _)| &v.version().patch)
                    .collect::<Vec<_>>()),
                |idx| vs[idx],
            ),
        };

        let vs = match query.commit_dt {
            OrdPlacement::Any => vs,
            _ => query.commit_dt.find(
                &(vs.iter().map(|(v, _)| &v.commit_dt).collect::<Vec<_>>()),
                |idx| vs[idx],
            ),
        };

        vs.into_iter().map(|(_, x)| x).collect()
    }
}
