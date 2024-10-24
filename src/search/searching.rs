use crate::info::{BasicBuildInfo, BlendBuild};

use super::query::{OrdPlacement, VersionSearchQuery, WildPlacement};

type RepoNickname = String;
pub struct BInfoMatcher<'a, BI>
where
    BI: AsRef<BasicBuildInfo>,
{
    versions: &'a [(BI, RepoNickname)],
}

impl<'a, BI> BInfoMatcher<'a, BI>
where
    BI: AsRef<BasicBuildInfo>,
{
    pub fn new(versions: &'a [(BI, RepoNickname)]) -> Self {
        BInfoMatcher { versions }
    }

    pub fn find_all(&self, query: &VersionSearchQuery) -> Vec<&(BI, RepoNickname)> {
        let vs = self
            .versions
            .into_iter()
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
                &(vs.iter().map(|(v, _)| &v.ver.v.major).collect::<Vec<_>>()),
                |idx| vs[idx].clone(),
            ),
        };
        let vs = match query.minor {
            OrdPlacement::Any => vs,
            _ => query.minor.find(
                &(vs.iter().map(|(v, _)| &v.ver.v.minor).collect::<Vec<_>>()),
                |idx| vs[idx].clone(),
            ),
        };
        let vs = match query.patch {
            OrdPlacement::Any => vs,
            _ => query.patch.find(
                &(vs.iter().map(|(v, _)| &v.ver.v.patch).collect::<Vec<_>>()),
                |idx| vs[idx].clone(),
            ),
        };

        let vs = match query.commit_dt {
            OrdPlacement::Any => vs,
            _ => query.commit_dt.find(
                &(vs.iter().map(|(v, _)| &v.commit_dt).collect::<Vec<_>>()),
                |idx| vs[idx].clone(),
            ),
        };

        vs.into_iter().map(|(_, x)| x).collect()
    }
}
