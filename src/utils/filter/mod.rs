use std::fs;
use std::collections::HashSet;
use std::sync::LazyLock;

#[derive(Clone,Debug)]
pub enum Section {
    Word(String),
    SpaceOrPunc(String),
}
use Section::{Word,SpaceOrPunc};

static SWEARS: LazyLock<Vec<String>> = LazyLock::new(|| {
    let contents = fs::read_to_string("src/utils/filter/bannedwords.txt")
    .expect("Should have been able to read the file");

    return contents.split("\n").map(|s| {return String::from(s.to_lowercase().as_str().trim())}).collect();   
});

static MOSTCOMMONWORDS: LazyLock<HashSet<String>> = LazyLock::new(|| {
    let contents = fs::read_to_string("src/utils/filter/mostcommonwords.txt")
    .expect("Should have been able to read the file");

    return contents.split("\n").map(|s| {return String::from(s.to_lowercase().as_str().trim())}).collect();   
});

static ALLWORDS: LazyLock<HashSet<String>> = LazyLock::new(|| {
    let contents = fs::read_to_string("src/utils/filter/words_alpha.txt")
    .expect("Should have been able to read the file");

    return contents.split("\n").map(|s| {return String::from(s.to_lowercase().as_str().trim())}).collect();
});

pub fn get_most_common_words() -> &'static HashSet<String> {
    return &*MOSTCOMMONWORDS
}

pub fn get_all_word_hashset() -> &'static HashSet<String> {

    return &*ALLWORDS;
}

pub fn stand_censore(msg: &mut String) {
    let original_len = msg.len();
    msg.clear();
    msg.push_str("*".repeat(original_len).as_str() );
}

pub fn swear_censore(msg: &mut String) {
    let original_len = msg.len();
    msg.clear();
    msg.push_str("_".repeat(original_len).as_str() );
}

pub fn combine_section_vec_into_string(msg_section: Vec<Section>) -> String {
    let mut msg = String::new();
    for sect in msg_section {
        match sect {
            Word(string) => {
                msg.push_str(string.as_str());
            }
            SpaceOrPunc(string) => {
                msg.push_str(string.as_str());
            }
        }
    }

    return msg;
}

pub fn split_into_word_vec(text: &String) ->  Vec<Section> {
    let mut vec: Vec<Section> = Vec::new();

    let punc = r#" .,<>';:[]{}#~/!"£$%^&*()"#;
    let mut current_section: Section = Word(String::new());

    for char in text.chars() {
        if let Some(_)  = punc.find(char) {
            match &mut current_section {
                Word(_string) => {
                    vec.push(current_section);
                    current_section =  SpaceOrPunc(String::from(char));
                }
                SpaceOrPunc(string) => {
                    string.push(char);
                }
            }
        } else {
            match &mut current_section {
                Word(string) => {
                    string.push(char);
                }
                SpaceOrPunc(_string) => {
                    vec.push(current_section);
                    current_section = Word(String::from(char));
                }
            }
        }
    }
    vec.push(current_section);

    let first = vec.get(0).unwrap();
    if let Section::Word(string) = first {
        if string.len() == 0 {
            vec.remove(0);
        }
    }
    return vec;
}

pub fn censore_message(mut msg: String) -> String {
    let swear_vec = &*SWEARS;
  
    let mut sects = split_into_word_vec(&msg);
    for mut sect in &mut sects {
        match &mut sect {
            Word(string) => {
                for swear in swear_vec {
                    if string.contains(swear) {
                        swear_censore(string);
                        break
                    }
                }
            },
            _ => {},
        }
    }

    msg = combine_section_vec_into_string(sects);
    return msg;
}