use crate::ecs::{
    query::{fetch::WorldQuery, filter::FilterFetch, state::QueryState},
    World,
};

pub struct Query<'w, Q: WorldQuery, F: WorldQuery /* = ()*/>
where
    F::Fetch: FilterFetch,
{
    pub(crate) world: &'w World,
    pub(crate) state: &'w QueryState<Q, F>,
}
