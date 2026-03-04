//! Unicode Math Symbols Module
//! 
//! Provides comprehensive Unicode math symbol support including:
//! - Mathematical Alphanumeric Symbols (U+1D400–U+1D7FF)
//! - Greek and Coptic letters
//! - Hebrew letters
//! - Mathematical Operators

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unicode block ranges for math symbols
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnicodeBlock {
    /// Mathematical Alphanumeric Symbols (U+1D400–U+1D7FF)
    MathAlphanumeric,
    /// Greek and Coptic (U+0370–U+03FF)
    GreekCoptic,
    /// Hebrew (U+0590–U+05FF)
    Hebrew,
    /// Mathematical Operators (U+2200–U+22FF)
    MathOperators,
    /// Miscellaneous Mathematical Symbols (U+27C0–U+27EF)
    MiscMathSymbols,
    /// Supplemental Mathematical Operators (U+2A00–U+2AFF)
    SupplementalMathOps,
}

impl UnicodeBlock {
    /// Check if a codepoint belongs to this block
    pub fn contains(self, cp: u32) -> bool {
        match self {
            UnicodeBlock::MathAlphanumeric => (0x1D400..=0x1D7FF).contains(&cp),
            UnicodeBlock::GreekCoptic => (0x0370..=0x03FF).contains(&cp),
            UnicodeBlock::Hebrew => (0x0590..=0x05FF).contains(&cp),
            UnicodeBlock::MathOperators => (0x2200..=0x22FF).contains(&cp),
            UnicodeBlock::MiscMathSymbols => (0x27C0..=0x27EF).contains(&cp),
            UnicodeBlock::SupplementalMathOps => (0x2A00..=0x2AFF).contains(&cp),
        }
    }
}

/// Category of math symbol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolCategory {
    Letter,
    Number,
    Operator,
    Relation,
    Arrow,
    Delimiter,
    Misc,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MathSymbol {
    pub codepoint: u32,
    pub character: char,
    pub name: String,
    pub category: SymbolCategory,
    pub block: UnicodeBlock,
    pub ascii_equivalent: Option<String>,
    pub latex_equivalent: Option<String>,
}

impl MathSymbol {
    pub fn new(codepoint: u32, name: &str, category: SymbolCategory) -> Option<Self> {
        let ch = char::from_u32(codepoint)?;
        Some(Self {
            codepoint,
            character: ch,
            name: name.to_string(),
            category,
            block: Self::detect_block(codepoint),
            ascii_equivalent: None,
            latex_equivalent: None,
        })
    }
    
    fn detect_block(cp: u32) -> UnicodeBlock {
        if UnicodeBlock::MathAlphanumeric.contains(cp) {
            UnicodeBlock::MathAlphanumeric
        } else if UnicodeBlock::GreekCoptic.contains(cp) {
            UnicodeBlock::GreekCoptic
        } else if UnicodeBlock::Hebrew.contains(cp) {
            UnicodeBlock::Hebrew
        } else if UnicodeBlock::MathOperators.contains(cp) {
            UnicodeBlock::MathOperators
        } else if UnicodeBlock::MiscMathSymbols.contains(cp) {
            UnicodeBlock::MiscMathSymbols
        } else if UnicodeBlock::SupplementalMathOps.contains(cp) {
            UnicodeBlock::SupplementalMathOps
        } else {
            UnicodeBlock::MathAlphanumeric
        }
    }
}

/// Unicode math symbols registry
pub struct SymbolRegistry {
    by_codepoint: HashMap<u32, MathSymbol>,
    by_name: HashMap<String, MathSymbol>,
    by_latex: HashMap<String, MathSymbol>,
}

