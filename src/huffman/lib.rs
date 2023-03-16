mod huffman {
    use std::collections::HashMap;
    use std::collections::BinaryHeap;
    use std::cmp::Reverse;

    #[derive(Debug,Eq,PartialEq, PartialOrd,Ord)]
    enum Node {
        Node{left: Box<Node>, right: Box<Node>},
        Leaf(char),
    }

    
    // text -> vec<(char,count)>
    fn frequency(text: &str) -> Vec<(char, usize)> {
        let mut occurence_map: HashMap<char, usize> = HashMap::new();

        for c in text.chars() {
            *occurence_map.entry(c).or_insert(0) += 1;
        }

        let mut result: Vec<(char, usize)> = occurence_map.into_iter()
            .collect();

        result
    }

    // compile occurences into tree
    fn huffman_tree(text: &str) -> Node {
        let frequencies = frequency(text); 

        // collect into binary heap with peek -> min
        let mut heap: BinaryHeap<(Reverse<usize>, Node)> = frequencies.into_iter()
            .map(|(c, count)| (Reverse(count), Node::Leaf(c)))
            .collect();

        // take 2 lowers and merge, return when only one element left
        loop {
            let first = heap.pop().expect("heap is empty: input text is empty");
            let second = heap.pop();

            match second {
                Some((Reverse(count), node)) => heap.push((Reverse(count + first.0.0), Node::Node { left: (Box::from(first.1)), right: (Box::from(node)) })),
                None => break first.1,
            }
        }
    }

    // returns huffman table (character, length, value)
    fn huffman_table(text: &str) -> Vec<(char, u8, u8)> {
        let tree = huffman_tree(text); 
        let mut table: Vec<(char, u8, u8)> = Vec::new();

        fn explore_branch(table: &mut Vec<(char, u8, u8)>, node: &Node, length: u8, code: u8) {
            match node {
                Node::Leaf(c) => {table.push((*c, length, code));},
                Node::Node { left, right } => {
                    explore_branch(table, left, length+1, code << 1); 
                    explore_branch(table, right, length+1, (code << 1) + 1); 
                }
            }
        }
        explore_branch(&mut table, &tree, 0, 0); 

        table
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn frequency_short() {
            let s = "acab";
            let mut r = vec![('a', 2), ('b', 1), ('c',1)]; 
            let mut f = frequency(s); 
            r.sort();
            f.sort();

            assert_eq!(f, r);
        }

        #[test]
        fn frequency_long() {
            let s = "this is an example of a huffman tree";
            let mut r = vec![(' ', 7), ('a',4), ('e',4), ('f',3), ('h',2), ('i',2), ('m',2), ('n',2), ('s',2), ('t',2), ('l',1), ('o',1), ('p',1), ('r',1), ('u',1), ('x',1)];
            let mut f = frequency(s); 
            f.sort();
            r.sort();

            assert_eq!(f, r);
        }

        #[test]
        fn tree_short_1() {
            let text = "acab";
            let tree = huffman_tree(text); 
            let correct = Node::Node { 
                left: Box::new(Node::Leaf('a')), 
                right: Box::new(Node::Node {
                                left: Box::new(Node::Leaf('c')), 
                                right: Box::new(Node::Leaf('b')) }) };

            assert_eq!(tree, correct);
        }

        #[test]
        fn tree_short_2() {
            let text = "abcd";
            let tree = huffman_tree(text); 
            let correct = Node::Node { 
                left: Box::new(Node::Node {
                                left: Box::new(Node::Leaf('d')), 
                                right: Box::new(Node::Leaf('c')) }),
                right: Box::new(Node::Node {
                                left: Box::new(Node::Leaf('b')), 
                                right: Box::new(Node::Leaf('a')) }) };

            assert_eq!(tree, correct);
        }

        #[test]
        fn table_long() { 
            let s = "this is an example of a huffman tree";
            let frequencies: HashMap<char, usize> = frequency(s).into_iter()
                .collect(); 

            // (length_of_code, count_of_char)
            let mut table: Vec<(u8, usize)> = huffman_table(s).into_iter() 
                .map(|(character, lenght,_)| (lenght, *frequencies.get(&character).unwrap()))
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
}
