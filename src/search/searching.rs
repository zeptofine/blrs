use crate::info::{BasicBuildInfo, BlendBuild};

use super::query::{OrdPlacement, VersionSearchQuery, WildPlacement};

pub struct BInfoMatcher<'a> {
    versions: &'a [BasicBuildInfo],
}

impl<'a> BInfoMatcher<'a> {
    pub fn new(versions: &'a [BasicBuildInfo]) -> Self {
        BInfoMatcher { versions }
    }

    pub fn find_all(&self, query: &VersionSearchQuery) -> Vec<&BasicBuildInfo> {
        let vs = self
            .versions
            .iter()
            .filter(|build| match query.build_hash.clone() {
                WildPlacement::Any => true,
                WildPlacement::Exact(hash) => build.ver.build_hash() == hash,
            })
            .filter(|build| match query.branch.clone() {
                WildPlacement::Any => true,
                WildPlacement::Exact(branch) => build.ver.branch() == branch,
            })
            .collect::<Vec<&BasicBuildInfo>>();

        let vs = match query.major {
            OrdPlacement::Any => vs,
            _ => query.major.find(
                &(vs.iter().map(|v| &v.ver.v.major).collect::<Vec<_>>()),
                |idx| vs[idx],
            ),
        };
        let vs = match query.minor {
            OrdPlacement::Any => vs,
            _ => query.minor.find(
                &(vs.iter().map(|v| &v.ver.v.minor).collect::<Vec<_>>()),
                |idx| vs[idx],
            ),
        };
        let vs = match query.patch {
            OrdPlacement::Any => vs,
            _ => query.patch.find(
                &(vs.iter().map(|v| &v.ver.v.patch).collect::<Vec<_>>()),
                |idx| vs[idx],
            ),
        };

        let vs = match query.commit_dt {
            OrdPlacement::Any => vs,
            _ => query.commit_dt.find(
                &(vs.iter().map(|v| &v.commit_dt).collect::<Vec<_>>()),
                |idx| vs[idx],
            ),
        };

        vs
    }
}
