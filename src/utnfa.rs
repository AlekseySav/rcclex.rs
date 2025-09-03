use crate::Charset;

/// Represents uncooked tagged nondetermitistic automata
#[derive(Clone)]
pub struct UTnfa {
    nodes: usize,
    begin: usize,
    end: usize,
    edges: Vec<(usize, usize, Charset)>,
    eps_edges: Vec<(usize, usize, isize)>,
}

impl UTnfa {
    /// Creates UTnfa to match empty string
    pub fn empty() -> Self {
        UTnfa {
            nodes: 1,
            begin: 0,
            end: 0,
            edges: Vec::new(),
            eps_edges: Vec::new(),
        }
    }

    /// Creates UTnfa to match single char from charset `c`
    pub fn charset(c: Charset) -> Self {
        UTnfa {
            nodes: 2,
            begin: 0,
            end: 1,
            edges: vec![(0, 1, c)],
            eps_edges: Vec::new(),
        }
    }

    /// Creates UTnfa to match tag `tag`
    pub fn tag(tag: isize) -> Self {
        UTnfa {
            nodes: 2,
            begin: 0,
            end: 1,
            edges: Vec::new(),
            eps_edges: vec![(0, 1, tag)],
        }
    }

    /// Concatenates `self` with `nfa`
    pub fn concat(&mut self, nfa: &UTnfa) {
        self.merge(nfa);
        self.eps_edges.push((self.end, nfa.begin, -1));
        self.end = nfa.end;
    }

    /// Unions `self` with `nfa`, i.e. applies `|` operator
    pub fn union(&mut self, nfa: &UTnfa) {
        self.merge(nfa);
        self.prepend_node();
        self.eps_edges.push((self.begin, nfa.begin, -1));
        self.append_node();
        self.eps_edges.push((nfa.end, self.end, -1));
    }

    /// Applies kleene start to `self`, i.e. applies `*` operator
    pub fn kleene(&mut self) {
        self.prepend_node();
        self.append_node();
        self.eps_edges.push((self.end, self.begin, -1));
        self.end = self.begin;
    }

    /// Makes `self` optional, i.e. applies `?` operator
    pub fn optional(&mut self) {
        self.union(&Self::empty())
    }

    /// Increases all node indices by `n`
    fn shift(&mut self, n: usize) {
        self.begin += n;
        self.end += n;
        for e in self.edges.iter_mut() {
            *e = (e.0 + n, e.1 + n, e.2);
        }
        for e in self.eps_edges.iter_mut() {
            *e = (e.0 + n, e.1 + n, e.2);
        }
    }

    /// Merges `self` with `nfa` by shifting `self` and appending all `nfa` edges
    fn merge(&mut self, nfa: &UTnfa) {
        self.shift(nfa.nodes);
        self.nodes += nfa.nodes;
        self.edges.extend(nfa.edges.iter());
        self.eps_edges.extend(nfa.eps_edges.iter());
    }

    /// Creates a new node, that preceedes `self.begin`, and assignes it to `self.begin`
    fn prepend_node(&mut self) {
        self.eps_edges.push((self.nodes, self.begin, -1));
        self.begin = self.nodes;
        self.nodes += 1
    }

    /// Creates a new node, that follows `self.end`, and assignes it to `self.end`
    fn append_node(&mut self) {
        self.eps_edges.push((self.end, self.nodes, -1));
        self.end = self.nodes;
        self.nodes += 1
    }
}
