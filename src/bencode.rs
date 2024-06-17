use std::str::{self, from_utf8};
use std::collections::BTreeMap;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ByteString {
    String(String),
    ByteString(Vec<u8>)
}

#[derive(Eq, PartialEq, PartialOrd, Ord, Debug)]
pub enum Value {
    Integer(i64),
    String(ByteString),
    List(Vec<Value>),
    Dict(BTreeMap<Value, Value>)
}

impl Value {
    fn to_integer(&self) -> Option<i64> {
        match self {
            Value::Integer(int) => Some(*int),
            _ => None
        }
    }

    fn to_string(&self) -> Option<String> {
        match self {
            Value::String(str_value) => {
                match str_value {
                    ByteString::String(str) => Some(str.clone()),
                    _ => None
                }
            }
            _ => None
        }
    }

    fn get_value(&self, key: &str) -> Option<&Value> {
        if let Value::Dict(dict) = self {
            dict.get(&Value::String(ByteString::String(key.to_owned())))
        } else {
            None
        }
    }

    pub fn get_integer(&self, key: &str) -> Option<i64> {
        match self.get_value(key) {
            Some(value) => value.to_integer(),
            None => None
        }
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        match self.get_value(key) {
            Some(value) => value.to_string(),
            None => None
        }
    }

    pub fn get_string_list(&self, key: &str) -> Option<Vec<String>> {
        match self.get_value(key) {
            Some(value) => {
                match value {
                    Value::List(list) => {
                        let mut strings: Vec<String> = Vec::with_capacity(list.len());
                        for value in list {
                            let string = match value.to_string() {
                                Some(string) => string,
                                None => break
                            };

                            strings.push(string);
                        }

                        Some(strings)
                    }
                    _ => None
                }
            }
            None => None,
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    WrongType,
    UtfError,
    ConvertError,
    NoTerminator
}

pub fn decode(data: &[u8]) -> Result<(Value, usize), ParseError> {
    match data[0] {
        // integer
        b'i' => {
            let data = &data[1..];

            let len = data.iter().position(|b| *b == b'e').ok_or(ParseError::NoTerminator)?;
            let int: i64 = {
                let str = str::from_utf8(&data[..len]).map_err(|_| ParseError::UtfError)?;
                str::parse(str).map_err(|_| ParseError::ConvertError)?
            };

            Ok((Value::Integer(int), 1 + len + 1))
        },

        // list
        b'l' => {
            let data = &data[1..];

            let mut off = 0usize;
            let mut list: Vec<Value> = Vec::new();

            while data[off] != b'e' {
                let (value, len) = decode(&data[off..])?;
                off += len;

                list.push(value);
            }

            Ok((Value::List(list), 1 + off + 1))
        },

        // dictionary
        b'd' => {
            let data = &data[1..];

            let mut off = 0usize;
            let mut dict: BTreeMap<Value, Value> = BTreeMap::new();
            
            while data[off] != b'e' {
                let (key, len) = decode(&data[off..])?;
                off += len;
                let (value, len) = decode(&data[off..])?;
                off += len;

                dict.insert(key, value);
            }

            Ok((Value::Dict(dict), 1 + off + 1))
        }

        // what's remaining is string type which has no specific type byte
        _ => {
            let delim_off = data.iter().position(|b| *b == b':').ok_or(ParseError::NoTerminator)?;
            let len: usize = {
                let len_str = from_utf8(&data[..delim_off]).map_err(|_| ParseError::ConvertError)?;
                str::parse(len_str).map_err(|_| ParseError::ConvertError)?
            };
            let str_data = &data[delim_off + 1 .. delim_off + 1 + len];
            let value = if let Ok(str) = from_utf8(str_data) {
                Value::String(ByteString::String(str.to_owned()))
            } else {
                Value::String(ByteString::ByteString(str_data.to_owned()))
            };

            Ok((value, delim_off + 1 + len))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::bencode::{ByteString, Value};
    use super::decode;

    #[test]
    fn parse_integer() {
        let (decoded, ret_len) = decode("i42e".as_bytes()).unwrap();
        assert_eq!(Value::Integer(42), decoded);
        assert_eq!(4, ret_len);
    }

    #[test]
    fn parse_string() {
        let (decoded, ret_len) = decode("5:hello".as_bytes()).unwrap();
        assert_eq!(Value::String(ByteString::String("hello".to_owned())), decoded);
        assert_eq!(7, ret_len);
    }

    #[test]
    fn parse_byte_string() {
        let (decoded, ret_len) = decode(&[0x35u8, 0x3Au8, 0xFFu8, 0xEAu8, 0xBCu8, 0xBDu8, 0xAAu8]).unwrap();
        assert_eq!(Value::String(ByteString::ByteString(vec![0xFFu8, 0xEAu8, 0xBCu8, 0xBDu8, 0xAAu8])), decoded);
        assert_eq!(7, ret_len);
    }

    #[test]
    fn parse_list() {
        let (decoded, ret_len) = decode("li0e6:seconde".as_bytes()).unwrap();
        assert_eq!(Value::List(vec![Value::Integer(0), Value::String(ByteString::String("second".to_owned()))]), decoded);
        assert_eq!(13, ret_len);
    }

    #[test]
    fn parse_dict() {
        let (decoded, ret_len) = decode("di0e6:seconde".as_bytes()).unwrap();
        let mut dict: BTreeMap<Value, Value> = BTreeMap::new();
        dict.insert(Value::Integer(0), Value::String(ByteString::String("second".to_owned())));
        assert_eq!(Value::Dict(dict), decoded);
        assert_eq!(13, ret_len);
    }
}