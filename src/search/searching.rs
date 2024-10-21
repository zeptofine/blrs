use crate::{
    fetching::build_repository::BuildRepo,
    info::{BasicBuildInfo, BlendBuild},
};

use super::query::{OrdPlacement, VersionSearchQuery, WildPlacement};

pub struct BInfoMatcher<'a, 'b> {
    versions: &'a [(BasicBuildInfo, &'b BuildRepo)],
}

impl<'a, 'b> BInfoMatcher<'a, 'b> {
    pub fn new(versions: &'a [(BasicBuildInfo, &'b BuildRepo)]) -> Self {
        BInfoMatcher { versions }
    }

    pub fn find_all(&self, query: &VersionSearchQuery) -> Vec<&(BasicBuildInfo, &BuildRepo)> {
        let vs = self
            .versions
            .iter()
            .filter(|(_, repo)| match query.repository.clone() {
                WildPlacement::Any => true,
                WildPlacement::Exact(r) => repo.nickname == r,
            })
            .filter(|(build, _)| match query.build_hash.clone() {
                WildPlacement::Any => true,
                WildPlacement::Exact(hash) => build.ver.build_hash() == hash,
            })
            .filter(|(build, _)| match query.branch.clone() {
                WildPlacement::Any => true,
                WildPlacement::Exact(branch) => build.ver.branch() == branch,
            })
            .collect::<Vec<_>>();

        let vs = match query.major {
            OrdPlacement::Any => vs,
            _ => query.major.find(
                &(vs.iter().map(|(v, _)| &v.ver.v.major).collect::<Vec<_>>()),
                |idx| vs[idx],
            ),
        };
        let vs = match query.minor {
            OrdPlacement::Any => vs,
            _ => query.minor.find(
                &(vs.iter().map(|(v, _)| &v.ver.v.minor).collect::<Vec<_>>()),
                |idx| vs[idx],
            ),
        };
        let vs = match query.patch {
            OrdPlacement::Any => vs,
            _ => query.patch.find(
                &(vs.iter().map(|(v, _)| &v.ver.v.patch).collect::<Vec<_>>()),
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

        vs
    }
}
