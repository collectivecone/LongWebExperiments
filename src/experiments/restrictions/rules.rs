#![allow(dead_code)]
#![allow(unused_imports)]

use std::{
    ops::DerefMut, sync::Mutex,
    collections::HashSet,
};
use fastrand;
use self::super::super::restrictions::User;

#[derive(Clone,Copy,Debug)]
pub enum MessageType {
    User = 0,
    System = 1,
}
#[derive(Clone,Debug)]
pub struct Message {
    pub text: String,
    pub by: String,
    pub message_type: MessageType,
    pub time: u64,
}

#[derive(Debug)]
pub struct Rule  {
    pub name: String,
    pub desc: String,
    pub weight: f32,
    pub process: fn(Message,&User,&Vec<Message>) -> Message,
    pub _starttime: u64,
    pub _endtime: u64,
}

impl Default for Rule {
    fn default() -> Rule {
        Rule {
            name: String::from("Default name"),
            desc: String::from("Default desc"),
            weight: 1.0,
            process: |msg,_,_| {return msg},
            _starttime: 0,
            _endtime: 0,
        }
    }
}

pub static GLOBAL_RULES: Mutex<Vec<Rule>> = Mutex::new(Vec::new());



use crate::utils::filter::{
    self,
    combine_section_vec_into_string,
    split_into_word_vec,
    stand_censore,


};
use filter::Section::{Word,SpaceOrPunc,self};