impl SymbolRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            by_codepoint: HashMap::new(),
            by_name: HashMap::new(),
            by_latex: HashMap::new(),
        };
        
        registry.init_builtin_symbols();
        registry
    }
    
    fn init_builtin_symbols(&mut self) {
        // Greek letters (sample)
        let greek = vec![
            (0x03B1, "alpha", "α"),
            (0x03B2, "beta", "β"),
            (0x03B3, "gamma", "γ"),
            (0x03B4, "delta", "δ"),
            (0x03B5, "epsilon", "ε"),
            (0x03B6, "zeta", "ζ"),
            (0x03B7, "eta", "η"),
            (0x03B8, "theta", "θ"),
            (0x03B9, "iota", "ι"),
            (0x03BA, "kappa", "κ"),
            (0x03BB, "lambda", "λ"),
            (0x03BC, "mu", "μ"),
            (0x03BD, "nu", "ν"),
            (0x03BE, "xi", "ξ"),
            (0x03BF, "omicron", "ο"),
            (0x03C0, "pi", "π"),
            (0x03C1, "rho", "ρ"),
            (0x03C3, "sigma", "σ"),
            (0x03C4, "tau", "τ"),
            (0x03C5, "upsilon", "υ"),
            (0x03C6, "phi", "φ"),
            (0x03C7, "chi", "χ"),
            (0x03C8, "psi", "ψ"),
            (0x03C9, "omega", "ω"),
            // Uppercase Greek
            (0x0391, "Alpha", "Α"),
            (0x0392, "Beta", "Β"),
            (0x0393, "Gamma", "Γ"),
            (0x0394, "Delta", "Δ"),
            (0x0395, "Epsilon", "Ε"),
            (0x0396, "Zeta", "Ζ"),
            (0x0397, "Eta", "Η"),
            (0x0398, "Theta", "Θ"),
            (0x0399, "Iota", "Ι"),
            (0x039A, "Kappa", "Κ"),
            (0x039B, "Lambda", "Λ"),
            (0x039C, "Mu", "Μ"),
            (0x039D, "Nu", "Ν"),
            (0x039E, "Xi", "Ξ"),
            (0x039F, "Omicron", "Ο"),
            (0x03A0, "Pi", "Π"),
            (0x03A1, "Rho", "Ρ"),
            (0x03A3, "Sigma", "Σ"),
            (0x03A4, "Tau", "Τ"),
            (0x03A5, "Upsilon", "Υ"),
            (0x03A6, "Phi", "Φ"),
            (0x03A7, "Chi", "Χ"),
            (0x03A8, "Psi", "Ψ"),
            (0x03A9, "Omega", "Ω"),
        ];
        
        for (cp, name, _ch) in greek {
            if let Some(mut sym) = MathSymbol::new(cp, name, SymbolCategory::Letter) {
                sym.latex_equivalent = Some(format!("\\{}", name));
                self.register(sym);
            }
        }
        
        // Mathematical operators (sample)
        let operators = vec![
            (0x2200, "forall", "∀"),
            (0x2202, "partial", "∂"),
            (0x2203, "exists", "∃"),
            (0x2205, "emptyset", "∅"),
            (0x2207, "nabla", "∇"),
            (0x2208, "in", "∈"),
            (0x2209, "notin", "∉"),
            (0x220B, "ni", "∋"),
            (0x220F, "prod", "∏"),
            (0x2211, "sum", "∑"),
            (0x2212, "minus", "−"),
            (0x2215, "setminus", "∖"),
            (0x2217, "ast", "∗"),
            (0x221A, "sqrt", "√"),
            (0x221E, "infty", "∞"),
            (0x2220, "angle", "∠"),
            (0x2227, "land", "∧"),
            (0x2228, "lor", "∨"),
            (0x2229, "cap", "∩"),
            (0x222A, "cup", "∪"),
            (0x222B, "int", "∫"),
            (0x2248, "approx", "≈"),
            (0x2260, "neq", "≠"),
            (0x2261, "equiv", "≡"),
            (0x2264, "leq", "≤"),
            (0x2265, "geq", "≥"),
            (0x2282, "subset", "⊂"),
            (0x2283, "supset", "⊃"),
            (0x2284, "nsubset", "⊄"),
            (0x2286, "subseteq", "⊆"),
            (0x2287, "supseteq", "⊇"),
            (0x22A5, "perp", "⊥"),
            (0x22C0, "land", "⋀"),
            (0x22C1, "lor", "⋁"),
        ];
        
        for (cp, name, ch) in operators {
            if let Some(mut sym) = MathSymbol::new(cp, name, SymbolCategory::Operator) {
                sym.latex_equivalent = Some(format!("\\{}", name));
                self.register(sym);
            }
        }
        
        // Arrows
        let arrows = vec![
            (0x2190, "leftarrow", "←"),
            (0x2191, "uparrow", "↑"),
            (0x2192, "rightarrow", "→"),
            (0x2193, "downarrow", "↓"),
            (0x2194, "leftrightarrow", "↔"),
            (0x2195, "updownarrow", "↕"),
            (0x21D0, "Leftarrow", "⇐"),
            (0x21D1, "Uparrow", "⇑"),
            (0x21D2, "Rightarrow", "⇒"),
            (0x21D3, "Downarrow", "⇓"),
            (0x21D4, "Leftrightarrow", "⇔"),
            (0x21D5, "Updownarrow", "⇕"),
        ];
        
        for (cp, name, _ch) in arrows {
            if let Some(mut sym) = MathSymbol::new(cp, name, SymbolCategory::Arrow) {
                sym.latex_equivalent = Some(format!("\\{}", name));
                self.register(sym);
            }
        }
    }
    
    fn register(&mut self, symbol: MathSymbol) {
        let cp = symbol.codepoint;
        let name = symbol.name.clone();
        
        self.by_codepoint.insert(cp, symbol.clone());
        self.by_name.insert(name, symbol.clone());
        
        if let Some(ref latex) = symbol.latex_equivalent {
            self.by_latex.insert(latex.trim_start_matches('\\').to_string(), symbol);
        }
    }
    
    /// Lookup by codepoint
    pub fn get_by_codepoint(&self, cp: u32) -> Option<&MathSymbol> {
        self.by_codepoint.get(&cp)
    }
    
    /// Lookup by name
    pub fn get_by_name(&self, name: &str) -> Option<&MathSymbol> {
        self.by_name.get(name)
    }
    
    /// Lookup by LaTeX command
    pub fn get_by_latex(&self, latex: &str) -> Option<&MathSymbol> {
        let key = latex.trim_start_matches('\\');
        self.by_latex.get(key)
    }
    
    /// Get all symbols in a category
    pub fn get_by_category(&self, category: SymbolCategory) -> Vec<&MathSymbol> {
        self.by_codepoint
            .values()
            .filter(|s| s.category == category)
            .collect()
    }
    
    /// Get all symbols in a block
    pub fn get_by_block(&self, block: UnicodeBlock) -> Vec<&MathSymbol> {
        self.by_codepoint
            .values()
            .filter(|s| s.block == block)
            .collect()
    }
}

