//! Symbol Table Module
//! 
//! Provides symbol table management with scope, aliases, and macro support.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SymbolTableError {
    #[error("Symbol not found: {0}")]
    NotFound(String),
    #[error("Symbol already exists: {0}")]
    AlreadyExists(String),
    #[error("Scope error: {0}")]
    ScopeError(String),
}

/// Symbol entry in the table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolEntry {
    pub name: String,
    pub symbol_type: SymbolType,
    pub value: Option<String>,
    pub attributes: HashMap<String, String>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolType {
    Variable,
    Function,
    Constant,
    Macro,
    Alias,
    Type,
    Operator,
}

/// Scope level in the symbol table
#[derive(Debug, Clone)]
struct Scope {
    symbols: HashMap<String, SymbolEntry>,
    parent: Option<usize>,
}

impl Scope {
    fn new(parent: Option<usize>) -> Self {
        Self {
            symbols: HashMap::new(),
            parent,
        }
    }
    
    fn insert(&mut self, name: String, entry: SymbolEntry) -> Result<(), SymbolTableError> {
        if self.symbols.contains_key(&name) {
            return Err(SymbolTableError::AlreadyExists(name));
        }
        self.symbols.insert(name, entry);
        Ok(())
    }
    
    fn lookup(&self, name: &str) -> Option<&SymbolEntry> {
        self.symbols.get(name)
    }
    
    fn lookup_mut(&mut self, name: &str) -> Option<&mut SymbolEntry> {
        self.symbols.get_mut(name)
    }
}

/// Symbol table with hierarchical scoping
pub struct SymbolTable {
    scopes: Vec<Scope>,
    current_scope: usize,
    aliases: HashMap<String, String>,
    macros: HashMap<String, MacroDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroDefinition {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: String,
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut table = Self {
            scopes: Vec::new(),
            current_scope: 0,
            aliases: HashMap::new(),
            macros: HashMap::new(),
        };
        
        // Create global scope
        table.scopes.push(Scope::new(None));
        
        // Add built-in constants
        table.add_builtin_constants();
        
        table
    }
    
    fn add_builtin_constants(&mut self) {
        // Add built-in constants to global scope
        let builtins = vec![
            ("pi", "3.141592653589793", SymbolType::Constant),
            ("e", "2.718281828459045", SymbolType::Constant),
            ("i", "sqrt(-1)", SymbolType::Constant),
            ("true", "1", SymbolType::Constant),
            ("false", "0", SymbolType::Constant),
        ];
        
        for (name, value, sym_type) in builtins {
            let entry = SymbolEntry {
                name: name.to_string(),
                symbol_type: sym_type,
                value: Some(value.to_string()),
                attributes: HashMap::new(),
                line: 0,
                column: 0,
            };
            
            if let Some(scope) = self.scopes.get_mut(0) {
                let _ = scope.insert(name.to_string(), entry);
            }
        }
    }
    
    /// Enter a new scope
    pub fn enter_scope(&mut self) {
        let parent = Some(self.current_scope);
        self.scopes.push(Scope::new(parent));
        self.current_scope = self.scopes.len() - 1;
    }
    
    /// Exit the current scope
    pub fn exit_scope(&mut self) {
        if self.current_scope > 0 {
            let parent = self.scopes[self.current_scope].parent;
            self.scopes.pop();
            self.current_scope = parent.unwrap_or(0);
        }
    }
    
    /// Define a symbol in the current scope
    pub fn define(&mut self, name: &str, sym_type: SymbolType, value: Option<String>) -> Result<(), SymbolTableError> {
        let entry = SymbolEntry {
            name: name.to_string(),
            symbol_type: sym_type,
            value,
            attributes: HashMap::new(),
            line: 0,
            column: 0,
        };
        
        self.scopes[self.current_scope].insert(name.to_string(), entry)
    }
    
    /// Lookup a symbol (searches from current scope outward)
    pub fn lookup(&self, name: &str) -> Result<&SymbolEntry, SymbolTableError> {
        // First check aliases
        if let Some(alias) = self.aliases.get(name) {
            return self.lookup(alias);
        }
        
        // Search scopes from current to global
        let mut scope_idx = Some(self.current_scope);
        
        while let Some(idx) = scope_idx {
            if let Some(entry) = self.scopes[idx].lookup(name) {
                return Ok(entry);
            }
            scope_idx = self.scopes[idx].parent;
        }
        
        Err(SymbolTableError::NotFound(name.to_string()))
    }
    