pub fn initalise_rules() {
    let mut guard  = GLOBAL_RULES.lock().unwrap();
    let rules = guard.deref_mut();



    rules.push(Rule{ 
        name: String::from("Fck Vwls"), 
        desc: String::from("All vowels are removed"),
        weight: 1.0,
        process: |mut msg, _, _|  {
            let vowels = "aeiou".chars();
            let mut string: String= String::new();
            'outer: for letter in msg.text.chars() {
                for char in vowels.clone() {
                    if char == letter {continue 'outer}
                }
                string.push(letter);
            }
            msg.text = string;
            return msg;
        }, 
        ..Default::default()
    });

    rules.push(Rule{ 
        name: String::from("Hopeless sacrastic"), 
        desc: String::from("All characters are randomly upper and lower case"),
        weight: 1.0,
        process: |mut msg, _, _|  {
            msg.text = msg.text.chars().map(|char| {
                if fastrand::bool() {
                    return char.to_uppercase().to_string().chars().last().unwrap();
                } else {
                    return char.to_lowercase().to_string().chars().last().unwrap();
                }
            }).collect();
            return msg;
        }, 
        ..Default::default()
    });

    /* 
    rules.push(Rule{ 
        name: String::from("Palindrome"), 
        desc: String::from("Sentences are forced to be palindromic (same backwards and forwards ignoring punctation)"),
        weight: 1.0,
        process: |mut msg, _, _|  {
            let ignore = r#",.<>';:[]{}#~/!"£$%^&*()"#;
            let chars = msg.text.chars().filter(|char| {
                if let Some(_) = ignore.find(*char) {
                    return false;
                } else {
                    return true;
                };
            });
            let rev_chars = chars.clone().rev();
            if rev_chars.clone().collect::<String>().to_lowercase() == chars.clone().collect::<String>().to_lowercase() {
                let mut pali_chars: Vec<char> = Vec::new();
                let a: Vec<char> = chars.clone().collect();
                let rev: Vec<char> = rev_chars.collect();
                let len = a.len();
                for (i,char) in chars.enumerate() {
               
                    let j: usize = len - 1 - i;
                    if i > j {
                        pali_chars.push(rev[i]);
                       
                    } else {
                        pali_chars.push(char);
                    }
                }
                let str = pali_chars.iter().collect::<String>();
                let i = 0;


                println!("{}", str);

                return msg;
            } else {
            
                return msg;
            }
        }, 
        ..Default::default()
    });
*/

    rules.push(Rule{ 
        name: String::from("Even Steven"), 
        desc: String::from("Only sentences with an even amount of words are permited or otherwise every other word is censored"),
        weight: 1.0,
        process: |mut msg, _, _|  {
            let mut sects = split_into_word_vec(&msg.text);
            let mut count: u32 = 0;
            for sect in &sects {
                if let Section::Word(_) = sect {
                    count += 1;
                }
            }
            if count % 2 == 0 {return msg} 
               
            let mut i = 0;
            for sect in sects.iter_mut() {
                if let Section::Word(string) = sect {
                    i += 1;
                    if i % 2 == 1 {
                        stand_censore(string);
                    }
                }
            }
            msg.text = combine_section_vec_into_string(sects);

            return msg;
        }, 
        ..Default::default()
    });

    
    rules.push(Rule{ 
        name: String::from("Hive mind"), 
        desc: String::from("If you do not use a word that has been sent in the last 30 messages, it is replaced by one that has"),
        weight: 1.0,
        process: |mut msg: Message, _, msg_history|  {
            let mut word_list: Vec<String> = Vec::new();
            word_list.push(String::from("elephant"));
            for i in (msg_history.len().max(30) - 30)..msg_history.len() {
                let msg = msg_history.get(i).unwrap();
                let sects = split_into_word_vec(&msg.text);
                for sect in sects {
                    if let Word(string) = sect {
                        let a= string.to_lowercase();
                        word_list.push(a);
                    }
                }
            }


            let mut sects = split_into_word_vec(&msg.text);
 
            for sect in &mut sects {
                match sect {
                    Word(string) => {
                        let mut exists: bool = false; 
                        for word in &word_list {
                            if word == string {
                                exists = true;
                                break
                            }
                        }
                        if !exists {
                            string.clear();
                            let random_word = &word_list[fastrand::usize(0..word_list.len())];
                            string.push_str(random_word.as_str());
                        }
                    },
                    _ => {},
                }
            }
           
            msg.text = combine_section_vec_into_string(sects);
            return msg;
        }, 
        ..Default::default()
    });

    rules.push(Rule{ 
        name: String::from("Gluttony"), 
        desc: String::from("You can only use words that include a single vowel or its censored"),
        weight: 0.2,
        process: |mut msg, _, _|  {
            let vowels = "aeiou".chars();
            let mut sects = split_into_word_vec(&msg.text);
            for mut sect in &mut sects {
                if let Word(string) = &mut sect {
                    let mut count = 0;
                    for letter in string.chars() {
                        for char in vowels.clone() {
                            if char == letter {count += 1;}
                        }
                    }
                    if count != 1 {
                        stand_censore(string);
                    }
                }
            }

            msg.text = combine_section_vec_into_string(sects);
            return msg;
        }, 
        ..Default::default()
    });

    rules.push(Rule{ 
        name: String::from("The elephant in the room"), 
        desc: String::from("you must mention the elephant in the room or your message just becomes 'Elephant'"),
        weight: 0.2,
        process: |mut msg, _, _|  {
            if let Some(_) = msg.text.to_lowercase().as_str().find("elephant") {
                return msg
            } else {
                msg.text = String::from("Elephant");
                return msg;
            };
        }, 
        ..Default::default()
    });

    rules.push(Rule{ 
        name: String::from("Simplicity"), 
        desc: String::from("You must only use the top 1000 most common english words or it is replaced by one that is"),
        weight: 0.2,
        process: |mut msg, _, _|  {
            let word_hash: &HashSet<String> = filter::get_most_common_words();
            let word_list: Vec<String> = word_hash.iter().map(|str| str.clone()).collect();

            let mut sects = split_into_word_vec(&msg.text);
            for mut sect in &mut sects {
                match &mut sect {
                    Word(string) => {
                        if !word_hash.contains(&string.to_lowercase()) {
                            string.clear();
                            let random_word = &word_list[fastrand::usize(0..word_list.len())];
                            string.push_str(random_word.as_str());
                        }
                    },
                    _ => {},
                }
            }
            msg.text = combine_section_vec_into_string(sects);
            return msg;
        }, 
        ..Default::default()
    });

    rules.push(Rule{ 
        name: String::from("Research document"), 
        desc: String::from("You cannot use personal pronouns; which are replaced by the name derek"),
        weight: 0.2,
        process: |mut msg, _, _|  {
            let word_list: Vec<&str> = vec!("I","me","you","he","she","they","them","him","her","hers","his","its","theirs","our","your");

            let mut sects = split_into_word_vec(&msg.text);
            for mut sect in &mut sects {
                match &mut sect {
                    Word(string) => {
                        let mut exists: bool = false; 
                        for word in &word_list {
                            if *word.to_lowercase() == string.as_str().to_lowercase() {
                                exists = true;
                                break
                            }
                        }
                        if exists {
                            string.clear();
                            string.push_str("derek");
                        }
                    },
                    _ => {},
                }
            }
            msg.text = combine_section_vec_into_string(sects);

            return msg;
        }, 
        ..Default::default()
    });


    rules.push(Rule{ 
        name: String::from("English degree"), 
        desc: String::from("50% of your message must be valid english words above 8 letters or all words under 8 letters are censored"),
        weight: 0.2,
        process: |mut msg, _, _|  {
            let word_hash = filter::get_all_word_hashset();

            let mut total_words: f32 = 0.0;
            let mut total_passing_words: f32 = 0.0;

            let mut sects = split_into_word_vec(&msg.text);
            for mut sect in &mut sects {
                match &mut sect {
                    Word(string) => {
                        total_words += 1.0;
                        if string.len() > 8 {
                            if word_hash.contains(&string.to_lowercase())  {
                                total_passing_words += 1.0;
                                continue
                            }
                        }
                        stand_censore(string);
                    },
                    _ => {},
                }
            }

            if (total_passing_words / total_words) >= 0.5 {
                return msg;
            }

            msg.text = combine_section_vec_into_string(sects);

            return msg;
        }, 
        ..Default::default()
    });

    rules.push(Rule{ 
        name: String::from("Ordered"), 
        desc: String::from("All words must be in alphabetical order in your message"),
        weight: 0.2,
        process: |mut msg, _, _|  {
            let mut sects = split_into_word_vec(&msg.text);
            let mut words: Vec<String> = Vec::new();
            for sect in &mut sects {
                if let Section::Word(string) = sect {
                    words.push(string.clone());
                } 
            };
            words.sort_by_key(|name| name.to_lowercase());
            let mut i: usize = 0;
            
            for sect in &mut sects {
                if let Section::Word(string) = sect {
                    string.clear();
                    string.push_str(words[i].as_str());
                    i += 1;
                }
            }

            msg.text = combine_section_vec_into_string(sects);
            return msg;
        }, 
        ..Default::default()
    });

    rules.push(Rule{ 
        name: String::from("Unique"), 
        desc: String::from("You cannot use words used by the most 10 recent messages"),
        weight: 1.0,
        process: |mut msg, _, msg_history|  {
            let mut word_list: Vec<String> = Vec::new();
            for i in ((msg_history.len() - 10).max(0))..msg_history.len(){
                let msg = msg_history.get(i).unwrap();
     
                let sects = split_into_word_vec(&msg.text);
                for sect in sects {
                    if let Word(string) = sect {
                        let a= string.to_lowercase();
                        word_list.push(a);
                    }
                }
            }

            let mut sects = split_into_word_vec(&msg.text);
            for mut sect in &mut sects {
                match &mut sect {
                    Word(string) => {
                        let mut exists: bool = false; 
                        for word in &word_list {
                            if word.to_lowercase() == string.to_lowercase() {
                                exists = true;
                                break
                            }
                        }
                        if exists {
                            stand_censore(string);
                        }
                    },
                    _ => {},
                }
            }

            msg.text = combine_section_vec_into_string(sects);


            return msg;
        }, 
        ..Default::default()
    });


    rules.push(Rule{ 
        name: String::from("Corruption"), 
        desc: String::from("Random letters get swap with ones near themselves"),
        weight: 1.0,
        process: |mut msg, _, _|  {
            let mut chars = msg.text.chars().collect::<Vec<_>>();

            for _ in 0..10 {
                let random_index = fastrand::usize(0..chars.len());

                let mut new_index = random_index;
                if fastrand::bool() {
                    if new_index != chars.len() - 1 {
                        new_index += 1
                    }
                } else {
                    if new_index != 0 {
                        new_index -= 1
                    } 
                   
                }
                new_index = new_index.clamp(0,chars.len() - 1);

                chars.swap(random_index,new_index);
            }

            msg.text = chars.into_iter().collect();

            return msg;
        }, 
        ..Default::default()
    });

    /* 
    rules.push(Rule{ 
        name: String::from("No repeat letters"), 
        desc: String::from("Random letters get swap with ones near themselves"),
        weight: 1.0,
        process: |mut msg, _, _|  {
            return msg;
        }, 
        ..Default::default()
    }); */
}