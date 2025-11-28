//! Bencode parser and encoder for BitTorrent
//! 
//! Bencode format:
//! - Integers: i<number>e (e.g., i42e)
//! - Strings: <length>:<string> (e.g., 4:spam)
//! - Lists: l<items>e (e.g., l4:spami42ee)
//! - Dictionaries: d<key><value>...e (keys must be strings, sorted)

use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum BencodeValue {
    Integer(i64),
    String(Vec<u8>),
    List(Vec<BencodeValue>),
    Dict(BTreeMap<Vec<u8>, BencodeValue>),
}

#[derive(Debug, Error)]
pub enum BencodeError {
    #[error("Unexpected end of input")]
    UnexpectedEof,
    #[error("Invalid integer format")]
    InvalidInteger,
    #[error("Invalid string length")]
    InvalidStringLength,
    #[error("Invalid format at position {0}")]
    InvalidFormat(usize),
    #[error("Expected dictionary key to be string")]
    ExpectedStringKey,
}

impl BencodeValue {
    /// Parse bencode from bytes
    pub fn parse(data: &[u8]) -> Result<(Self, usize), BencodeError> {
        if data.is_empty() {
            return Err(BencodeError::UnexpectedEof);
        }

        match data[0] {
            b'i' => Self::parse_integer(data),
            b'l' => Self::parse_list(data),
            b'd' => Self::parse_dict(data),
            b'0'..=b'9' => Self::parse_string(data),
            _ => Err(BencodeError::InvalidFormat(0)),
        }
    }

    fn parse_integer(data: &[u8]) -> Result<(Self, usize), BencodeError> {
        let end = data.iter().position(|&b| b == b'e')
            .ok_or(BencodeError::UnexpectedEof)?;
        
        let num_str = std::str::from_utf8(&data[1..end])
            .map_err(|_| BencodeError::InvalidInteger)?;
        
        let num: i64 = num_str.parse()
            .map_err(|_| BencodeError::InvalidInteger)?;
        
        Ok((BencodeValue::Integer(num), end + 1))
    }

    fn parse_string(data: &[u8]) -> Result<(Self, usize), BencodeError> {
        let colon = data.iter().position(|&b| b == b':')
            .ok_or(BencodeError::InvalidStringLength)?;
        
        let len_str = std::str::from_utf8(&data[..colon])
            .map_err(|_| BencodeError::InvalidStringLength)?;
        
        let len: usize = len_str.parse()
            .map_err(|_| BencodeError::InvalidStringLength)?;
        
        let start = colon + 1;
        let end = start + len;
        
        if end > data.len() {
            return Err(BencodeError::UnexpectedEof);
        }
        
        Ok((BencodeValue::String(data[start..end].to_vec()), end))
    }

    fn parse_list(data: &[u8]) -> Result<(Self, usize), BencodeError> {
        let mut items = Vec::new();
        let mut pos = 1; // Skip 'l'
        
        while pos < data.len() && data[pos] != b'e' {
            let (value, consumed) = Self::parse(&data[pos..])?;
            items.push(value);
            pos += consumed;
        }
        
        if pos >= data.len() {
            return Err(BencodeError::UnexpectedEof);
        }
        
        Ok((BencodeValue::List(items), pos + 1)) // +1 for 'e'
    }

    fn parse_dict(data: &[u8]) -> Result<(Self, usize), BencodeError> {
        let mut dict = BTreeMap::new();
        let mut pos = 1; // Skip 'd'
        
        while pos < data.len() && data[pos] != b'e' {
            // Parse key (must be string)
            let (key, key_consumed) = Self::parse(&data[pos..])?;
            let key = match key {
                BencodeValue::String(s) => s,
                _ => return Err(BencodeError::ExpectedStringKey),
            };
            pos += key_consumed;
            
            // Parse value
            let (value, value_consumed) = Self::parse(&data[pos..])?;
            pos += value_consumed;
            
            dict.insert(key, value);
        }
        
        if pos >= data.len() {
            return Err(BencodeError::UnexpectedEof);
        }
        
        Ok((BencodeValue::Dict(dict), pos + 1)) // +1 for 'e'
    }

    /// Encode to bencode bytes
    pub fn encode(&self) -> Vec<u8> {
        match self {
            BencodeValue::Integer(n) => format!("i{}e", n).into_bytes(),
            BencodeValue::String(s) => {
                let mut result = format!("{}:", s.len()).into_bytes();
                result.extend(s);
                result
            }
            BencodeValue::List(items) => {
                let mut result = vec![b'l'];
                for item in items {
                    result.extend(item.encode());
                }
                result.push(b'e');
                result
            }
            BencodeValue::Dict(dict) => {
                let mut result = vec![b'd'];
                for (key, value) in dict {
                    result.extend(format!("{}:", key.len()).into_bytes());
                    result.extend(key);
                    result.extend(value.encode());
                }
                result.push(b'e');
                result
            }
        }
    }

    // Helper methods for accessing values
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            BencodeValue::Integer(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&[u8]> {
        match self {
            BencodeValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            BencodeValue::String(s) => std::str::from_utf8(s).ok(),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&[BencodeValue]> {
        match self {
            BencodeValue::List(l) => Some(l),
            _ => None,
        }
    }

    pub fn as_dict(&self) -> Option<&BTreeMap<Vec<u8>, BencodeValue>> {
        match self {
            BencodeValue::Dict(d) => Some(d),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&BencodeValue> {
        match self {
            BencodeValue::Dict(d) => d.get(key.as_bytes()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_integer() {
        let (val, _) = BencodeValue::parse(b"i42e").unwrap();
        assert_eq!(val.as_integer(), Some(42));
    }

    #[test]
    fn test_parse_string() {
        let (val, _) = BencodeValue::parse(b"4:spam").unwrap();
        assert_eq!(val.as_str(), Some("spam"));
    }

    #[test]
    fn test_parse_list() {
        let (val, _) = BencodeValue::parse(b"l4:spami42ee").unwrap();
        let list = val.as_list().unwrap();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_parse_dict() {
        let (val, _) = BencodeValue::parse(b"d3:bar4:spam3:fooi42ee").unwrap();
        assert_eq!(val.get("bar").and_then(|v| v.as_str()), Some("spam"));
        assert_eq!(val.get("foo").and_then(|v| v.as_integer()), Some(42));
    }
}

