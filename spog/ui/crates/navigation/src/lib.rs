use yew_nested_router::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, Target)]
pub enum AppRoute {
    #[target(index)]
    Index,
    NotLoggedIn,
    Chicken,
    Sbom(View),
    Advisory(View),
    Scanner,
    Search {
        terms: String,
    },
    Cve(View),
    Package {
        id: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Target)]
pub enum View {
    Search { query: String },
    Content { id: String },
}

impl Default for View {
    fn default() -> Self {
        Self::Search {
            query: Default::default(),
        }
    }
}
