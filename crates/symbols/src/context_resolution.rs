//! Context Resolution Module
//!
//! Provides context-aware symbol resolution and disambiguation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Domain context for symbol resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Domain {
    Arithmetic,
    Algebra,
    Calculus,
    LinearAlgebra,
    SetTheory,
    Logic,
    Geometry,
    Statistics,
    Physics,
    Custom,
}

/// Symbol meaning in a specific domain
#[derive(Debug, Clone)]
pub struct DomainMeaning {
    pub symbol: char,
    pub domain: Domain,
    pub meaning: String,
    pub latex: Option<String>,
    pub description: String,
}

/// Context for resolving ambiguous symbols
pub struct ResolutionContext {
    domain: Domain,
    variables: HashMap<String, Domain>,
    scope_stack: Vec<HashMap<String, String>>,
}

impl ResolutionContext {
    pub fn new(domain: Domain) -> Self {
        Self {
            domain,
            variables: HashMap::new(),
            scope_stack: Vec::new(),
        }
    }

    pub fn domain(&self) -> Domain {
        self.domain
    }

    pub fn set_domain(&mut self, domain: Domain) {
        self.domain = domain;
    }

    /// Register a variable with its domain
    pub fn register_variable(&mut self, name: &str, domain: Domain) {
        self.variables.insert(name.to_string(), domain);
    }

    /// Get the domain of a variable
    pub fn get_variable_domain(&self, name: &str) -> Option<Domain> {
        self.variables.get(name).copied()
    }

    /// Enter a new scope
    pub fn enter_scope(&mut self) {
        self.scope_stack.push(HashMap::new());
    }

    /// Exit the current scope
    pub fn exit_scope(&mut self) {
        self.scope_stack.pop();
    }

    /// Bind a symbol in current scope
    pub fn bind(&mut self, name: &str, value: &str) {
        if let Some(scope) = self.scope_stack.last_mut() {
            scope.insert(name.to_string(), value.to_string());
        }
    }

    /// Resolve a name in current scope
    pub fn resolve(&self, name: &str) -> Option<String> {
        // Search from innermost to outermost scope
        for scope in self.scope_stack.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value.clone());
            }
        }
        None
    }
}

/// Disambiguation rules for symbols
pub struct DisambiguationRules {
    rules: HashMap<char, Vec<DomainMeaning>>,
}

impl DisambiguationRules {
    pub fn new() -> Self {
        let mut rules = Self {
            rules: HashMap::new(),
        };

        rules.init_builtin_rules();
        rules
    }

    fn init_builtin_rules(&mut self) {
        // Symbol 'P' - different meanings in different domains
        self.add_rule(
            'P',
            Domain::Logic,
            "Proposition",
            r"\P",
            "Propositional variable",
        );
        self.add_rule(
            'P',
            Domain::Calculus,
            "Probability",
            r"\mathbb{P}",
            "Probability function",
        );
        self.add_rule(
            'P',
            Domain::LinearAlgebra,
            "Projection",
            r"\operatorname{P}",
            "Projection matrix",
        );

        // Symbol 'S' - different meanings
        self.add_rule('S', Domain::Calculus, "Sum", r"\sum", "Summation");
        self.add_rule('S', Domain::SetTheory, "Set", "S", "Arbitrary set");
        self.add_rule(
            'S',
            Domain::Statistics,
            "Sample space",
            r"\mathcal{S}",
            "Sample space",
        );

        // Symbol 'D' - different meanings
        self.add_rule(
            'D',
            Domain::Calculus,
            "Differential",
            "d",
            "Differential operator",
        );
        self.add_rule('D', Domain::Algebra, "Division", "D", "Division ring");

        // Symbol 'I' - different meanings
        self.add_rule(
            'I',
            Domain::Calculus,
            "Integral",
            r"\int",
            "Integral symbol",
        );
        self.add_rule(
            'I',
            Domain::LinearAlgebra,
            "Identity",
            "I",
            "Identity matrix",
        );
        self.add_rule(
            'I',
            Domain::Logic,
            "Implication",
            r"\implies",
            "Logical implication",
        );

        // Symbol '∇' - Nabla/Del operator
        self.add_rule(
            '∇',
            Domain::Calculus,
            "Gradient",
            r"\nabla",
            "Gradient operator",
        );
        self.add_rule(
            '∇',
            Domain::Geometry,
            "Laplacian",
            r"\nabla^2",
            "Laplacian operator",
        );
        self.add_rule(
            '∇',
            Domain::LinearAlgebra,
            "Vector differential",
            r"\nabla",
            "Del operator",
        );
    }