impl Default for SymbolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a Unicode math symbol to ASCII equivalent
pub fn to_ascii(s: &str) -> Option<String> {
    let registry = SymbolRegistry::new();
    
    for ch in s.chars() {
        let cp = ch as u32;
        if let Some(symbol) = registry.get_by_codepoint(cp) {
            if let Some(ref ascii) = symbol.ascii_equivalent {
                return Some(ascii.clone());
            }
        }
    }
    
    None
}

/// Convert a Unicode math symbol to LaTeX
pub fn to_latex(s: &str) -> Option<String> {
    let registry = SymbolRegistry::new();
    
    let mut result = String::new();
    for ch in s.chars() {
        let cp = ch as u32;
        if let Some(symbol) = registry.get_by_codepoint(cp) {
            if let Some(ref latex) = symbol.latex_equivalent {
                result.push_str(latex);
            } else {
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }
    
    if result.is_empty() { None } else { Some(result) }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_greek_letters() {
        let registry = SymbolRegistry::new();
        
        // Test lowercase Greek
        assert!(registry.get_by_name("alpha").is_some());
        assert!(registry.get_by_name("beta").is_some());
        assert!(registry.get_by_name("gamma").is_some());
        
        // Test uppercase Greek
        assert!(registry.get_by_name("Alpha").is_some());
        assert!(registry.get_by_name("Beta").is_some());
        
        // Test codepoint lookup
        assert!(registry.get_by_codepoint(0x03B1).is_some());
    }
    
    #[test]
    fn test_operators() {
        let registry = SymbolRegistry::new();
        
        assert!(registry.get_by_name("forall").is_some());
        assert!(registry.get_by_name("exists").is_some());
        assert!(registry.get_by_name("infty").is_some());
        assert!(registry.get_by_name("leq").is_some());
        assert!(registry.get_by_name("geq").is_some());
    }
    
    #[test]
    fn test_latex_conversion() {
        let latex = to_latex("α + β");
        assert!(latex.is_some());
        assert!(latex.unwrap().contains("\\alpha"));
    }
    
    #[test]
    fn test_category_filter() {
        let registry = SymbolRegistry::new();
        
        let operators = registry.get_by_category(SymbolCategory::Operator);
        assert!(!operators.is_empty());
        
        let letters = registry.get_by_category(SymbolCategory::Letter);
        assert!(!letters.is_empty());
    }
}
