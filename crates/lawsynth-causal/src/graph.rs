use crate::{CausalError, Result};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Edge {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CausalGraph {
    variables: BTreeSet<String>,
    edges: BTreeSet<Edge>,
}

impl CausalGraph {
    pub fn new<I, S>(variables: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut graph = Self::default();
        for variable in variables {
            graph.add_variable(variable)?;
        }
        Ok(graph)
    }
    pub fn add_variable(&mut self, variable: impl Into<String>) -> Result<()> {
        let variable = variable.into();
        if variable.trim().is_empty() || !self.variables.insert(variable.clone()) {
            return Err(CausalError::DuplicateVariable(variable));
        }
        Ok(())
    }
    pub fn add_edge(&mut self, from: impl Into<String>, to: impl Into<String>) -> Result<()> {
        let (from, to) = (from.into(), to.into());
        if !self.variables.contains(&from) {
            return Err(CausalError::UnknownVariable(from));
        }
        if !self.variables.contains(&to) {
            return Err(CausalError::UnknownVariable(to));
        }
        if from == to {
            return Err(CausalError::SelfEdge(from));
        }
        if self.has_path(&to, &from) {
            return Err(CausalError::Cycle { from, to });
        }
        self.edges.insert(Edge { from, to });
        Ok(())
    }
    pub fn variables(&self) -> impl Iterator<Item = &str> {
        self.variables.iter().map(String::as_str)
    }
    pub fn edges(&self) -> impl Iterator<Item = &Edge> {
        self.edges.iter()
    }
    pub fn parents(&self, variable: &str) -> Result<Vec<&str>> {
        self.require(variable)?;
        Ok(self.edges.iter().filter(|e| e.to == variable).map(|e| e.from.as_str()).collect())
    }
    pub fn children(&self, variable: &str) -> Result<Vec<&str>> {
        self.require(variable)?;
        Ok(self.edges.iter().filter(|e| e.from == variable).map(|e| e.to.as_str()).collect())
    }
    pub fn topological_order(&self) -> Vec<&str> {
        let mut degree: BTreeMap<&str, usize> = self.variables().map(|v| (v, 0)).collect();
        for edge in &self.edges {
            *degree.get_mut(edge.to.as_str()).expect("registered endpoint") += 1;
        }
        let mut ready: BTreeSet<&str> =
            degree.iter().filter_map(|(v, d)| (*d == 0).then_some(*v)).collect();
        let mut out = Vec::with_capacity(degree.len());
        while let Some(v) = ready.pop_first() {
            out.push(v);
            for child in self.edges.iter().filter(|e| e.from == v).map(|e| e.to.as_str()) {
                let d = degree.get_mut(child).expect("registered endpoint");
                *d -= 1;
                if *d == 0 {
                    ready.insert(child);
                }
            }
        }
        out
    }
    pub fn has_edge(&self, from: &str, to: &str) -> bool {
        self.edges.contains(&Edge { from: from.into(), to: to.into() })
    }
    fn require(&self, variable: &str) -> Result<()> {
        if self.variables.contains(variable) {
            Ok(())
        } else {
            Err(CausalError::UnknownVariable(variable.into()))
        }
    }
    fn has_path(&self, start: &str, target: &str) -> bool {
        let mut queue = VecDeque::from([start]);
        let mut seen = BTreeSet::new();
        while let Some(v) = queue.pop_front() {
            if !seen.insert(v) {
                continue;
            }
            if v == target {
                return true;
            }
            for e in self.edges.iter().filter(|e| e.from == v) {
                queue.push_back(e.to.as_str());
            }
        }
        false
    }
}
