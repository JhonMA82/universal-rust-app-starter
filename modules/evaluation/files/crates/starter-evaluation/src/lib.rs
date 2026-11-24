#![forbid(unsafe_code)]
pub use petgraph::stable_graph::NodeIndex;
use petgraph::{
    Direction,
    algo::{has_path_connecting, toposort},
    stable_graph::StableDiGraph,
    visit::Dfs,
};
use std::collections::BTreeMap;

#[derive(Debug, PartialEq)]
pub enum Error<E> {
    Missing,
    Cycle,
    Cancelled,
    Compute(NodeIndex, E),
}
pub struct Graph<K, V> {
    graph: StableDiGraph<K, ()>,
    cache: BTreeMap<NodeIndex, V>,
}
impl<K, V> Default for Graph<K, V> {
    fn default() -> Self {
        Self {
            graph: StableDiGraph::new(),
            cache: BTreeMap::new(),
        }
    }
}
impl<K, V> Graph<K, V> {
    pub fn add(&mut self, key: K) -> NodeIndex {
        self.graph.add_node(key)
    }
    pub fn get(&self, id: NodeIndex) -> Option<&V> {
        self.cache.get(&id)
    }
    pub fn depend(&mut self, source: NodeIndex, dest: NodeIndex) -> Result<(), Error<()>> {
        if !self.graph.contains_node(source) || !self.graph.contains_node(dest) {
            return Err(Error::Missing);
        }
        if has_path_connecting(&self.graph, dest, source, None) {
            return Err(Error::Cycle);
        }
        if self.graph.find_edge(source, dest).is_none() {
            self.graph.add_edge(source, dest, ());
            self.invalidate(dest)?;
        }
        Ok(())
    }
    pub fn invalidate(&mut self, id: NodeIndex) -> Result<(), Error<()>> {
        if !self.graph.contains_node(id) {
            return Err(Error::Missing);
        }
        let mut dfs = Dfs::new(&self.graph, id);
        while let Some(next) = dfs.next(&self.graph) {
            self.cache.remove(&next);
        }
        Ok(())
    }
    pub fn evaluate<E>(
        &mut self,
        cancelled: impl Fn() -> bool,
        compute: impl Fn(&K, &[(NodeIndex, &V)]) -> Result<V, E>,
    ) -> Result<usize, Error<E>> {
        let mut pending = BTreeMap::new();
        for id in toposort(&self.graph, None).map_err(|_| Error::Cycle)? {
            if cancelled() {
                return Err(Error::Cancelled);
            }
            if self.cache.contains_key(&id) {
                continue;
            }
            let mut inputs: Vec<_> = self
                .graph
                .neighbors_directed(id, Direction::Incoming)
                .map(|input| {
                    (
                        input,
                        pending
                            .get(&input)
                            .or_else(|| self.cache.get(&input))
                            .expect("ordered dependency"),
                    )
                })
                .collect();
            inputs.sort_by_key(|(node, _)| *node);
            pending.insert(
                id,
                compute(&self.graph[id], &inputs).map_err(|e| Error::Compute(id, e))?,
            );
        }
        if cancelled() {
            return Err(Error::Cancelled);
        }
        let count = pending.len();
        self.cache.append(&mut pending);
        Ok(count)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cycles_and_recomputation() {
        let mut g = Graph::default();
        let a = g.add(2);
        let b = g.add(3);
        let c = g.add(0);
        g.depend(a, c).unwrap();
        g.depend(b, c).unwrap();
        assert_eq!(g.depend(c, a), Err(Error::Cycle));
        let compute = |n: &i32, v: &[(NodeIndex, &i32)]| {
            Ok::<_, ()>(*n + v.iter().map(|(_, x)| **x).sum::<i32>())
        };
        assert_eq!(g.evaluate(|| false, compute), Ok(3));
        assert_eq!(g.get(c), Some(&5));
        g.invalidate(a).unwrap();
        assert_eq!(g.evaluate(|| true, compute), Err(Error::Cancelled));
        assert_eq!(g.evaluate(|| false, compute), Ok(2));
    }
}
