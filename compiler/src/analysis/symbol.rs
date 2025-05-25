use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
pub struct ScopeManager {
    scope_array: Vec<SymbolTable>,
    scope_id: AtomicUsize,
    scope_current: Option<usize>,
}

impl ScopeManager {
    pub fn new() -> Self {
        ScopeManager {
            scope_array: Vec::new(),
            scope_id: AtomicUsize::new(0),
            scope_current: None,
        }
    }

    fn next_id(&mut self) -> usize {
        self.scope_id.fetch_add(1, Ordering::Relaxed)
    }

    // we need methods to:
    //  1. create a new scope / symbol table and transition into it
    //  2. close the current scope and rollback to the parent
    //  3. get the current scope
    //  4. get a given scope by id

    fn create(&mut self, parent_scope_id: Option<usize>, is_function: bool, return_type: Option<SymbolType>) -> usize
    {
        let scope_id = self.next_id();

        let new_scope = SymbolTable::new(
            scope_id,
            parent_scope_id,
            is_function,
            return_type,
        );

        self.scope_array.push(new_scope);

        scope_id
    }    

    pub fn activate(&mut self, scope_id: usize) -> Result <(), ()> {
        let scope = self.scope_array
            .iter()
            .find(|&s| s.scope_id == scope_id);

        self.scope_current = scope.map_or(None, |s| Some(s.scope_id));
        self.scope_current.map_or(Err(()), |_| Ok(()))
    }

    pub fn find_symbol_from_scope(&self, name: &str, scope_id: usize) -> Option<(usize, usize, &SymbolEntry)> {
        let mut distance = 0;
        let mut current_scope_id = Some(scope_id);

        while current_scope_id != None 
        {
            // Get scope with current scope id
            let scope = 
                self.scope_array
                    .iter()
                    .find(|&s| s.scope_id == current_scope_id.unwrap());
            
            // Lookup symbol in scope
            let symbol = scope.unwrap().get(name);

            // Found it, then return it
            if symbol != None {
                // println!("Name: {}", name);
                // println!("Distance: {}", distance);
                // println!("Start Scope ID: {:?}", scope_id.clone());
                // println!("Found Scope ID: {:?}", current_scope_id.unwrap());
                // println!("Scope: {:?}", scope.unwrap());

                return Some((current_scope_id.unwrap(), distance, symbol.unwrap()));
            }

            // Otherwise, go up a scope
            current_scope_id = scope.unwrap().parent_scope_id.clone();

            distance += 1;
        }

        None
    }

    pub fn find_symbol(&self, name: &str) -> Option<(usize, usize, &SymbolEntry)> {
        self.find_symbol_from_scope(name, self.scope_current.clone().unwrap())
    }

    pub fn open(&mut self, is_function: bool, return_type: Option<SymbolType>) -> Result<(), ()>
    {
        let parent_scope_id;

        if  self.is_empty() {
            parent_scope_id = None;
        } else {
            parent_scope_id = Some(self.current().unwrap().scope_id());
        }
        
        let scope_id = self.create(parent_scope_id, is_function, return_type);
        
        self.activate(scope_id)
    }

    pub fn close(&mut self) -> Result<(), ()> {
        let parent_scope_id = self.current().unwrap().parent_scope_id();
        
        parent_scope_id.map_or(Err(()), |id| self.activate(id))
    }
    
    // returns the current scope
    pub fn current(&self) -> Option<&SymbolTable> {
        self.scope_current.map(|id| self.scope_array.get(id).unwrap())
    }

    pub fn current_mut(&mut self) -> Option<&mut SymbolTable> {
        self.scope_current.map(|id| self.scope_array.get_mut(id).unwrap())
    }

    pub fn get(&mut self, scope_id: usize) -> Option<&SymbolTable> {
        self.scope_array.get(scope_id)
    }

    pub fn get_mut(&mut self, scope_id: usize) -> Option<&mut SymbolTable> {
        self.scope_array.get_mut(scope_id)
    }

    // check if there are any scopes available
    fn is_empty(&self) -> bool {
        self.scope_array.is_empty()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct SymbolTable {
    symbols: HashMap<String, SymbolEntry>,
    scope_id: usize,
    parent_scope_id: Option<usize>,
    is_function: bool,
    return_type: Option<SymbolType>,
}

impl SymbolTable {
    pub fn new(scope_id: usize, parent_scope_id: Option<usize>, is_function: bool, return_type: Option<SymbolType>) -> Self {
        SymbolTable {
            symbols: HashMap::new(),
            scope_id,
            parent_scope_id,
            is_function,
            return_type,
        }
    }

    pub fn insert(&mut self, name: String, entry: SymbolEntry) {
        let mut symbol_entry = entry.clone();
        symbol_entry.offset = Some(self.size());
        self.symbols.insert(name, symbol_entry);
    }

    pub fn exists(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }

    pub fn get(&self, name: &str) -> Option<&SymbolEntry> {
        self.symbols.get(name)
    }

    pub fn get_iter(&self) -> std::collections::hash_map::Iter<String, SymbolEntry> {
        self.symbols.iter()
    }

    pub fn size(&self) -> usize {
        self.symbols.len()
    }

    pub fn scope_id(&self) -> usize {
        self.scope_id
    }

    pub fn parent_scope_id(&self) -> Option<usize> {
        self.parent_scope_id
    }

    pub fn is_function(&self) -> bool {
        self.is_function
    }

    pub fn return_type(&self) -> Option<SymbolType> {
        self.return_type.clone()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum SymbolType
{
    Bool,
    Int,
    Float,
    Colour,
    Function,
    Undefined,
}

impl SymbolType
{
    pub fn from_string(s: &str) -> Option<SymbolType>
    {
        match s
        {
            "bool" => Some(SymbolType::Bool),
            "int" => Some(SymbolType::Int),
            "float" => Some(SymbolType::Float),
            "colour" => Some(SymbolType::Colour),
            "function" => Some(SymbolType::Function),
            _ => None
        }
    }

    pub fn to_string(&self) -> String
    {
        match self
        {
            SymbolType::Bool => String::from("bool"),
            SymbolType::Int => String::from("int"),
            SymbolType::Float => String::from("float"),
            SymbolType::Colour => String::from("colour"),
            SymbolType::Function => String::from("function"),
            SymbolType::Undefined => String::from("undefined")
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct SymbolEntry {
    pub name: String,
    pub symbol_type: SymbolType,
    pub params: Option<Vec<SymbolEntry>>,
    pub return_type: Option<SymbolType>,
    pub offset: Option<usize>,
}