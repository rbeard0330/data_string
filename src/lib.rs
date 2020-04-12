use std::collections::HashMap;
use std::convert::{AsRef};
use std::fmt::Display;

const SUBSTITUTE: u8 = 26; // ASCII SUBSTITUTE CODE
const ASCII_MAX: u8 = 127;


#[derive(Debug, PartialEq)]
pub struct DataString {
    string_rep: Option<String>,
    replacements: HashMap<u8, Vec<usize>>
}

impl DataString {
    pub fn from_vec(mut data: Vec<u8>) -> Self {
        let mut replacements: HashMap<u8, Vec<usize>> = HashMap::new();
        for (index, value) in data.iter_mut().enumerate() {
            if *value > ASCII_MAX {
                replacements.entry(*value).or_default().push(index);
                *value = SUBSTITUTE;
            }
        }
        for value in data.iter() {
            debug_assert!(*value <= ASCII_MAX);
        }
        unsafe {
            DataString {
            string_rep: Some(String::from_utf8_unchecked(data)),
            replacements
            }
        }
    }
    pub fn take_string(&mut self) -> Option<String> {
        self.string_rep.take()
    }
    pub fn return_string(&mut self, s: String) -> Option<String> {
        match self.string_rep {
            None => {
                self.string_rep = Some(s);
                None
            },
            Some(_) => Some(s)
        }
    }
    pub fn take_data(&mut self) -> Option<Vec<u8>> {
        if let None = self.string_rep { return None };
        let mut data = self.string_rep.take().unwrap().into_bytes();
        for (saved_data, index_vec) in self.replacements.iter() {
            for index in index_vec.iter() {
                data[*index] = *saved_data;
            }
        };
        Some(data)
    }
    pub fn return_data_unchecked(&mut self, mut data: Vec<u8>) -> Option<Vec<u8>> {
        if let Some(_) = self.string_rep { return Some(data) };
        for index_vec in self.replacements.values() {
            for index in index_vec {
                data[*index] = SUBSTITUTE;
            }
        }
        self.string_rep = Some(String::from_utf8(data).unwrap());
        None
    }
}

impl AsRef<str> for DataString {
    fn as_ref(&self) -> &str {
        match self.string_rep {
            None => "",
            Some(ref s) => &s
        }
    }
}

impl AsRef<[u8]> for DataString {
    fn as_ref(&self) -> &[u8] {
        match self.string_rep {
            None => &[],
            Some(ref s) => &s.as_ref()
        }
    }
}

impl Display for DataString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.string_rep {
            None => write!(f, "")?,
            Some(ref s) => {
                for c in self.string_rep.as_ref().unwrap().chars() {
                    if c != SUBSTITUTE as char {
                        write!(f, "{}", c)?;
                    } else {
                        write!(f, "�")?;
                    };
                };
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Write;
    use super::*;

    const TEST_SLICE: &[u8; 30] = b"Want binary? Here's data: \x1A\x1A\x1A\x1A";

    fn get_test_data() -> Vec<u8> {
        let mut data: Vec<u8> = b"Want binary? Here's data: ".iter().map(|r| *r).collect();
        data.extend(vec!(128, 199, 200, 255));
        data
    }

    fn get_string_equivalent() -> String {
        "Want binary? Here's data: \x1A\x1A\x1A\x1A".to_string()
    }

    fn get_expected_display_string() -> String {
        "Want binary? Here's data: ����".to_string()
    }

    #[test]
    fn basic_test() {
        let data = get_test_data();
        let data_copy = data.clone();
        let mut d_string = DataString::from_vec(data);
        let mut test_output = String::new();
        write!(test_output, "{}", d_string).unwrap();
        assert_eq!(test_output, get_expected_display_string());
        let data_ref: &[u8] = d_string.as_ref();
        let string_ref: &str = d_string.as_ref();
        assert_eq!(data_ref, TEST_SLICE);
        assert_eq!(string_ref, &get_string_equivalent());
        let reconstituted_data = d_string.take_data();
        assert_eq!(Some(data_copy), reconstituted_data);
    }

    #[test]
    fn no_double_borrows() {
        let data = get_test_data();
        let mut d_string = DataString::from_vec(data);
        assert_eq!(d_string.take_string(), Some(get_string_equivalent()));
        assert_eq!(d_string.take_string(), None);
        assert_eq!(d_string.take_data(), None);
    }

    #[test]
    fn no_double_returns() {
        let data = get_test_data();
        let mut d_string = DataString::from_vec(data);
        let new_data = vec!(1, 2, 3);
        let copied_new_data = new_data.clone();
        assert_eq!(d_string.return_data_unchecked(new_data), Some(copied_new_data));
        let new_string = "test".to_string();
        let copied_new_string = new_string.clone();
        assert_eq!(d_string.return_string(new_string), Some(copied_new_string));
    }

    #[test]
    fn data_borrow_works() {
        let data = get_test_data();
        let mut d_string = DataString::from_vec(data);
        let borrowed_data = d_string.take_data();
        assert_eq!(borrowed_data, Some(get_test_data()));
        assert_eq!(d_string.return_data_unchecked(borrowed_data.unwrap()), None);
        assert_eq!(d_string.take_data().unwrap(), get_test_data());
    }

    #[test]
    fn string_borrow_works() {
        let data = get_test_data();
        let mut d_string = DataString::from_vec(data);
        let borrowed_string = d_string.take_string();
        assert_eq!(borrowed_string, Some(get_string_equivalent()));
        assert_eq!(d_string.return_string(borrowed_string.unwrap()), None);
        assert_eq!(d_string.take_data().unwrap(), get_test_data());
    }

    #[test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: FromUtf8Error")]
    fn panic_if_data_modified() {
        let data = vec!(b'a', 200, b'g');
        let mut d_string = DataString::from_vec(data);
        let mut borrowed_data = d_string.take_data().unwrap();
        // Introducing invalid character at a new place
        borrowed_data[0] = 200;
        d_string.return_data_unchecked(borrowed_data);
    }


    
}
