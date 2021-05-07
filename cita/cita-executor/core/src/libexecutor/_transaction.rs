use std::collections::HashMap;
use std::*;
//bincode
pub use bincode::*;
pub use serde_derive::*;
pub use petgraph::Graph;
//use uilt::*;
//use std::string::Stirng;
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct TransactionInfo{
    pub tran_id: usize,
    pub read_set: Option<Vec<String>>,
    pub read_values: Option<Vec<String>>,
    pub write_set: Option<Vec<String>>,
    pub write_values: Option<Vec<String>>,
    pub weight_: i32,
}
impl TransactionInfo{
    pub fn new(tranid:usize,readset:Option<Vec<String>>,read_values:Option<Vec<String>>,writeset:Option<Vec<String>>,write_values:Option<Vec<String>>)
    -> TransactionInfo{
        let tx = TransactionInfo{
            tran_id : tranid,
            read_set: readset,
            read_values: read_values,
            write_set: writeset,
            write_values: write_values,
            weight_ : 1,
        };
        tx
    }
    pub fn print_tran(&self){
        println!("tranID: {}",self.tran_id);
        if self.read_set != None{
            let vec1 = self.read_set.clone().unwrap().into_iter();
            let vec2 = self.read_values.clone().unwrap();
            let mut index : usize = 0;
            for n in vec1{
                print!("R({})={} ",n,vec2[index]);
                index = index + 1;
            }
            println!("");
        }
        if self.write_set != None{
            let vec1 = self.write_set.clone().unwrap().into_iter();
            let vec2 = self.write_values.clone().unwrap();
            let mut index : usize = 0;
            for n in vec1{
                print!("W({})={} ",n,vec2[index]);
                index = index + 1;
            }
            println!("");
        }
    }
    pub fn set_read(&mut self,readset:Option<Vec<String>>,readvalues:Option<Vec<String>>){
        self.read_set = readset;
        self.read_values = readvalues;
    }
    pub fn set_write_value(&mut self,writeset:Option<Vec<String>>,writevalues:Option<Vec<String>>){
        self.write_set = writeset;
        self.write_values = writevalues;
    }
    
}
//运用bincode序列化和反序列化图信息
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct BincodeGraph{
    pub nodes_set: Vec<TransactionInfo>,
    pub edges_set: Vec<(usize, usize)>,
    pub toposort: Vec<usize>,
}
impl BincodeGraph{
    pub fn new(vec_nodes: Vec<TransactionInfo>, vec_edges:Vec<(usize, usize)>, vec_usize: Vec<usize>) -> Self{
        let bincodegraph = BincodeGraph{
            nodes_set: vec_nodes,
            edges_set: vec_edges,
            toposort: vec_usize,
        };
        bincodegraph
    }
    pub fn create_graph(&self) -> Graph<TransactionInfo, i32>{
        let mut g = Graph::new();
        let iter = self.nodes_set.iter();
        //创建一个hashmap存储交易的index和Nodeindex和键值对,插入边的时候好寻找
        let mut index_nodeindex_map = HashMap::new();
        //添加图节点
        for node in iter{
            let node_index = g.add_node(node.clone());
            let index = node.tran_id.clone();
            index_nodeindex_map.insert(index,node_index.clone());
        }
        let iter = self.edges_set.iter();
        for edge in iter{
            let (from, to) = edge.clone();
            let from_node_index = index_nodeindex_map.get(&from).unwrap().clone();
            let to_node_index = index_nodeindex_map.get(&to).unwrap().clone();
            g.add_edge(from_node_index, to_node_index, 1);
        }
        return g;
    }

}