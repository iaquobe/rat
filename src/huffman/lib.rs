use std::collections::HashMap;
use std::collections::BinaryHeap;
use std::cmp::Reverse;
use bitvec::prelude::*;

#[derive(Debug,Eq,PartialEq, PartialOrd,Ord)]
enum Node {
    Root{left: Box<Node>, right: Box<Node>},
    Leaf(u8),
}

#[derive(Debug)]
pub struct Metadata {
    character_sequence: String,
    tree_encoding: String,
}

use Node::*; 


// text -> min heap for char count
fn frequency(text: &str) -> BinaryHeap<(Reverse<usize>, Node)> {
    let mut occurence_map: HashMap<u8, usize> = HashMap::new();

    for c in text.bytes() {
        *occurence_map.entry(c).or_insert(0) += 1;
    }

    // collect into binary heap with peek -> min
    occurence_map.into_iter()
        .map(|(c, count)| (Reverse(count), Leaf(c)))
        .collect()
}

// compile occurences into tree
fn huffman_tree(text: &str) -> Node {
    let mut frequencies = frequency(text); 

    // take 2 lowers and merge, return when only one element left
    loop {
        let first = frequencies.pop().expect("heap is empty: input text is empty");
        let second = frequencies.pop();

        match second {
            Some((Reverse(count), node)) => frequencies.push(
                (Reverse(count + first.0.0), Root{ left: (Box::from(first.1)), right: (Box::from(node)) })),
            None => break first.1,
        }
    }
}

/***
    returns huffman table (character, length, value)
    returns huffman table, that is a vector with:
        character: u8,
        code: bitvec
  */
pub fn huffman_table(text: &str) -> Vec<(u8, BitVec)> {
    let tree = huffman_tree(text); 
    let mut table: Vec<(u8, BitVec)> = Vec::new();

    fn explore_branch(table: &mut Vec<(u8, BitVec)>, node: &Node, code: BitVec) {
        match node {
            Leaf(c) => {table.push((*c, code));},
            Root { left, right } => {
                explore_branch(table, left, {
                    let mut new_code = code.clone();
                    new_code.push(false);
                    new_code
                }); 

                explore_branch(table, right, {
                    let mut new_code = code.clone();
                    new_code.push(true);
                    new_code
                });
            }
        }
    }
    explore_branch(&mut table, &tree, bitvec![]); 

    table
}


pub fn huffman_encoding_metadata(text: &str) -> Metadata {
    let tree  = huffman_tree(text); 

    let character_sequence: String = huffman_table(text)
        .iter()
        .map(|(c,_)| *c as char)
        .collect();

    let mut tree_encoding = String::new(); 
    fn explore_branch(node: &Node, tree_encoding: &mut String) {
        match node {
            Leaf(_) => {},
            Root { left, right } => {
                explore_branch(left, tree_encoding); 
                tree_encoding.push('0');
                explore_branch(right, tree_encoding); 
                tree_encoding.push('1');
            }
        }
    }
    explore_branch(&tree, &mut tree_encoding); 
    
    Metadata{character_sequence, tree_encoding }
}

/***
    take text, and return string wich can be written to file
  */
pub fn huffman_encoding(text: &str) -> String {

    // get metadata, add character sequence and tree encoding
    let metadata = huffman_encoding_metadata(text); 

    let mut res = String::from(metadata.character_sequence); 
    res.push_str(&metadata.tree_encoding); 



    // encode with tree 
    let table = huffman_table(text); 


    res
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frequency_test_short() {
        let s = "acab";
        let mut result   = frequency(s); 
        let mut expected = BinaryHeap::from([
                                            (Reverse(2), Leaf(b'a')),
                                            (Reverse(1), Leaf(b'b')),
                                            (Reverse(1), Leaf(b'c'))]);

        loop {
            match (result.pop(),expected.pop()) {
                (Some(a), Some(b)) => assert_eq!(a, b),
                (None, None) => break,
                _ => assert!(false)
            }
        }
    }

    #[test]
    fn frequency_test_long() {
        let s = "this is an example of a huffman tree";
        let mut result = frequency(s); 
        let mut expected = BinaryHeap::from([
                                            (Reverse(7), Leaf(b' ')),
                                            (Reverse(4), Leaf(b'a')),
                                            (Reverse(4), Leaf(b'e')),
                                            (Reverse(3), Leaf(b'f')),
                                            (Reverse(2), Leaf(b'h')),
                                            (Reverse(2), Leaf(b'i')),
                                            (Reverse(2), Leaf(b'm')),
                                            (Reverse(2), Leaf(b'n')),
                                            (Reverse(2), Leaf(b's')),
                                            (Reverse(2), Leaf(b't')),
                                            (Reverse(1), Leaf(b'l')),
                                            (Reverse(1), Leaf(b'o')),
                                            (Reverse(1), Leaf(b'p')),
                                            (Reverse(1), Leaf(b'r')),
                                            (Reverse(1), Leaf(b'u')),
                                            (Reverse(1), Leaf(b'x'))]);

        loop {
            match (result.pop(),expected.pop()) {
                (Some(a), Some(b)) => assert_eq!(a, b),
                (None, None) => break,
                _ => assert!(false)
            }
        }
    }

    #[test]
    fn tree_test_short_1() {
        let text = "acab";
        let tree = huffman_tree(text); 
        let correct = Root { 
            left: Box::new(Leaf(b'a')), 
            right: Box::new(Root {
                left: Box::new(Leaf(b'c')), 
                right: Box::new(Leaf(b'b')) }) };

        assert_eq!(tree, correct);
    }

    #[test]
    fn tree_test_short_2() {
        let text = "abcd";
        let tree = huffman_tree(text); 
        let correct = Root { 
            left: Box::new(Root {
                left: Box::new(Leaf(b'd')), 
                right: Box::new(Leaf(b'c')) }),
                right: Box::new(Root {
                    left: Box::new(Leaf(b'b')), 
                    right: Box::new(Leaf(b'a')) }) };

        assert_eq!(tree, correct);
    }

    #[test]
    fn table_long() { 
        let text = "this is an example of a huffman tree";
        let frequencies: HashMap<u8, usize> = frequency(text).into_iter()
            .filter_map(|(Reverse(count), character)| match character {
                Node::Leaf(ch) => Some((ch, count)),
                _ => None
                
            })
            .collect(); 

        // (length_of_code, count_of_char)
        let mut table: Vec<(usize, usize)> = huffman_table(text).into_iter() 
            .map(|(character, code)| (code.len(), *frequencies.get(&character).unwrap()))
            .collect(); 
        table.sort();


        // check that as count decreases, code length increases
        let mut last_code_len = 0; 
        for (code_len, _) in table{
            assert!(last_code_len <= code_len); 
            last_code_len = code_len; 
        }
    }
}
