//! Parseur minimal du format VDF de Valve (KeyValues).
//! Miroir de la logique de scripts/platforms/steam.js, réécrit en Rust
//! sans dépendance externe — juste ce dont Steam a besoin.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Value {
    Str(String),
    Map(HashMap<String, Value>),
}

impl Value {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }
    pub fn as_map(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Value::Map(m) => Some(m),
            _ => None,
        }
    }
}

fn read_quoted(chars: &[char], i: &mut usize) -> String {
    // Précondition : chars[*i] == '"'
    *i += 1;
    let mut s = String::new();
    while let Some(&c) = chars.get(*i) {
        if c == '\\' {
            if let Some(&next) = chars.get(*i + 1) {
                s.push(next);
                *i += 2;
                continue;
            }
        }
        if c == '"' {
            *i += 1;
            break;
        }
        s.push(c);
        *i += 1;
    }
    s
}

fn skip_ws(chars: &[char], i: &mut usize) {
    while matches!(chars.get(*i), Some(c) if c.is_whitespace()) {
        *i += 1;
    }
}

/// Récursif : un manifeste corrompu ne doit jamais faire planter le scan entier,
/// juste être ignoré au niveau où il casse.
fn parse_block(chars: &[char], i: &mut usize) -> HashMap<String, Value> {
    let mut map = HashMap::new();
    loop {
        skip_ws(chars, i);
        match chars.get(*i) {
            None => break,
            Some('}') => {
                *i += 1;
                break;
            }
            Some('"') => {
                let key = read_quoted(chars, i);
                skip_ws(chars, i);
                match chars.get(*i) {
                    Some('"') => {
                        let val = read_quoted(chars, i);
                        map.insert(key, Value::Str(val));
                    }
                    Some('{') => {
                        *i += 1;
                        let child = parse_block(chars, i);
                        map.insert(key, Value::Map(child));
                    }
                    _ => {} // clé isolée sans valeur : ignorée proprement
                }
            }
            _ => {
                *i += 1;
            } // caractère inattendu : on avance sans planter
        }
    }
    map
}

pub fn parse(text: &str) -> HashMap<String, Value> {
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0usize;
    parse_block(&chars, &mut i)
}