    /// Update a symbol's value
    pub fn update(&mut self, name: &str, value: String) -> Result<(), SymbolTableError> {
        // Search scopes from current to global
        let mut scope_idx = Some(self.current_scope);
        
        while let Some(idx) = scope_idx {
            if let Some(entry) = self.scopes[idx].lookup_mut(name) {
                entry.value = Some(value);
                return Ok(());
            }
            scope_idx = self.scopes[idx].parent;
        }
        
        Err(SymbolTableError::NotFound(name.to_string()))
    }
    
    /// Define an alias
    pub fn define_alias(&mut self, alias: &str, target: &str) -> Result<(), SymbolTableError> {
        // Verify target exists
        self.lookup(target)?;
        self.aliases.insert(alias.to_string(), target.to_string());
        Ok(())
    }
    
    /// Define a macro
    pub fn define_macro(&mut self, name: &str, params: Vec<String>, body: &str) -> Result<(), SymbolTableError> {
        let macro_def = MacroDefinition {
            name: name.to_string(),
            parameters: params,
            body: body.to_string(),
        };
        
        self.macros.insert(name.to_string(), macro_def);
        
        // Also define as a symbol
        self.define(name, SymbolType::Macro, Some(body.to_string()))
    }
    
    /// Expand a macro
    pub fn expand_macro(&self, name: &str, args: &[String]) -> Result<String, SymbolTableError> {
        let macro_def = self.macros.get(name)
            .ok_or_else(|| SymbolTableError::NotFound(name.to_string()))?;
        
        let mut body = macro_def.body.clone();
        
        // Simple macro expansion (replace $n with arguments)
        for (i, arg) in args.iter().enumerate() {
            let placeholder = format!("${}", i + 1);
            body = body.replace(&placeholder, arg);
        }
        
        Ok(body)
    }
    
    /// Get all symbols in current scope
    pub fn get_scope_symbols(&self) -> Vec<&SymbolEntry> {
        self.scopes[self.current_scope].symbols.values().collect()
    }
    
    /// Get the current scope depth
    pub fn scope_depth(&self) -> usize {
        self.current_scope
    }
    
    /// Check if a symbol exists in any scope
    pub fn exists(&self, name: &str) -> bool {
        self.lookup(name).is_ok()
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_operations() {
        let mut table = SymbolTable::new();
        
        // Should have built-in constants
        assert!(table.lookup("pi").is_ok());
        assert!(table.lookup("e").is_ok());
        
        // Define new symbol
        table.define("x", SymbolType::Variable, Some("10".to_string())).unwrap();
        assert!(table.lookup("x").is_ok());
        
        // Update symbol
        table.update("x", "20".to_string()).unwrap();
        let entry = table.lookup("x").unwrap();
        assert_eq!(entry.value, Some("20".to_string()));
    }
    
    #[test]
    fn test_scope_isolation() {
        let mut table = SymbolTable::new();
        
        // Define in global scope
        table.define("global_var", SymbolType::Variable, Some("1".to_string())).unwrap();
        
        // Enter new scope
        table.enter_scope();
        
        // Define in inner scope
        table.define("local_var", SymbolType::Variable, Some("2".to_string())).unwrap();
        
        // Both should be visible
        assert!(table.lookup("global_var").is_ok());
        assert!(table.lookup("local_var").is_ok());
        
        // Exit scope
        table.exit_scope();
        
        // local_var should not be visible
        assert!(table.lookup("local_var").is_err());
        
        // global_var should still be visible
        assert!(table.lookup("global_var").is_ok());
    }
    
    #[test]
    fn test_aliases() {
        let mut table = SymbolTable::new();
        
        // Define an alias
        table.define_alias("my_pi", "pi").unwrap();
        
        // Alias should resolve to target
        let entry = table.lookup("my_pi").unwrap();
        let target = table.lookup("pi").unwrap();
        
        // Both should have the same value
        assert_eq!(entry.value, target.value);
    }
    
    #[test]
    fn test_macros() {
        let mut table = SymbolTable::new();
        
        // Define a macro: add($1, $2) = $1 + $2
        table.define_macro("add", vec!["x".to_string(), "y".to_string()], "$1 + $2").unwrap();
        
        // Expand macro
        let result = table.expand_macro("add", &["1".to_string(), "2".to_string()]).unwrap();
        assert_eq!(result, "1 + 2");
    }
    
    #[test]
    fn test_scope_depth() {
        let mut table = SymbolTable::new();
        
        assert_eq!(table.scope_depth(), 0);
        
        table.enter_scope();
        assert_eq!(table.scope_depth(), 1);
        
        table.enter_scope();
        assert_eq!(table.scope_depth(), 2);
        
        table.exit_scope();
        assert_eq!(table.scope_depth(), 1);
        
        table.exit_scope();
        assert_eq!(table.scope_depth(), 0);
    }
}
