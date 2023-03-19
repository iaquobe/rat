use std::collections::HashMap;
use std::collections::BinaryHeap;
use std::cmp::Reverse;
use std::slice::Iter;
use bitvec::prelude::BitVec;

/// A node in the Huffman tree
///
/// # Possible Values
/// 
/// - Root{left: Node, right: Node}
/// - Leaf(u8)
#[derive(Debug,Eq,PartialEq, PartialOrd,Ord)]
enum Node {
    Root{left: Box<Node>, right: Box<Node>},
    Leaf(u8),
}

pub type Codeword = BitVec<u8>;

pub struct FileData {
    characters: Vec<u8>,
    tree: Codeword,
    text: Codeword,
}

use Node::*; 


/// Takes text and returns a Nodes of characters found, sorted by least used
///
/// # Params
/// text: &str; string 
///
/// # Returns
/// sorted_nodes: BinaryHeap<(Reverse<usize>, Node)>
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

/// Takes text and returns Huffman tree for text
/// 
/// # Returns
/// root: Node; recursive structure can be Root or Leaf
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

/// takes text and returns huffman table (character, code)
/// 
/// # Returns
/// Vec<(character: u8, code: Bitvec)>
fn huffman_table(tree: &Node) -> HashMap<u8, Codeword> {
    let mut table: HashMap<u8, Codeword> = HashMap::new();

    fn explore_branch(table: &mut HashMap<u8, Codeword>, node: &Node, code: Codeword) {
        match node {
            Leaf(c) => {table.insert(*c, code);},
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
    explore_branch(&mut table, &tree, Codeword::new()); 

    table
}


/// takes huffman tree and returns character list and huffman encoding
/// 
/// # Returns 
/// Vec<u8>: characters in tree, in order from left to right
/// Bitvec: encoding of tree
/// 
/// # References
/// https://www.cs.scranton.edu/~mccloske/courses/cmps340/huff_tree_encoding.html
fn huffman_encode_tree(root: &Node) -> (Vec<u8>, Codeword) {
    let mut char_order: Vec<u8> = Vec::new();
    let mut tree_encoding = Codeword::new();

    // recursively explore tree to find order of characters and encoding of tree
    fn explore_tree(tree: &Node, characters: &mut Vec<u8>, tree_encoding: &mut Codeword) {
        match tree {
            Root { left, right } => {
                tree_encoding.push(false);
                explore_tree(left, characters, tree_encoding);

                tree_encoding.push(true);
                explore_tree(right, characters, tree_encoding);
            }
            Leaf(character) => {
                characters.push(*character);
            }
        }
    }
    explore_tree(root, &mut char_order, &mut tree_encoding);

    (char_order, tree_encoding)
}

/// Takes huffman tree and text and returns encoded text back
/// 
/// # Returns 
/// 
/// 
fn huffman_encode_text(text: &str, code_table: &HashMap<u8, Codeword>) -> Codeword {
    text.bytes()
        .map(|character| code_table.get(&character).expect("character in text, but not in tree"))
        .fold(Codeword::new(), |mut acc, code| {acc.extend(code); acc})
}



/// Takes and returns encoded text
/// 
/// # Returns 
/// Bitvec, which contains:
/// - characters 
/// - encoded tree
/// - encoded text
pub fn huffman_encode(text: &str) -> FileData {
    let tree = huffman_tree(text); 
    let code_table = huffman_table(&tree);

    let text = huffman_encode_text(text, &code_table);
    let (characters, tree) = huffman_encode_tree(&tree);

    FileData { characters, tree, text}
}

/// Take character list and tree and return hashmap of char -> code
fn huffman_decode_tree(characters: &Vec<u8>, tree: &Codeword) -> HashMap<Codeword, u8> {
    let mut character = characters.iter();
    let mut code: Codeword = Codeword::new();
    let mut result: HashMap<Codeword, u8> = HashMap::new();

    // when 0 add it to current code
    // when 1 means last number was an end: 
    // - add the other number to hashmap
    // - remove from code while poped value == 1
    // - append 1 to it
    for bit in tree.iter() {
        match *bit {
            true => {
                // add number to hashmap 
                result.insert(code.clone(),*character.next().expect("not enough characters for this tree"));

                // remove from code until first false
                while code.pop().unwrap() {}

                // append 1 to code
                code.push(true);
            },
            false => {
                code.push(false);
            },
        }
    }
    result.insert(code, *character.next().expect("not enough characters for this tree"));

    result
}


/// Decodes text in codeword
///
/// # Returns 
pub fn huffman_decode(filedata: &FileData) -> String {
    let code_table: HashMap<Codeword, u8> = huffman_decode_tree(&filedata.characters, &filedata.tree);

    // read from encoded text until bitvec found in table -> clear code
    let mut code = Codeword::new();
    let mut bytes = Vec::new();

    for bit in filedata.text.iter() {
        code.push(*bit);
        match code_table.get(&code) {
            Some(character) => {
                bytes.push(*character);
                code.clear();
            },
            None => {},
        }
    }
    
    String::from_utf8(bytes).unwrap()
}




#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_frequency() {
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
    fn test_huffman_tree_short() {
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
    fn test_huffman_tree_long() {
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
    fn test_huffman_table() { 
        let text = "this is an example of a huffman tree";
        let frequencies: HashMap<u8, usize> = frequency(text).into_iter()
            .filter_map(|(Reverse(count), character)| match character {
                Leaf(ch) => Some((ch, count)),
                _ => None

            })
        .collect(); 

        // to sorted vec of (length_of_code, count_of_char)
        let mut table: Vec<(usize, usize)> = huffman_table(&huffman_tree(text)).into_iter() 
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

    #[test]
    fn test_huffman_encode_tree(){
        let text = "this is an example of a huffman tree";
        let textmap: HashSet<u8> = text.bytes().collect();
        let tree = huffman_tree(text);
        let (characters, tree_encoding) = huffman_encode_tree(&tree);

        fn count_edges(node: &Node) -> usize {
            match node {
                Root { left, right }    => 2 + count_edges(left) + count_edges(right),
                Leaf(_)                 => 0,
            }
        }
        let acc = count_edges(&tree);
        
        assert_eq!(characters.len(), textmap.len());
        assert_eq!(tree_encoding.len(), acc)

    }

    /*
    #[test]
    fn test_huffman_decode() {

    }
    */

    #[test]
    fn test_huffman_decode_tree() {
        let text = "this is a test string for encode and decode";
        //let text = "ab";
        let encoded: FileData = huffman_encode(text);
        let tree = huffman_tree(text);
        let table: HashMap<Codeword, u8> = huffman_table(&tree)
            .into_iter()
            .map(|(v,k)| (k,v))
            .collect();

        let decoded_table = huffman_decode_tree(&encoded.characters, &encoded.tree);
        

        assert_eq!(table, decoded_table);
    }

    #[test]
    fn test_huffman_decode() {
        let text = "this is a test string for encode and decode";
        //let text = "ab";
        let encoded: FileData = huffman_encode(text);
        let decoded = huffman_decode(&encoded);
        

        assert_eq!(text, decoded);
    }
}