    fn add_rule(&mut self, symbol: char, domain: Domain, meaning: &str, latex: &str, desc: &str) {
        let entry = DomainMeaning {
            symbol,
            domain,
            meaning: meaning.to_string(),
            latex: Some(latex.to_string()),
            description: desc.to_string(),
        };

        self.rules
            .entry(symbol)
            .or_insert_with(Vec::new)
            .push(entry);
    }

    /// Get meanings for a symbol in a specific domain
    pub fn get_meaning(&self, symbol: char, domain: Domain) -> Option<&DomainMeaning> {
        self.rules
            .get(&symbol)
            .and_then(|meanings| meanings.iter().find(|m| m.domain == domain))
    }

    /// Get all meanings for a symbol
    pub fn get_all_meanings(&self, symbol: char) -> Option<&Vec<DomainMeaning>> {
        self.rules.get(&symbol)
    }

    /// Resolve an ambiguous symbol using context
    pub fn resolve(&self, symbol: char, context: &ResolutionContext) -> Option<&DomainMeaning> {
        self.get_meaning(symbol, context.domain())
    }
}

impl Default for DisambiguationRules {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = ResolutionContext::new(Domain::Calculus);
        assert_eq!(ctx.domain(), Domain::Calculus);
    }

    #[test]
    fn test_variable_domain() {
        let mut ctx = ResolutionContext::new(Domain::Algebra);
        ctx.register_variable("x", Domain::Algebra);
        ctx.register_variable("matrix", Domain::LinearAlgebra);

        assert_eq!(ctx.get_variable_domain("x"), Some(Domain::Algebra));
        assert_eq!(
            ctx.get_variable_domain("matrix"),
            Some(Domain::LinearAlgebra)
        );
    }

    #[test]
    fn test_scope_resolution() {
        let mut ctx = ResolutionContext::new(Domain::Arithmetic);

        // Outer scope
        ctx.enter_scope();
        ctx.bind("a", "1");

        // Inner scope
        ctx.enter_scope();
        ctx.bind("a", "2");
        assert_eq!(ctx.resolve("a"), Some("2".to_string()));

        // Exit inner scope
        ctx.exit_scope();
        assert_eq!(ctx.resolve("a"), Some("1".to_string()));
    }

    #[test]
    fn test_disambiguation_rules() {
        let rules = DisambiguationRules::new();

        // Test P in Logic domain
        let meaning = rules.get_meaning('P', Domain::Logic);
        assert!(meaning.is_some());

        // Test P in Calculus domain
        let meaning = rules.get_meaning('P', Domain::Calculus);
        assert!(meaning.is_some());

        // Test ∇ in Calculus
        let meaning = rules.get_meaning('∇', Domain::Calculus);
        assert!(meaning.is_some());
    }

    #[test]
    fn test_context_resolution() {
        let rules = DisambiguationRules::new();
        let mut ctx = ResolutionContext::new(Domain::Calculus);

        let meaning = rules.resolve('∇', &ctx);
        assert!(meaning.is_some());

        // Change domain
        ctx.set_domain(Domain::Geometry);
        let meaning = rules.resolve('∇', &ctx);
        assert!(meaning.is_some());
    }
}
