use std::fmt::Debug;

use crate::info::BasicBuildInfo;

use super::query::{OrdPlacement, VersionSearchQuery, WildPlacement};

type RepoNickname = String;

/// A matcher meant for searching through a list of builds (Used in tandem with [`VersionSearchQuery`]).
pub struct BInfoMatcher<'a, BI>
where
    BI: AsRef<BasicBuildInfo>,
{
    versions: &'a [(BI, RepoNickname)],
}

impl<'a, BI> BInfoMatcher<'a, BI>
where
    BI: AsRef<BasicBuildInfo> + Debug,
{
    /// Creates a new instance of the matcher.
    pub fn new(versions: &'a [(BI, RepoNickname)]) -> Self {
        BInfoMatcher { versions }
    }

    /// Finds all the `BI`s that are matched by query: [`VersionSearchQuery`].
    pub fn find_all(&self, query: &VersionSearchQuery) -> Vec<&(BI, RepoNickname)> {
        let vs = self
            .versions
            .iter()
            .filter_map(|x| {
                let build: &BasicBuildInfo = x.0.as_ref();

                let r = match query.repository.clone() {
                    WildPlacement::Any => true,
                    WildPlacement::Exact(r) => x.1.clone() == r,
                };

                let b = match query.build_hash.clone() {
                    WildPlacement::Any => true,
                    WildPlacement::Exact(hash) => build.ver.build_hash() == hash,
                };
                let br = match query.branch.clone() {
                    WildPlacement::Any => true,
                    WildPlacement::Exact(branch) => build.ver.branch() == branch,
                };

                if r && b && br {
                    Some((build, x))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

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
