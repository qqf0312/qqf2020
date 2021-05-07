// CITA
// Copyright 2016-2019 Cryptape Technologies LLC.

// This program is free software: you can redistribute it
// and/or modify it under the terms of the GNU General Public
// License as published by the Free Software Foundation,
// either version 3 of the License, or (at your option) any
// later version.

// This program is distributed in the hope that it will be
// useful, but WITHOUT ANY WARRANTY; without even the implied
// warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR
// PURPOSE. See the GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use super::block::{ClosedBlock, ExecutedBlock, OpenBlock};
use super::economical_model::EconomicalModel;
use super::executor::Executor;
use super::executor::RwSet;
//qqf-code
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::engines::{NullEngine};
use super::_transaction::{TransactionInfo};
//use cita_types::U256;
pub use petgraph::Graph;
pub use petgraph::algo;
pub use petgraph::Direction;
pub use petgraph::prelude::NodeIndex;
pub use petgraph::graph::UnGraph;
pub use chrono::*;
//use protobuf::RepeatedField;
use super::batching_occ::BatchingOcc;
pub use bincode::{deserialize, serialize, Bounded}; 
pub use serde_derive::*;
//use std::sync::atomic::{AtomicUsize};
//use std::sync::atomic::{Ordering};
pub use cita_types::H256;
pub use std::thread;
pub use std::sync::mpsc::channel;
pub use threadpool::ThreadPool;
use std::time;
//pub use ethereum_types::H256;
//use std::thread;
//use time::Duration;
//qqf-code-end

#[cfg_attr(feature = "cargo-clippy", allow(clippy::large_enum_variant))]
pub enum StatusOfFSM {
    Initialize(OpenBlock),
    Pause(ExecutedBlock, usize),
    Execute(ExecutedBlock, usize),
    Finalize(ExecutedBlock),
}

impl std::fmt::Display for StatusOfFSM {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            StatusOfFSM::Initialize(ref open_block) => write!(
                f,
                "StatusOfFSM::Initialize(height: {}, parent_hash: {:?}, timestamp: {})",
                open_block.number(),
                open_block.parent_hash(),
                open_block.timestamp(),
            ),
            StatusOfFSM::Pause(ref executed_block, index) => write!(
                f,      
                "StatusOfFSM::Pause(height: {}, parent_hash: {:?}, state_root: {:?}, timestamp: {}, index: {})",
                executed_block.number(),
                executed_block.parent_hash(),
                executed_block.state.root(),
                executed_block.timestamp(),
                index,
            ),
            StatusOfFSM::Execute(ref executed_block, index) => write!(
                f,
                "StatusOfFSM::Execute(height: {}, parent_hash: {:?}, state_root: {:?}, timestamp: {}, index: {})",
                executed_block.number(),
                executed_block.parent_hash(),
                executed_block.state.root(),
                executed_block.timestamp(),
                index,
            ),
            StatusOfFSM::Finalize(ref executed_block) => write!(
                f,
                "StatusOfFSM::Finalize(height: {}, parent_hash: {:?}, state_root: {:?}, timestamp: {})",
                executed_block.number(),
                executed_block.parent_hash(),
                executed_block.state.root(),
                executed_block.timestamp(),
            ),
        }
    }
}

pub trait FSM {
    fn into_fsm(&mut self, open_block: OpenBlock) -> ClosedBlock;
    //qqf-code
    fn real_into_fsm(&mut self, open_block: OpenBlock) -> ClosedBlock;
    fn pre_into_fsm(&mut self, open_block: OpenBlock) -> BatchingOcc;
    fn fake_fsm_finalize(&self, executed_block: ExecutedBlock);
    fn pre_fsm_execute(&self, executed_block: ExecutedBlock, index: usize ,rw:&mut RwSet) -> StatusOfFSM;
    fn pre_fsm_execute_in_parallel(&self, open_block: OpenBlock, rw_real: &mut RwSet, bocc_real: &mut BatchingOcc) -> StatusOfFSM;
    fn test_pre_fsm_execute_in_parallel(&self, open_block: OpenBlock, rw_real: &mut RwSet, bocc_real: &mut BatchingOcc) -> StatusOfFSM;
    //fn pre_fsm_pause(&self, executed_block: ExecutedBlock, index: usize) -> StatusOfFSM;
    //fn pre_fsm_initialize(&self, open_block: OpenBlock, transaction_map: HashMap) -> StatusOfFSM;
    fn deocc_fsm_execute(&self, executed_block: ExecutedBlock, index: usize) -> StatusOfFSM;
    fn test_deocc_fsm_execute(&self, open_block: OpenBlock, index: usize);
    //对比并发的串行执行
    //fn serial_fsm_execute(&mut self, executed_block: ExecutedBlock, index: usize) -> ClosedBlock;
    fn test_1_deocc_fsm_execute(&mut self, executed_block: ExecutedBlock, open_block: OpenBlock, index: usize) -> ClosedBlock;
    //qqf-code-end
    fn fsm_initialize(&self, open_block: OpenBlock) -> StatusOfFSM;
    fn fsm_pause(&self, executed_block: ExecutedBlock, index: usize) -> StatusOfFSM;
    fn fsm_execute(&self, executed_block: ExecutedBlock, index: usize) -> StatusOfFSM;
    fn fsm_finalize(&self, executed_block: ExecutedBlock) -> ClosedBlock;
}

impl FSM for Executor {
    fn into_fsm(&mut self, open_block: OpenBlock) -> ClosedBlock {
        let mut status = StatusOfFSM::Initialize(open_block);
        loop {
            trace!("executor is at {}", status);
            status = match status {
                StatusOfFSM::Initialize(open_block) => self.fsm_initialize(open_block),
                StatusOfFSM::Pause(executed_block, index) => self.fsm_pause(executed_block, index),
                StatusOfFSM::Execute(executed_block, index) => {
                    self.fsm_execute(executed_block, index)
                }
                StatusOfFSM::Finalize(executed_block) => return self.fsm_finalize(executed_block),
            }
        }
    }
    /*
    fn serial_fsm_execute(&mut self, executed_block: ExecutedBlock, index: usize) -> ClosedBlock {
        
    }
    */
    fn test_deocc_fsm_execute(&self, open_block: OpenBlock, _index: usize){
        let mut executed_block =self.to_executed_block(open_block.clone());
        //把tdg图给拆分成若干子图
        
        let _g = self.tdg.clone();
        //制作每一笔交易的读写集!!!
        
        let dir = Direction::Incoming;
        //tran是g（子图）的index
        //let mut index_vec = Vec::new();
        let iter = self.tdg.node_indices();
        let mut consist_read_set = HashMap::new();
        let mut consist_read_set_h256 = HashMap::new();
        //let mut s_1= " ";
        //let mut s_2= " ";
        for n_index in iter{
            let x = self.tdg.node_weight(n_index.clone()).unwrap().clone();
            //遍历每一个点，生成每一个节点的一致性读集
            let mut transaction_read_set = Vec::new();
            if x.read_set != None{
                //获取该节点的读集
                transaction_read_set = x.read_set.unwrap().clone();
            }
            let mut hash_map = HashMap::new();
            let mut hash_map_h256 = HashMap::new();
            //遍历该节点所有入边的节点，判断他们的写集是否和主节点读集吻合
            let neighbors_node_iter = self.tdg.neighbors_directed(n_index, dir);
            for neighbor in neighbors_node_iter{
                let tran = self.tdg.node_weight(neighbor.clone()).unwrap().clone();
                //主节点读集
                let iter = transaction_read_set.iter();
                //邻居节点写集
                let tran_write_set = tran.write_set.unwrap_or(Vec::default()).clone();
                let tran_write_values = tran.write_values.unwrap_or(Vec::default()).clone();
                //对主节点读集中每一个要读的数据遍历
                //主机节点读集元素
                for string in iter{
                    let mut index= 0 as usize;
                    let iter_neighbor = tran_write_set.iter();
                    //邻居节点写集元素
                    for write_value in iter_neighbor{
                        if string == write_value{
                            hash_map.insert(write_value.clone(), tran_write_values[index.clone()].clone());
                            
                            let x = write_value.clone().split_off(66);
                            let y = tran_write_values[index.clone()].clone().split_off(66).clone();
                            let x_v = hex::decode(x.as_str()).unwrap_or(Vec::new());
                            let y_v = hex::decode(y.as_str()).unwrap_or(Vec::new());
                            let x_u : &[u8] = &x_v;
                            let y_u : &[u8] = &y_v;
                            let x_h = H256::from(x_u);
                            let y_h = H256::from(y_u);
                            hash_map_h256.insert(x_h.clone(), y_h.clone());
                            
                            break;
                        }
                        index = index + 1;
                    }
                }
            }
            consist_read_set.insert(x.tran_id.clone(), hash_map.clone());
            consist_read_set_h256.insert(x.tran_id.clone(), hash_map_h256.clone());
        }
        info!("一致性读集:{:?}",&consist_read_set);
        info!("一致性读集(H256):{:?}",&consist_read_set_h256);
        
        let s = "0000000000000000000000000000000000000000000000000000000000000000";
        //let byte = s.as_bytes();
        //let str_test = s.to_string(); 
        let temp = hex::decode(s);
        let temp_o = temp.unwrap();
        let o : &[u8] = &temp_o;
        //info!("Stirng->Vec<u8>:{:?}",&temp);
        let h = H256::from(o);
        info!("Vec<u8>->H256:{:?}",&h);
        //let mut topo_sort = self.tdg_toposort.clone();
        //Tdg用完以后初始化
        //self.tdg = Graph::default();
        //self.tdg_toposort = Vec::new();
        //info!("开始执行交易了,deocc_g:{:?}",&g);
        if executed_block.body().clone().transactions.is_empty(){
            //return executed_block.close(&(self.sys_config.block_sys_config));
            return;
        }
        //查看有多少交易
        let tran_num = executed_block.body().transactions.len();
        info!("本区块交易个数：{:?}",&tran_num);
        let ten_millis = time::Duration::from_millis(3);

        let _temp_vec = self.tdg_toposort.clone();
        //self.tdg_toposort = Vec::new();
        info!("执行的顺序:{:?}",&_temp_vec);
        
        //info!("未执行前的executed_blcok:{:?}",&executed_block);
        //let only_executed_block = Arc::new(Mutex::new(executed_block));
        //let count = Arc::new(AtomicUsize::new(0));
        let temp_consist_read_set = Arc::new(consist_read_set_h256);
        //let mut handles= Vec::new();
        let iter = _temp_vec.clone().into_iter();
        //info!("process:{:?}",&vec);
        //let only_executed_block = Arc::clone(&only_executed_block);
        //let commit_index = count.clone();
        let consist_read_set_map = temp_consist_read_set.clone();
        //let temp_vec = _temp_vec.clone();
        let conf = self.sys_config.block_sys_config.clone();
        let quota_price = conf.quota_price;
        let economical_model: EconomicalModel = conf.economical_model;
        let start_time = Local::now();
        for index in iter{
            info!("process index:{:?}",&index);
            //开始执行交易
            //生成执行块延长单笔交易的时间
            //let mut _the_executed_block =self.to_executed_block(open_block.clone());
            let consist_read_set_of_itself = consist_read_set_map.get(&index).unwrap_or(&HashMap::new()).clone();
            //插入对应交易的一致性读集合
            executed_block.state.insert_consist_read_set(consist_read_set_of_itself.clone());
            info!("插入对应的一致性读集{:?}",&consist_read_set_of_itself);
            let mut transaction = executed_block.body().transactions[index.clone()].clone();
            if economical_model == EconomicalModel::Charge {
            transaction.gas_price = quota_price;
            }
            let engine = NullEngine::cita();
            executed_block.apply_transaction(&engine, &transaction, &conf);
            //延长交易执行时间
            thread::sleep(ten_millis.clone());
            //executed_block.state.clear();
            //清除原有state一致性依赖集
            executed_block.state.insert_consist_read_set(HashMap::new());
            info!("one transaction end:{:?}",&index);
            //drop(executed_block);
        }
        let end_time = Local::now();
        let duration = end_time.timestamp_millis() - start_time.timestamp_millis();
        info!("交易串行执行时间:{:?}",&duration);
        info!("完成所有交易执行！");
        //提交所有交易修改内容
        //let _only_executed_block = Arc::clone(&only_executed_block);
        //info!("完成提交所有交易修改内容");
        //executed_block.close(conf: &BlockSysConfig)
        //let x = Arc::try_unwrap(only_executed_block).unwrap().into_inner().unwrap();
        //打印temp_storage_cache
        //info!("the cache:{:?}",&executed_block.state.cache());
        
        //打印storage_changes
        let xl = executed_block.state.clone_cache();
        for temp in xl{
            let (_temp, account_entry) = temp;
            let account = account_entry.account.unwrap();
            let storage_changes = account.storage_changes();
            if !storage_changes.is_empty(){
                info!("串行修改结果:{:?}",&storage_changes);
            }  
        }
        
        executed_block.state.commit().expect("failed to commit state trie");
        info!("串行state_root(after commit):{:?}",executed_block.state.root());
        //let closed_block = executed_block.close(&(self.sys_config.block_sys_config));
        //info!("deocc:{:?}",&closed_block);
        //Tdg用完以后初始化
        //self.tdg = Graph::default();
        //closed_block
    }
    //优化过后的并发算法
    fn test_1_deocc_fsm_execute(&mut self, executed_block: ExecutedBlock, open_block: OpenBlock, _index: usize) -> ClosedBlock{
        //把tdg图给拆分成若干子图
        info!("----并发执行----");
        let g = self.tdg.clone();
        //制作每一笔交易的读写集!!!
        
        let dir = Direction::Incoming;
        //tran是g（子图）的index
        //let mut index_vec = Vec::new();
        let iter = self.tdg.node_indices();
        let mut consist_read_set = HashMap::new();
        let mut consist_read_set_h256 = HashMap::new();
        for n_index in iter{
            let x = self.tdg.node_weight(n_index.clone()).unwrap().clone();
            //遍历每一个点，生成每一个节点的一致性读集
            let mut transaction_read_set = Vec::new();
            if x.read_set != None{
                //获取该节点的读集
                transaction_read_set = x.read_set.unwrap().clone();
            }
            let mut hash_map = HashMap::new();
            let mut hash_map_h256 = HashMap::new();
            //遍历该节点所有入边的节点，判断他们的写集是否和主节点读集吻合
            let neighbors_node_iter = self.tdg.neighbors_directed(n_index, dir);
            for neighbor in neighbors_node_iter{
                let tran = self.tdg.node_weight(neighbor.clone()).unwrap().clone();
                //主节点读集
                let iter = transaction_read_set.iter();
                //邻居节点写集
                let tran_write_set = tran.write_set.unwrap_or(Vec::default()).clone();
                let tran_write_values = tran.write_values.unwrap_or(Vec::default()).clone();
                //对主节点读集中每一个要读的数据遍历
                //主机节点读集元素
                for string in iter{
                    let mut index= 0 as usize;
                    let iter_neighbor = tran_write_set.iter();
                    //邻居节点写集元素
                    for write_value in iter_neighbor{
                        if string == write_value{
                            hash_map.insert(write_value.clone(), tran_write_values[index.clone()].clone());
                            
                            let x = write_value.clone().split_off(66);
                            let y = tran_write_values[index.clone()].clone().split_off(66).clone();
                            let x_v = hex::decode(x.as_str()).unwrap_or(Vec::new());
                            let y_v = hex::decode(y.as_str()).unwrap_or(Vec::new());
                            let x_u : &[u8] = &x_v;
                            let y_u : &[u8] = &y_v;
                            let x_h = H256::from(x_u);
                            let y_h = H256::from(y_u);
                            hash_map_h256.insert(x_h.clone(), y_h.clone());
                            
                            break;
                        }
                        index = index + 1;
                    }
                }
            }
            consist_read_set.insert(x.tran_id.clone(), hash_map.clone());
            consist_read_set_h256.insert(x.tran_id.clone(), hash_map_h256.clone());
        }
        info!("一致性读集:{:?}",&consist_read_set);
        info!("一致性读集(H256):{:?}",&consist_read_set_h256);
        
        //let s = "0000000000000000000000000000000000000000000000000000000000000000";
        //let byte = s.as_bytes();
        //let str_test = s.to_string(); 
        //let temp = hex::decode(s);
        //let temp_o = temp.unwrap();
        //let o : &[u8] = &temp_o;
        //info!("Stirng->Vec<u8>:{:?}",&temp);
        //let h = H256::from(o);
        //info!("Vec<u8>->H256:{:?}",&h);
        //let mut topo_sort = self.tdg_toposort.clone();
        //Tdg用完以后初始化
        //self.tdg = Graph::default();
        //self.tdg_toposort = Vec::new();
        //info!("开始执行交易了,deocc_g:{:?}",&g);
        if executed_block.body().clone().transactions.is_empty(){
            return executed_block.close(&(self.sys_config.block_sys_config));
        }
        let transactions_number = executed_block.body().clone().transactions.len();
        info!("块内一共有{:?}笔交易",&transactions_number);
        let mut topo_sort = algo::toposort(&g,None).unwrap();
        info!("topo_sort:{:?}",&topo_sort);
        let mut total_vec_usize = Vec::new();
        //生成无向图gg
        let gg: UnGraph<TransactionInfo , i32>= g.clone().into_edge_type();
        loop{
            if topo_sort.is_empty(){
                break;
            }
            let mut vec_usize = Vec::new();
            let from_node = topo_sort.remove(0);
            let node = g.node_weight(from_node.clone()).unwrap().clone();
            let index = node.tran_id.clone();
            vec_usize.push(index);
            let iter = topo_sort.clone().into_iter();
            let mut index: usize = 0;
            for to_node in iter{
                //根据gg无向图判断是否在一个子图中
                let has_bool = algo::has_path_connecting(&gg, from_node.clone(), to_node.clone(), None);
                if has_bool == true{
                    let tran_index = g.node_weight(to_node.clone()).unwrap().clone().tran_id;
                    vec_usize.push(tran_index);
                    topo_sort.remove(index);
                }
                else{
                    index = index + 1;
                }
            }
            total_vec_usize.push(vec_usize);
        }
        //子图分组
        info!("total_vec:{:?}",&total_vec_usize);
        let _temp_vec = self.tdg_toposort.clone();
        //给后面提交交易的写做topo
        let sort = self.tdg_toposort.clone();
        self.tdg_toposort = Vec::new();
        info!("执行的顺序:{:?}",&_temp_vec);
        
        //info!("未执行前的executed_blcok:{:?}",&executed_block);
        //let only_executed_block = Arc::new(Mutex::new(executed_block));
        //let only_executed_block = Arc::new(executed_block);
        //let count = Arc::new(AtomicUsize::new(0));
        let temp_consist_read_set = Arc::new(consist_read_set_h256);
        //let cache_map = HashMap::new();
        //let total_cache = Arc::new(Mutex::new(cache_map));
        //生成多生产者的通道channel
        let mut cache_channel = HashMap::new();
        let (tx, rx) = channel();
        //let op = temp_consist_read_set;
        //let mut handles= Vec::new();
        let iter = total_vec_usize.clone().into_iter();
        //let start_time = Local::now();
        let ten_millis = time::Duration::from_millis(3);
        //交易池生成
        let n_workers = 4;
        let _n_jobs = 8;
        let pool = ThreadPool::new(n_workers);
        //执行块池
        let mut executed_block_vec = Vec::new();
        let len = total_vec_usize.clone().into_iter();
        for _null in len{
            let executed_block = self.to_executed_block(open_block.clone());
            executed_block_vec.push(executed_block);
        }
        let start_time = Local::now();
        for vec in iter{
            info!("process_sub_graph:{:?}",&vec);
            //time start

            //let only_executed_block = Arc::clone(&only_executed_block);
            //let commit_index = count.clone();
            //let cache_map = Arc::clone(&total_cache);
            let consist_read_set_map = temp_consist_read_set.clone();
            //let temp_vec = _temp_vec.clone();
            let conf = self.sys_config.block_sys_config.clone();
            let quota_price = conf.quota_price;
            let economical_model: EconomicalModel = conf.economical_model;
            let build_executed_block_time =Local::now();
            //let mut executed_block = self.to_executed_block(open_block.clone());
            let mut executed_block = executed_block_vec.pop().unwrap();
            //复制一个channel_tx
            let tx  = tx.clone();
            //time
            let g_time_start = Local::now(); 
            info!("生成一个执行块的时间:{:?}",g_time_start.timestamp_millis() - build_executed_block_time.timestamp_millis());
            info!("front:{:?}",g_time_start.timestamp_millis() - start_time.timestamp_millis());
            pool.execute(move || {
                //let mut executed_block = only_executed_block.lock().unwrap();
                //let consist_read_set_map = temp_consist_read_set.clone();
                let iter = vec.clone().into_iter();
                for index in iter{
                    info!("process index:{:?}",&index);
                    //开始执行交易
                    let operation_start = Local::now();
                    info!("线程开始时间:{:?}",operation_start.timestamp_millis() - start_time.timestamp_millis());
                    //let mut executed_block = only_executed_block.lock().unwrap();
                    //插入对应交易的一致性读集合
                    let consist_read_set_of_itself = consist_read_set_map.get(&index).unwrap_or(&HashMap::new()).clone();
                    executed_block.state.insert_consist_read_set(consist_read_set_of_itself.clone());
                    info!("插入对应的一致性读集{:?}",&consist_read_set_of_itself);
                    let insert_end = Local::now();
                    info!("插入time:{:?}",insert_end.timestamp_millis() - operation_start.timestamp_millis());
                    //let stamp1 = Local::now();
                    let mut transaction = executed_block.body().transactions[index.clone()].clone();
                    //let stamp2 = Local::now();
                    //let duration = stamp2.timestamp_millis() - stamp1.timestamp_millis();
                    //info!("evm时间:{:?}ms",&duration);

                    if economical_model == EconomicalModel::Charge {
                        transaction.gas_price = quota_price;
                    }
                    let engine = NullEngine::cita();
                    executed_block.apply_transaction(&engine, &transaction, &conf);
                    //延长交易执行时间
                    thread::sleep(ten_millis.clone());
                    let executed_time = Local::now();
                    info!("执行时间:{:?}",executed_time.timestamp_millis() - insert_end.timestamp_millis());
                    //info!("当前executed_block情况：{:?}",&executed_block);
                    //提交每一个交易，修改tire（内存）
                    //executed_block.state.commit().expect("failed to commit state trie");
                    //清空缓存，保持每个缓存只有一笔交易的数据
                    //executed_block.state.clear();
                    //清除原有state一致性依赖集
                    executed_block.state.insert_consist_read_set(HashMap::new());
                    info!("one transaction end:{:?}",&index);
                    //drop(executed_block);
                    let self_cache = executed_block.state.clone_cache();
                    //把inex和self_cache传入channel
                    tx.send((index.clone(), self_cache.clone())).unwrap();
                    
                    //let mut x = cache_map.lock().unwrap();
                    let lock_time = Local::now();
                    info!("发送到channel的时间:{:?}",lock_time.timestamp_millis() - executed_time.timestamp_millis());
                    //x.insert(index.clone(),self_cache.clone());
                    //drop(x);
                }
            });
            //let g_time_end = Local::now();
            //let d = g_time_end.timestamp_millis() - g_time_start.timestamp_millis();
            //info!("d:{:?}",&d);

            //handles.push(handle);
        }
        //let start_time = Local::now();
        pool.join();
        /*
        info!("块内交易完成，阻塞");
        for handle in handles {
            handle.join().unwrap();
        }
        */
        //接收channel的信息
        drop(tx);
        for index_cache in rx.iter(){
            //接收channel_rx中的信息
            let (index, cache) = index_cache;
            cache_channel.insert(index.clone(), cache.clone());

        }
        let end_time = Local::now();
        let duration = end_time.timestamp_millis() - start_time.timestamp_millis();
        let tps = transactions_number as f64;
        let tps = tps / duration as f64 * 1000 as f64;
        info!("并发执行{:?}笔交易需要时间:{:?}ms,TPS:{:?}", &transactions_number, &duration, &tps);
        info!("完成所有交易执行！");
        //let total_hash_map = Arc::try_unwrap(total_cache).unwrap().into_inner().unwrap();
        let total_hash_map = cache_channel.clone();
        //定义替换原本executed_block的cache
        let mut real_cache = executed_block.state.clone_cache();
        //let iter  = total_hash_map.clone().into_iter();
        //let sort = self.tdg_toposort.clone();
        let usize_iter = sort.into_iter();
        for index in usize_iter{
            //let (_usize, hash_map) = index.clone();
            let hash_map = total_hash_map.get(&index).unwrap().clone();
            let iter = hash_map.clone().into_iter();
            for x in iter{
                let (address, account_entry) = x.clone();
                if real_cache.contains_key(&address) == true{
                    let old_account_entry = real_cache.get_mut(&address).unwrap();
                    let new_account = account_entry.account.unwrap();
                    //更换nonce
                    let nonce = new_account.nonce().clone();
                    //如果noce大于old的，则取最大的
                    old_account_entry.change_nonce(nonce);
                    //let mut old_storage_changes = old_account.storage_changes().clone();
                    let new_storage_changes = new_account.storage_changes().clone();
                    let iter = new_storage_changes.into_iter();
                    for w in iter{
                        let (k, v) = w.clone();
                        //old_storage_changes.insert(k.clone(), v.clone());
                        //old_account.set_storage(k.clone(), v.clone());
                        old_account_entry.insert_account_storage_changes(k.clone(), v.clone());
                    }
                    //account_entry.account = Some(old_account);
                    //real_cache.insert(address.clone(), account_entry.clone());
                }
                else{
                    real_cache.insert(address.clone(), account_entry.clone());
                }
            }
        }
        info!("完成提交所有交易修改内容");
        //let x = Arc::try_unwrap(only_executed_block).unwrap().into_inner().unwrap();
        //打印temp_storage_cache
        //info!("real_cache:{:?}",&real_cache);
        //executed_block.state.exchange_cache(real_cache);
        let mut _executed_block = self.to_executed_block(open_block.clone());
        //info!("CACHE:{:?}",&real_cache);
        _executed_block.exchange_state_cache(real_cache);
        //info!("state_root(before commit):{:?}",_executed_block.state.root());
        /*
        let print = _executed_block.state.clone_cache();
        for x in print.iter(){
            let (address,account_entry) = x.clone();
            info!("address:{:?}",address);
            account_entry.print_account();
        }
        */
        //打印执行storage change结果
        let xl = _executed_block.state.clone_cache();
        for temp in xl{
            let (_temp, account_entry) = temp;
            let account = account_entry.account.unwrap();
            let storage_changes = account.storage_changes();
            if !storage_changes.is_empty(){
                info!("并行修改结果:{:?}",&storage_changes);
            }  
        }
        //info!("the cache:{:?}",&_executed_block.state.cache());
        _executed_block.state.commit().expect("failed to commit state trie");
        info!("并发state_root(after commit):{:?}",_executed_block.state.root());
        /*
        let print = _executed_block.state.clone_cache();
        for x in print.iter(){
            let (address,account_entry) = x.clone();
            info!("address:{:?}",address);
            account_entry.print_account();
        }
        */
        let closed_block = _executed_block.close(&(self.sys_config.block_sys_config));
        info!("----end----");
        closed_block
    }
    
    fn deocc_fsm_execute(&self, executed_block: ExecutedBlock, _index: usize) -> StatusOfFSM{
        //把tdg图给拆分成若干子图
        let g = self.tdg.clone();
        if executed_block.body().clone().transactions.is_empty(){
            return StatusOfFSM::Finalize(executed_block);
        }
        let mut topo_sort = algo::toposort(&g.clone(),None).unwrap();
        let mut total_vec_usize = Vec::new();
        loop{
            if topo_sort.is_empty(){
                break;
            }
            let mut vec_usize = Vec::new();
            let from_node = topo_sort.remove(0);
            let node = g.node_weight(from_node.clone()).unwrap().clone();
            let index = node.tran_id.clone();
            vec_usize.push(index);
            let iter = topo_sort.clone().into_iter();
            let mut index: usize = 0;
            for to_node in iter{
                let has_bool = algo::has_path_connecting(&g, from_node.clone(), to_node.clone(), None);
                if has_bool == true{
                    let tran_index = g.node_weight(to_node.clone()).unwrap().clone().tran_id;
                    vec_usize.push(tran_index);
                    topo_sort.remove(index);
                }
                else{
                    index = index + 1;
                }
            }
            total_vec_usize.push(vec_usize);
        }
        let only_executed_block = Arc::new(Mutex::new(executed_block));
        let mut handles= Vec::new();
        //创建一个Map记录执行过后的transactioninfo
        let iter = total_vec_usize.clone().into_iter();
        for vec in iter{
            let only_executed_block = Arc::clone(&only_executed_block);
            let conf = self.sys_config.block_sys_config.clone();
            let handle = std::thread::spawn(move || {
                let mut executed_block = only_executed_block.lock().unwrap();
                let iter = vec.clone().into_iter();
                for index in iter{
                    let mut transaction = executed_block.body().transactions[index.clone()].clone();
                    let quota_price = conf.quota_price;
                    let economical_model: EconomicalModel = conf.economical_model;
                    if economical_model == EconomicalModel::Charge {
                        transaction.gas_price = quota_price;
                    }
                    let engine = NullEngine::cita();
                    executed_block.apply_transaction(&engine, &transaction, &conf);
                    //提交每一个交易，修改tire（内存）
                    executed_block.state.commit().expect("failed to commit state trie");
                    //清空缓存，保持每个缓存只有一笔交易的数据
                    executed_block.state.clear();
                }
            });
            handles.push(handle);
        }
        for handle in handles {
            handle.join().unwrap();
        }
        let x = Arc::try_unwrap(only_executed_block).unwrap().into_inner().unwrap();
        StatusOfFSM::Finalize(x)
        //let x = only_executed_block.get_mut().unwrap();
        //x.close(&(self.sys_config.block_sys_config));
        //let result_executed_block = x.into_inner().unwrap();
        //x.close(&(self.sys_config.block_sys_config));
        //let _executed_block = *result_executed_block;
        //*result_executed_block.close(&(self.sys_config.block_sys_config));
        //StatusOfFSM::Finalize(executed_block)
        //close_block
    }
    //真实执行语句
    fn real_into_fsm(&mut self, open_block: OpenBlock) -> ClosedBlock {
        //把openblock中的tdg取出来
        info!("into real fsm");
        let executed_block = self.to_executed_block(open_block);
        let mut status = StatusOfFSM::Execute(executed_block, 0);
        loop {
            trace!("executor is at {}", status);
            status = match status {
                StatusOfFSM::Initialize(open_block) => self.fsm_initialize(open_block),
                StatusOfFSM::Pause(executed_block, index) => self.fsm_pause(executed_block, index),
                StatusOfFSM::Execute(executed_block, index) => {
                    self.deocc_fsm_execute(executed_block, index)
                }
                StatusOfFSM::Finalize(executed_block) => {
                    info!("into-real-end");
                    return self.fsm_finalize(executed_block);
                }
            }
        }
    }
    //预执行的开始语句
    fn pre_into_fsm(&mut self, open_block: OpenBlock) -> BatchingOcc{
        //创建空的读写集合
        let mut rw = RwSet::new();
        let mutex_rw = Mutex::new(rw.clone());
        //test_pre
        let mut _rw_1 = RwSet::new();
        let mut _guard = mutex_rw.lock().unwrap();
        //创建新的batchingocc
        let mut bocc = BatchingOcc::new();
        let mut _bocc_1 = BatchingOcc::new();
        //let transaction_map = HashMap::new();
        //let mut status = StatusOfFSM::Initialize(open_block);
        //标志该节点为主节点
        self.is_prime = true;
        //直接执行
        let mut status = StatusOfFSM::Initialize(open_block);
        let start_time = Local::now();
        loop {
            trace!("executor is at {}", status);
            status = match status {
                StatusOfFSM::Initialize(open_block) => {
                    //self.test_pre_fsm_execute_in_parallel(open_block.clone(), &mut rw_1, &mut bocc_1);
                    self.test_pre_fsm_execute_in_parallel(open_block,&mut rw,&mut bocc)
                }    
                StatusOfFSM::Pause(executed_block, index) => self.fsm_pause(executed_block, index),
                StatusOfFSM::Execute(executed_block, index) => {
                    self.pre_fsm_execute(executed_block, index, &mut rw)
                }
                StatusOfFSM::Finalize(executed_block) => {
                    //info!("预执行结束,所有交易已经被执行:{:?}",executed_block.number());
                    let _closed_block = self.fsm_finalize(executed_block);
                    //info!("预执行后：{:?}",&closed_block);
                    break;
                    //return self.fake_fsm_finalize(executed_block)
                }
            }
        }
        let end_time = Local::now();
        info!("预执行执行时间{:?}",end_time.timestamp_millis() - start_time.timestamp_millis());
        bocc
    }
    //并发执行块中所有交易
    fn pre_fsm_execute_in_parallel(&self, open_block: OpenBlock, rw_real: &mut RwSet ,bocc_real: &mut BatchingOcc) -> StatusOfFSM {
        //开始时间戳
        let start_time = Local::now();
        //初始化rw,让他成为共享可变变量
        //唯一制定运行完所有交易的块 executed_block
        let executed_block = self.to_executed_block(open_block.clone());
        let  new_rw = RwSet::new();
        let rw = Arc::new(Mutex::new(new_rw));
        let num = executed_block.body().transactions().len() as usize;
        //可以并发访问的executed_block
        let only_executed_block = Arc::new(Mutex::new(executed_block));
        let executed_block = self.to_executed_block(open_block.clone());
        let mut transaction_map = HashMap::new();
        //topo_sort
        let mut tdg_toposort = Vec::new();
        //vec<index>把map中index的交易给执行了
        //let mut index_vec = Vec::new();
        //运用下标把body中所有的交易塞入交易hashmap中
        let mut transaction_index : usize = 0;
        for _ in 0..num{
            transaction_map.insert(transaction_index.clone() ,executed_block.body().transactions[transaction_index].clone());
            transaction_index += 1;
        }
        info!("tran_number:{:?}",transaction_map.len());
        //判断是否为第一次执行该loop
        //let mut is_first_loop = true;
        let mut bocc = BatchingOcc::new();
        //创造线程池
        let n_work = 4;
        let pool = ThreadPool::new(n_work);
        //loop直到map为空
        loop{
            if transaction_map.is_empty(){
                break;
            }
            //一整批occ时间戳
            let executed_start_stamp = Local::now();
            //给trans_map的Iter
            let iter = transaction_map.clone().into_iter();
            //处理transaction_map中每一笔交易
            for t in iter{
                //处理单笔交易的executedblock:e
                let mut e = self.to_executed_block(open_block.clone());
                let (index,transaction) = t;
                let mut transaction = transaction;
                let conf = self.sys_config.block_sys_config.clone();
                //let mut transaction = executed_block.body().transactions[index - 1].clone();
                let quota_price = conf.quota_price;
                let economical_model: EconomicalModel = conf.economical_model;
                if economical_model == EconomicalModel::Charge {
                    transaction.gas_price = quota_price;
                }
                let engine = NullEngine::cita();
                let rw = Arc::clone(&rw);
                pool.execute(move || {
                    e.apply_transaction(&engine, &transaction, &conf);
                    let map = e.state.clone_cache();
                    info!("deal {}-th transaction",index);
                    let iter = map.iter();
                    let mut guard = rw.lock().unwrap();
                    for n in iter{
                        let ( _k, v) = n;
                        //info!("contract_address:{:?}",_k);
                        if !v.account.is_none(){
                            let account = v.account().unwrap().clone();
                            //info!("Account{:?}",account);
                            let read = account.storage_cache();
                            let write = account.storage_changes_get();
                            if !read.is_empty(){
                                guard.insert_read_set(index.clone(), read);
                                info!("成功插入读");
                            }
                            if !write.is_empty(){
                                guard.insert_write_set(index.clone(), write);
                                info!("成功插入写");
                            }
                        }     
                    }
                });
                //handles.push(handle);
            }
            pool.join();
            //执行结束时间戳
            let executed_end_stamp = Local::now();
            info!("occ该批交易耗费时间：{:?}ms",executed_end_stamp.timestamp_millis() - executed_start_stamp.timestamp_millis());
            //info!("遍历account结束");
            //获取读写集
            let x = rw.lock().unwrap().clone();
            let read_set = x.read_set.clone(); 
            let write_set = x.write_set.clone();
            //把读写集合塞进vec<TransactionInfo>中
            let mut transaction_vec = Vec::new();
            //从transaction_map中获取执行的交易
            let iter = transaction_map.clone().into_iter();
            for tx in iter{
                let (index, _transcation) = tx.clone();
                let mut transaction_info = TransactionInfo::new(index.clone(),None,None,None,None);
                //读集
                let tx_read_set = read_set.get(&index);
                if tx_read_set != None{
                    let tx_read_set = tx_read_set.unwrap();
                    let iter_read = tx_read_set.iter();
                    //向transactionInfo添加读集合
                    let mut read_set_vec = Vec::new();
                    let mut read_values_vec = Vec::new();
                    for read in iter_read{   
                        let (key, value) = read;
                        //transaction_info.read_set.unwrap().push(key.clone());
                        //transaction_info.read_values.unwrap().push(value.clone());
                        read_set_vec.push(key.clone());
                        read_values_vec.push(value.clone());
                    }
                    transaction_info.read_set = Some(read_set_vec);
                    transaction_info.read_values = Some(read_values_vec);
                }
                //写集
                let tx_write_set = write_set.get(&index);
                if tx_write_set != None{
                    let tx_write_set = tx_write_set.unwrap();
                    let iter_write = tx_write_set.iter();
                    //向transactionInfo添加读集合
                    let mut write_set_vec = Vec::new();
                    let mut write_values_vec = Vec::new();
                    for write in iter_write{   
                        let (key, value) = write;
                        write_set_vec.push(key.clone());
                        write_values_vec.push(value.clone());
                    }
                    transaction_info.write_set = Some(write_set_vec);
                    transaction_info.write_values = Some(write_values_vec);
                }
                transaction_vec.push(transaction_info);
            }
            //info!("执行后的traninfo读写集合:{:?}",&transaction_vec);
            //let mut bocc = BatchingOcc::new();
            //设置获取读写集结束后的时间戳
            let rw_set_stamp = Local::now();
            info!("读写集获取后的时间:{:?}ms",rw_set_stamp.timestamp_millis() - executed_end_stamp.timestamp_millis());
            //获取完读写集，准备制作冲突图
            let mut cg = bocc.construct_conflict_graph(transaction_vec);
            //获得中止节点集合（以cg的顶点index为vec的值）
            let abort = bocc.find_abort_transaction_info_set(cg.clone());
            //info!("中止的节点{:?}",&abort);
            //从冲突图cg中删除中止的交易abort
            let iter = abort.clone().into_iter();
            for node in iter{
                //从冲突图中依据weight来寻找index
                let iter = cg.node_indices().clone();
                let mut vid : petgraph::prelude::NodeIndex = NodeIndex::new(0);
                for index in iter{
                    let x = cg.node_weight(index).unwrap().clone();
                    if x.tran_id == node.tran_id {
                        vid = index;
                        break;
                    }
                }
                //开始从component移除minvid的边
                //移除出度边
                let dir = Direction::Outgoing;
                //就离谱，不clone拿出来的component不能改，这你能信？
                let component_clone =cg.clone();
                //let iter_out = component.neighbors_directed(minvid,dir);
                let iter_out = component_clone.neighbors_directed(vid,dir);
                for n in iter_out{
                    let edge_index = cg.find_edge(vid, n);
                    //一定找得到minvid->n的边
                    cg.remove_edge(edge_index.unwrap());
                }
                //开始移除入度边
                let dir = Direction::Incoming;
                let component_clone =cg.clone();
                let iter_in = component_clone.neighbors_directed(vid,dir);
                for n in iter_in{
                    let edge_index = cg.find_edge(n, vid);
                    //一定能找到n->minvid的边
                    cg.remove_edge(edge_index.unwrap());
                }
                //开始从component移除minvid的节点
                let _tx = cg.node_weight(vid).unwrap().clone();
                cg.remove_node(vid);
            }
            //完成删除        
            //得到拓扑排序 sorted里面是cg拓扑的结果，以cg的节点标号为序
            let sorted = algo::toposort(&cg,None).unwrap();
            if !sorted.is_empty(){
                let s = sorted.clone();
                let iter = s.into_iter();
                for n in iter{
                    let x = cg.node_weight(n).unwrap().clone();
                    //以交易下标为序
                    tdg_toposort.push(x.tran_id);
                }
            }
            //vec<index>是按照topo顺序执行的交易序列，把map中index为下标的交易给执行了
            let mut index_vec = Vec::new();
            for node in sorted{
                //node代表的交易tx
                let commit_tx = cg.node_weight(node).unwrap().clone();
                //获取其id，也就是body的下标index
                let index = commit_tx.tran_id.clone();
                //把commit_tx插入transaction-info_map
                //transaction_info_map.insert(index.clone(), commit_tx.clone());
                index_vec.push(index);
            }
            //线程的向量
            let mut handles= Vec::new();
            //创建一个Map记录执行过后的transactioninfo
            let transaction_info_hashmap =Arc::new(Mutex::new(HashMap::new()));
            //对index-vex中的交易执行
            let iter = index_vec.clone().into_iter();
            for index in iter{
                let transaction_info_hashmap = Arc::clone(&transaction_info_hashmap);
                let only_executed_block = Arc::clone(&only_executed_block);
                let conf = self.sys_config.block_sys_config.clone();
                //记录执行过后的读写集合
                let mut executed_block_cache = HashMap::new();
                let handle = std::thread::spawn(move || {
                    let mut executed_block = only_executed_block.lock().unwrap();
                    let mut transaction_info_hashmap = transaction_info_hashmap.lock().unwrap();
                    //let mut transaction = transaction;
                    let mut transaction = executed_block.body().transactions[index.clone()].clone();
                    let quota_price = conf.quota_price;
                    let economical_model: EconomicalModel = conf.economical_model;
                    if economical_model == EconomicalModel::Charge {
                        transaction.gas_price = quota_price;
                    }
                    let engine = NullEngine::cita();
                    executed_block.apply_transaction(&engine, &transaction, &conf);
                    //获得单笔交易执行后的读写集合
                    executed_block_cache = executed_block.state.clone_cache();
                    //提交每一个交易，修改tire（内存）
                    executed_block.state.commit().expect("failed to commit state trie");
                    //清空缓存，保持每个缓存只有一笔交易的数据
                    executed_block.state.clear();
                    //更新自身读写集
                    let iter = executed_block_cache.clone().into_iter();
                    let mut transaction_info = TransactionInfo::new(index.clone(),None,None,None,None);
                    let mut read_set = Vec::new();
                    let mut read_value = Vec::new();
                    let mut write_set = Vec::new();
                    let mut write_value = Vec::new();
                    //开始遍历每一个address的account
                    for n in iter{
                        let (_address, account_entry) = n.clone();
                        let account = account_entry.account().unwrap().clone();
                        let read = account.storage_cache();
                        let write = account.storage_changes_get();
                        //塞入读集合
                        
                        for r in read.iter(){
                            let (key,value) = r;
                            read_set.push(key.clone());
                            read_value.push(value.clone());
                            //info!("insert read_k:{:?} read_v{:?}",key.clone(),value.clone());
                        }
                        //塞入写集合
                        for w in write.iter(){
                            let (key,value) = w;
                            write_set.push(key.clone());
                            write_value.push(value.clone());
                            //info!("insert write_k:{:?} write_v{:?}",key.clone(),value.clone());
                        }
                    }
                    //塞进transactioninfo
                    transaction_info.read_set = Some(read_set);
                    transaction_info.read_values = Some(read_value);
                    transaction_info.write_set = Some(write_set);
                    transaction_info.write_values = Some(write_value);
                    //加入已经执行完的交易队列中（transactioninfo）
                    transaction_info_hashmap.insert(index.clone(), transaction_info.clone());
                });
                handles.push(handle);
            }
            for handle in handles {
                handle.join().unwrap();
            }
            let iter = index_vec.clone().into_iter();
            let map = transaction_info_hashmap.lock().unwrap().clone();
            for index in iter{
                let tran_info = map.get(&index).clone().unwrap();
                //提交交易至TDG图中
                bocc.commit_transaction_info(tran_info.clone(), 1);
                info!("{:?} in tdg",tran_info.tran_id);
                //将执行过的交易从队列中删除
                transaction_map.remove(&index);
            }
            //transaction_map剩下的都是没有执行过的交易
            //正常交易中在前方excuted已经执行过交易了
            //这时候则需要回滚中止交易txs（提交过的交易已经被remove出去了）
            *rw_real = x.clone();
            bocc.set_toposort(tdg_toposort.clone());
            *bocc_real = bocc.clone();
        }
        info!("topo:{:?}",tdg_toposort);
        //运行完所有交易直接结束
        let x = Arc::try_unwrap(only_executed_block).unwrap().into_inner().unwrap();
        //结束时间
        let end_time =  Local::now();
        info!("消耗时间（修改之前）:{:?}ms",end_time.timestamp_millis() - start_time.timestamp_millis());
        StatusOfFSM::Finalize(x)
    }
    //并发执行块中所有交易
    fn test_pre_fsm_execute_in_parallel(&self, open_block: OpenBlock, rw_real: &mut RwSet ,bocc_real: &mut BatchingOcc) -> StatusOfFSM {
        //
        let start_time = Local::now();
        //初始化rw,让他成为共享可变变量
        //唯一制定运行完所有交易的块 executed_block
        let executed_block = self.to_executed_block(open_block.clone());
        let  new_rw = RwSet::new();
        let rw = Arc::new(Mutex::new(new_rw));
        let num = executed_block.body().transactions().len() as usize;
        //可以并发访问的executed_block
        let only_executed_block = Arc::new(Mutex::new(executed_block));
        let executed_block = self.to_executed_block(open_block.clone());
        let mut transaction_map = HashMap::new();
        //topo_sort
        let mut tdg_toposort = Vec::new();
        //vec<index>把map中index的交易给执行了
        //let mut index_vec = Vec::new();
        //运用下标把body中所有的交易塞入交易hashmap中
        let mut transaction_index : usize = 0;
        for _ in 0..num{
            transaction_map.insert(transaction_index.clone() ,executed_block.body().transactions[transaction_index].clone());
            transaction_index += 1;
        }
        info!("tran_number:{:?}",transaction_map.len());
        //判断是否为第一次执行该loop
        //let mut is_first_loop = true;
        let mut bocc = BatchingOcc::new();
        //创造线程池
        let n_work = 4;
        let pool = ThreadPool::new(n_work);
        //设置一个cache记录交易写过的写集
        let mut cache = HashMap::new();
        //loop直到map为空
        loop{
            if transaction_map.is_empty(){
                break;
            }
            //一整批occ时间戳
            let executed_start_stamp = Local::now();
            //给trans_map的Iter
            let iter = transaction_map.clone().into_iter();
            let _arc_cache = Arc::new(cache.clone());
            //处理transaction_map中每一笔交易
            for t in iter{
                //处理单笔交易的executedblock:e
                let mut e = self.to_executed_block(open_block.clone());
                let (index,transaction) = t;
                let mut transaction = transaction;
                let conf = self.sys_config.block_sys_config.clone();
                //let mut transaction = executed_block.body().transactions[index - 1].clone();
                let quota_price = conf.quota_price;
                let economical_model: EconomicalModel = conf.economical_model;
                if economical_model == EconomicalModel::Charge {
                    transaction.gas_price = quota_price;
                }
                let engine = NullEngine::cita();
                let rw = Arc::clone(&rw);
                let mut temp_hash = HashMap::new();
                temp_hash.insert(0, cache.clone());
                let arc_hash = Arc::new(temp_hash.clone());
                info!("开始执行批处理交易");
                pool.execute(move || {
                    //info!("插入前前前的cache:{:?}",&arc_hash);
                    let mut _cache = arc_hash.get(&0).unwrap().clone();
                    e.state.insert_consist_read_set(_cache.clone());
                    info!("插入的cache:{:?}",&_cache);
                    e.apply_transaction(&engine, &transaction, &conf);
                    let map = e.state.clone_cache();
                    info!("deal {}-th transaction",index);
                    let iter = map.iter();
                    let mut guard = rw.lock().unwrap();
                    for n in iter{
                        let ( _k, v) = n;
                        //info!("contract_address:{:?}",_k);
                        if !v.account.is_none(){
                            //记录合约地址

                            let account = v.account().unwrap().clone();
                            //info!("Account{:?}",account);
                            let read = account.storage_cache();
                            let write = account.storage_changes_get();
                            if !read.is_empty(){
                                guard.insert_read_set(index.clone(), read);
                                info!("成功插入读");
                            }
                            if !write.is_empty(){
                                guard.insert_write_set(index.clone(), write);
                                info!("成功插入写");
                            }
                        }     
                    }
                });
                //handles.push(handle);
            }
            pool.join();
            //执行结束时间戳
            let executed_end_stamp = Local::now();
            info!("occ该批交易耗费时间：{:?}ms",executed_end_stamp.timestamp_millis() - executed_start_stamp.timestamp_millis());
            /*
            for handle in handles {
                handle.join().unwrap();
            }
            */
            //info!("遍历account结束");
            //获取读写集
            let x = rw.lock().unwrap().clone();
            let read_set = x.read_set.clone(); 
            let write_set = x.write_set.clone();
            //把读写集合塞进vec<TransactionInfo>中
            let mut transaction_vec = Vec::new();
            //从transaction_map中获取执行的交易
            let iter = transaction_map.clone().into_iter();
            for tx in iter{
                let (index, _transcation) = tx.clone();
                let mut transaction_info = TransactionInfo::new(index.clone(),None,None,None,None);
                //读集
                let tx_read_set = read_set.get(&index);
                if tx_read_set != None{
                    let tx_read_set = tx_read_set.unwrap();
                    let iter_read = tx_read_set.iter();
                    //向transactionInfo添加读集合
                    let mut read_set_vec = Vec::new();
                    let mut read_values_vec = Vec::new();
                    for read in iter_read{   
                        let (key, value) = read;
                        //transaction_info.read_set.unwrap().push(key.clone());
                        //transaction_info.read_values.unwrap().push(value.clone());
                        read_set_vec.push(key.clone());
                        read_values_vec.push(value.clone());
                    }
                    transaction_info.read_set = Some(read_set_vec);
                    transaction_info.read_values = Some(read_values_vec);
                }
                //写集
                let tx_write_set = write_set.get(&index);
                if tx_write_set != None{
                    let tx_write_set = tx_write_set.unwrap();
                    let iter_write = tx_write_set.iter();
                    //向transactionInfo添加读集合
                    let mut write_set_vec = Vec::new();
                    let mut write_values_vec = Vec::new();
                    for write in iter_write{   
                        let (key, value) = write;
                        write_set_vec.push(key.clone());
                        write_values_vec.push(value.clone());
                    }
                    transaction_info.write_set = Some(write_set_vec);
                    transaction_info.write_values = Some(write_values_vec);
                }
                transaction_vec.push(transaction_info);
            }
            //info!("执行后的traninfo读写集合:{:?}",&transaction_vec);
            //let mut bocc = BatchingOcc::new();
            //设置获取读写集结束后的时间戳
            let rw_set_stamp = Local::now();
            info!("读写集获取后的时间:{:?}ms",rw_set_stamp.timestamp_millis() - executed_end_stamp.timestamp_millis());
            //获取完读写集，准备制作冲突图
            let mut cg = bocc.construct_conflict_graph(transaction_vec.clone());
            //获得中止节点集合（以cg的顶点index为vec的值）
            let abort = bocc.find_abort_transaction_info_set(cg.clone());
            //info!("中止的节点{:?}",&abort);
            //从冲突图cg中删除中止的交易abort
            let iter = abort.clone().into_iter();
            for node in iter{
                //从冲突图中依据weight来寻找index
                let iter = cg.node_indices().clone();
                let mut vid : petgraph::prelude::NodeIndex = NodeIndex::new(0);
                for index in iter{
                    let x = cg.node_weight(index).unwrap().clone();
                    if x.tran_id == node.tran_id {
                        vid = index;
                        break;
                    }
                }
                //开始从component移除minvid的边
                //移除出度边
                let dir = Direction::Outgoing;
                //就离谱，不clone拿出来的component不能改，这你能信？
                let component_clone =cg.clone();
                //let iter_out = component.neighbors_directed(minvid,dir);
                let iter_out = component_clone.neighbors_directed(vid,dir);
                for n in iter_out{
                    let edge_index = cg.find_edge(vid, n);
                    //一定找得到minvid->n的边
                    cg.remove_edge(edge_index.unwrap());
                }
                //开始移除入度边
                let dir = Direction::Incoming;
                let component_clone =cg.clone();
                let iter_in = component_clone.neighbors_directed(vid,dir);
                for n in iter_in{
                    let edge_index = cg.find_edge(n, vid);
                    //一定能找到n->minvid的边
                    cg.remove_edge(edge_index.unwrap());
                }
                //开始从component移除minvid的节点
                let _tx = cg.node_weight(vid).unwrap().clone();
                cg.remove_node(vid);
            }
            //完成删除        
            //得到拓扑排序 sorted里面是cg拓扑的结果，以cg的节点标号为序
            let sorted = algo::toposort(&cg,None).unwrap();
            if !sorted.is_empty(){
                let s = sorted.clone();
                let iter = s.into_iter();
                for n in iter{
                    let x = cg.node_weight(n).unwrap().clone();
                    //以交易下标为序
                    tdg_toposort.push(x.tran_id);
                }
            }
            //vec<index>是按照topo顺序执行的交易序列，把map中index为下标的交易给执行了
            let mut index_vec = Vec::new();
            for node in sorted{
                //node代表的交易tx
                let commit_tx = cg.node_weight(node).unwrap().clone();
                //获取其id，也就是body的下标index
                let index = commit_tx.tran_id.clone();
                //把commit_tx插入transaction-info_map
                //transaction_info_map.insert(index.clone(), commit_tx.clone());
                index_vec.push(index);
            }
            
            let mut map = HashMap::new();
            let iter = transaction_vec.iter();
            for tran_info in iter{
                map.insert(tran_info.tran_id.clone(),tran_info.clone());
            }
            let iter = index_vec.clone().into_iter();
            for index in iter{
                let tran_info = map.get(&index).clone().unwrap().clone();
                //提交交易至TDG图中
                bocc.commit_transaction_info(tran_info.clone(), 1);
                info!("{:?} in tdg",tran_info.tran_id);
                //把该交易的写集写入cache
                let key_set_iter = tran_info.write_set.unwrap().clone();
                let write_set_iter = key_set_iter.clone().into_iter();
                let value_set = tran_info.write_values.unwrap().clone();
                let mut key_index = 0;
                for key in write_set_iter{
                    let key_string = key.clone().split_off(66);
                    let value_string = value_set[key_index.clone()].clone().split_off(66);
                    let x_v = hex::decode(key_string.as_str()).unwrap_or(Vec::new());
                    let y_v = hex::decode(value_string.as_str()).unwrap_or(Vec::new());
                    let x_u : &[u8] = &x_v;
                    let y_u : &[u8] = &y_v;
                    let x_h = H256::from(x_u);
                    let y_h = H256::from(y_u);
                    cache.insert(x_h.clone(), y_h.clone());
                    key_index = key_index + 1;
                }
                //将执行过的交易从队列中删除
                transaction_map.remove(&index);
            }
            info!("cache:{:?}",&cache);
            //康康生成的tdg
            //info!("康康生成的tdg");
            //bocc.get_tdg_transactioninfo();
            //transaction_map剩下的都是没有执行过的交易
            //正常交易中在前方excuted已经执行过交易了
            //这时候则需要回滚中止交易txs（提交过的交易已经被remove出去了）
            *rw_real = x.clone();
            bocc.set_toposort(tdg_toposort.clone());
            *bocc_real = bocc.clone();
        }
        info!("topo:{:?}",tdg_toposort);
        //运行完所有交易直接结束
        let x = Arc::try_unwrap(only_executed_block).unwrap().into_inner().unwrap();
        let end_time = Local::now();
        info!("消耗时间（修改后）:{:?}ms",end_time.timestamp_millis() - start_time.timestamp_millis());
        StatusOfFSM::Finalize(x)
    }
    /*
    fn pre_fsm_execute_in_parallel(&self, open_block: OpenBlock, rw_real: &mut RwSet) -> StatusOfFSM {
        //初始化rw,让他成为共享可变变量
        let executed_block = self.to_executed_block(open_block.clone());
        let  new_rw = RwSet::new();
        let rw = Arc::new(Mutex::new(new_rw));
        let num = executed_block.body().transactions().len() as usize;
        let mut transaction_map = HashMap::new();
        //运用下标把body中所有的交易塞入交易hashmap中
        let mut transaction_index : usize = 0;
        for _ in 0..num{
            transaction_map.insert(transaction_index.clone() ,executed_block.body().transactions[transaction_index].clone());
            transaction_index += 1;
            info!("tran_hashmap:{:?}",transaction_map);
        }
        //看看能不能锁住其他进程m
        //let m = Arc::new(Mutex::new(0));
        //给trans_map的Iter
        let iter = transaction_map.clone().into_iter();
        //线程的向量
        let mut handles= Vec::new();
        //处理body中每一笔交易
        for t in iter{
            let mut e = self.to_executed_block(open_block.clone());
            let (index,transaction) = t;
            let mut transaction = transaction;
            let conf = self.sys_config.block_sys_config.clone();
            //let mut transaction = executed_block.body().transactions[index - 1].clone();
            let quota_price = conf.quota_price;
            let economical_model: EconomicalModel = conf.economical_model;
            if economical_model == EconomicalModel::Charge {
                transaction.gas_price = quota_price;
            }
            let engine = NullEngine::cita();
            let rw = Arc::clone(&rw);
            let handle = std::thread::spawn(move || {
                e.apply_transaction(&engine, &transaction, &conf);
                let map = e.state.clone_cache();
                info!("deal {}-th transaction",index);
                let iter = map.iter();
                let mut guard = rw.lock().unwrap();
                for n in iter{
                    let ( _k, v) = n;
                    //info!("address:{:?}",k);
                    let account = v.account().unwrap().clone();
                    //info!("Account{:?}",account);
                    let read = account.storage_cache();
                    let write = account.storage_changes_get();
                    if !read.is_empty(){
                        guard.insert_read_set(index.clone(), read); 
                    }
                    if !write.is_empty(){
                        guard.insert_write_set(index.clone(), write);
                    }     
                }
            });
            handles.push(handle);
        }
        for handle in handles {
            handle.join().unwrap();
        }
        //获取读写集
        let x = rw.lock().unwrap().clone();
        //dorp(rw);
        *rw_real = x.clone();
        //info!("read_fsm:{:?}",x.read_set);
        //info!("write_fsm:{:?}",x.write_set);
        //StatusOfFSM::Pause(executed_block, index)
        //运行完所有交易直接结束
        StatusOfFSM::Finalize(executed_block)
    }
    */
    //执行每一次交易都clear缓存的
    fn pre_fsm_execute(&self, mut executed_block: ExecutedBlock, index: usize, rw: &mut RwSet) -> StatusOfFSM {
        let conf = self.sys_config.block_sys_config.clone();
        let mut transaction = executed_block.body().transactions[index - 1].clone();
        let quota_price = conf.quota_price;
        let economical_model: EconomicalModel = conf.economical_model;
        if economical_model == EconomicalModel::Charge {
            transaction.gas_price = quota_price;
        }
        executed_block.apply_transaction(&*self.engine, &transaction, &conf);
        //获取state信息，并将其清空 
        let map = executed_block.state.clone_cache();
        let iter = map.iter();
        for n in iter{
            let ( k, v) = n;
            //info!("address:{:?}",k);
            let account = v.account().unwrap().clone();
            //info!("Account{:?}",account);
            let read = account.storage_cache();
            let write = account.storage_changes_get();
            let _address_string = &hex::encode(k);
            if !read.is_empty(){
               rw.insert_read_set(index.clone(), read); 
               //info!("插入读成功,address:{:?}",address_string);
            }
            if !write.is_empty(){
                rw.insert_write_set(index.clone(), write);
                //info!("插入写成功,address:{:?}",address_string);
            }     
        }
        //只更改自身db 不上传至trie
        ///executed_block.state.commit().expect("failed to commit state trie");
        //清除缓存,得到每一笔交易独立的读写集
        //executed_block.state.clear();
        info!("clear");
        StatusOfFSM::Pause(executed_block, index)
    }
    //虚假提交函数
    fn fake_fsm_finalize(&self, executed_block: ExecutedBlock){
        info!("在此处获取读写集:{:?}",executed_block.clone());
    }
    //qqf-code-end
    fn fsm_initialize(&self, open_block: OpenBlock) -> StatusOfFSM {
        let executed_block = self.to_executed_block(open_block);
        StatusOfFSM::Pause(executed_block, 0)
    }

    fn fsm_pause(&self, executed_block: ExecutedBlock, index: usize) -> StatusOfFSM {
        match self.fsm_req_receiver.try_recv() {
            None => {
                if index == executed_block.body().transactions().len() {
                    StatusOfFSM::Finalize(executed_block)
                } else {
                    StatusOfFSM::Execute(executed_block, index + 1)
                }
            }
            Some(open_block) => {
                if executed_block.header().is_equivalent(&open_block.header()) {
                    StatusOfFSM::Pause(executed_block, index)
                } else {
                    StatusOfFSM::Initialize(open_block)
                }
            }
        }
    }

    fn fsm_execute(&self, mut executed_block: ExecutedBlock, index: usize) -> StatusOfFSM {
        let conf = self.sys_config.block_sys_config.clone();
        let mut transaction = executed_block.body().transactions[index - 1].clone();
        let quota_price = conf.quota_price;
        let economical_model: EconomicalModel = conf.economical_model;
        if economical_model == EconomicalModel::Charge {
            transaction.gas_price = quota_price;
        }

        executed_block.apply_transaction(&*self.engine, &transaction, &conf);

        StatusOfFSM::Pause(executed_block, index)
    }

    fn fsm_finalize(&self, mut executed_block: ExecutedBlock) -> ClosedBlock {
        // commit changed-accounts into trie structure
        executed_block
            .state
            .commit()
            .expect("failed to commit state trie");
        let closed_block = executed_block.close(&(self.sys_config.block_sys_config));
        //info!("closed_block:{:?}",&closed_block);
        closed_block
    }
}

#[cfg(test)]
mod tests {
    use super::ExecutedBlock;
    use crate::libexecutor::block::OpenBlock;
    use crate::libexecutor::executor::Executor;
    use crate::libexecutor::fsm::{StatusOfFSM, FSM};
    use crate::tests::helpers::{
        create_block, generate_block_body, generate_block_header, generate_contract, init_executor,
        init_executor2,
    };
    use cita_crypto::{CreateKey, KeyPair};
    use cita_types::Address;
    use std::thread;
    use std::time::Duration;

    fn generate_empty_block() -> OpenBlock {
        let block_body = generate_block_body();
        let mut block_header = generate_block_header();
        block_header.set_number(1);
        OpenBlock {
            body: block_body,
            header: block_header,
        }
    }

    fn generate_block(executor: &Executor, txs: u32) -> OpenBlock {
        let keypair = KeyPair::gen_keypair();
        let privkey = keypair.privkey();
        let data = generate_contract();
        create_block(&executor, Address::from(0), &data, (0, txs), &privkey)
    }

    // transit and commit state root
    fn transit(executor: &mut Executor, status: StatusOfFSM) -> StatusOfFSM {
        let new_status = match status {
            StatusOfFSM::Initialize(open_block) => executor.fsm_initialize(open_block),
            StatusOfFSM::Pause(executed_block, iter) => executor.fsm_pause(executed_block, iter),
            StatusOfFSM::Execute(executed_block, iter) => {
                executor.fsm_execute(executed_block, iter)
            }
            StatusOfFSM::Finalize(_executed_block) => unimplemented!(),
        };
        match new_status {
            StatusOfFSM::Initialize(open_block) => StatusOfFSM::Initialize(open_block),
            StatusOfFSM::Pause(mut executed_block, iter) => {
                executed_block.state.commit().expect("commit state");
                StatusOfFSM::Pause(executed_block, iter)
            }
            StatusOfFSM::Execute(mut executed_block, iter) => {
                executed_block.state.commit().expect("commit state");
                StatusOfFSM::Execute(executed_block, iter)
            }
            StatusOfFSM::Finalize(mut executed_block) => {
                executed_block.state.commit().expect("commit state");
                StatusOfFSM::Finalize(executed_block)
            }
        }
    }

    fn transit_and_assert(
        executor: &mut Executor,
        status_from: StatusOfFSM,
        expect_to: StatusOfFSM,
    ) -> (StatusOfFSM, ExecutedBlock) {
        let status_to = transit(executor, status_from);
        assert_eq!(format!("{}", expect_to), format!("{}", status_to),);

        let executed_block = match expect_to {
            StatusOfFSM::Initialize(_open_block) => unimplemented!(),
            StatusOfFSM::Pause(executed_block, _iter) => executed_block,
            StatusOfFSM::Execute(executed_block, _iter) => executed_block,
            StatusOfFSM::Finalize(executed_block) => executed_block,
        };
        (status_to, executed_block)
    }

    #[test]
    fn test_fsm_initialize() {
        let executor = init_executor();
        let open_block = generate_empty_block();

        {
            let executed_block = executor.to_executed_block(open_block.clone());
            let status_after_init = executor.fsm_initialize(open_block.clone());
            assert_eq!(
                format!("{}", StatusOfFSM::Pause(executed_block, 0)),
                format!("{}", status_after_init)
            );
        }

        {
            let executed_block = executor.to_executed_block(open_block.clone());
            let executed_block_clone = executor.to_executed_block(open_block.clone());
            let status_after_pause_2 = executor.fsm_pause(executed_block, 2);
            assert_eq!(
                format!("{}", StatusOfFSM::Execute(executed_block_clone, 2 + 1)),
                format!("{}", status_after_pause_2)
            );
        }

        {
            let executed_block = executor.to_executed_block(open_block.clone());
            let executed_block_clone = executor.to_executed_block(open_block.clone());
            let status_after_pause_200 = executor.fsm_pause(executed_block, 200);
            assert_eq!(
                format!("{}", StatusOfFSM::Finalize(executed_block_clone)),
                format!("{}", status_after_pause_200)
            );
        }
    }

    #[test]
    fn test_fsm_pause_recv_diff_empty_block() {
        let (fsm_req_sender, fsm_req_receiver) = crossbeam_channel::unbounded();
        let (fsm_resp_sender, _fsm_resp_receiver) = crossbeam_channel::unbounded();
        let (_command_req_sender, command_req_receiver) = crossbeam_channel::bounded(0);
        let (command_resp_sender, _command_resp_receiver) = crossbeam_channel::bounded(0);
        let executor = init_executor2(
            fsm_req_receiver.clone(),
            fsm_resp_sender,
            command_req_receiver,
            command_resp_sender,
        );
        let mut open_block = generate_empty_block();
        let executed_block = executor.to_executed_block(open_block.clone());

        thread::spawn(move || {
            let mut new_open_block = generate_empty_block();
            new_open_block.header.set_timestamp(2);
            // new_open_block is different from outside open_block
            fsm_req_sender.send(new_open_block);
        });
        ::std::thread::sleep(Duration::new(2, 0));
        let status_after_pause_2 = executor.fsm_pause(executed_block, 2);

        open_block.header.set_timestamp(2);

        assert_eq!(
            format!("{}", StatusOfFSM::Initialize(open_block)),
            format!("{}", status_after_pause_2)
        );
    }

    #[test]
    fn test_fsm_pause_recv_same_empty_block() {
        let (fsm_req_sender, fsm_req_receiver) = crossbeam_channel::unbounded();
        let (fsm_resp_sender, _fsm_resp_receiver) = crossbeam_channel::unbounded();
        let (_command_req_sender, command_req_receiver) = crossbeam_channel::bounded(0);
        let (command_resp_sender, _command_resp_receiver) = crossbeam_channel::bounded(0);
        let executor = init_executor2(
            fsm_req_receiver.clone(),
            fsm_resp_sender,
            command_req_receiver,
            command_resp_sender,
        );
        let open_block = generate_empty_block();
        let executed_block = executor.to_executed_block(open_block.clone());
        let executed_block_clone = executor.to_executed_block(open_block.clone());

        thread::spawn(move || {
            let new_open_block = generate_empty_block();
            // new_open_block the same as outside open_block
            fsm_req_sender.send(new_open_block);
        });
        ::std::thread::sleep(Duration::new(2, 0));
        let status_after_pause_2 = executor.fsm_pause(executed_block, 2);

        assert_eq!(
            format!("{}", StatusOfFSM::Pause(executed_block_clone, 2)),
            format!("{}", status_after_pause_2)
        );
    }

    #[test]
    fn test_fsm_pause_recv_same_block() {
        let (fsm_req_sender, fsm_req_receiver) = crossbeam_channel::unbounded();
        let (fsm_resp_sender, _fsm_resp_receiver) = crossbeam_channel::unbounded();
        let (_command_req_sender, command_req_receiver) = crossbeam_channel::bounded(0);
        let (command_resp_sender, _command_resp_receiver) = crossbeam_channel::bounded(0);
        let mut executor = init_executor2(
            fsm_req_receiver.clone(),
            fsm_resp_sender,
            command_req_receiver,
            command_resp_sender,
        );
        let open_block = generate_block(&executor, 2);

        // 1. init -> pause(0) -> execute(1) -> pause(1)
        let status_of_initialize = StatusOfFSM::Initialize(open_block.clone());
        let executed_block = executor.to_executed_block(open_block.clone());
        let (status_of_pause, executed_block) = transit_and_assert(
            &mut executor,
            status_of_initialize,
            StatusOfFSM::Pause(executed_block, 0),
        );
        let (status_of_execute_1th, mut executed_block) = transit_and_assert(
            &mut executor,
            status_of_pause,
            StatusOfFSM::Execute(executed_block, 1),
        );

        // 2. execute 1th transaction
        let transaction = executed_block.body().transactions[0].clone();
        executed_block.apply_transaction(
            &*executor.engine,
            &transaction,
            &executor.sys_config.block_sys_config.clone(),
        );
        executed_block
            .state
            .commit()
            .expect("commit state to re-calculate state root");
        let (status_of_pause_1th, mut executed_block) = transit_and_assert(
            &mut executor,
            status_of_execute_1th,
            StatusOfFSM::Pause(executed_block, 1),
        );

        // 3. send an equivalent OpenBlock into fsm_req channel
        let new_open_block = open_block.clone();
        fsm_req_sender.send(new_open_block);

        // 4. continue until finalize
        let transaction = executed_block.body().transactions[1].clone();
        executed_block.apply_transaction(
            &*executor.engine,
            &transaction,
            &executor.sys_config.block_sys_config.clone(),
        );
        executed_block
            .state
            .commit()
            .expect("commit state to re-calculate state root");
        let mut status = status_of_pause_1th;
        loop {
            status = match status {
                StatusOfFSM::Finalize(_) => {
                    assert_eq!(
                        format!("{}", status),
                        format!("{}", StatusOfFSM::Finalize(executed_block)),
                    );
                    break;
                }
                _ => transit(&mut executor, status),
            };
        }
    }

    #[test]
    fn test_fsm_pause_recv_diff_block() {
        let (fsm_req_sender, fsm_req_receiver) = crossbeam_channel::unbounded();
        let (fsm_resp_sender, _fsm_resp_receiver) = crossbeam_channel::unbounded();
        let (_command_req_sender, command_req_receiver) = crossbeam_channel::bounded(0);
        let (command_resp_sender, _command_resp_receiver) = crossbeam_channel::bounded(0);
        let mut executor = init_executor2(
            fsm_req_receiver.clone(),
            fsm_resp_sender,
            command_req_receiver,
            command_resp_sender,
        );
        let open_block = generate_block(&executor, 2);

        // 1. init -> pause(0) -> execute(1) -> pause(1)
        let status_of_initialize = StatusOfFSM::Initialize(open_block.clone());
        let status_of_pause = transit(&mut executor, status_of_initialize);
        let status_of_execute = transit(&mut executor, status_of_pause);
        let status_of_pause = transit(&mut executor, status_of_execute);

        // 3. send an un-equivalent OpenBlock into fsm_req channel
        let new_open_block = generate_block(&executor, 10);
        fsm_req_sender.send(new_open_block.clone());

        // 4. continue until finalize
        let mut executed_block = executor.to_executed_block(new_open_block);
        let mut transactions = { executed_block.body.transactions.clone() };
        for transaction in transactions.iter_mut() {
            // let mut t = transaction.clone();
            executed_block.apply_transaction(
                &*executor.engine,
                &transaction,
                &executor.sys_config.block_sys_config.clone(),
            );
        }
        executed_block
            .state
            .commit()
            .expect("commit state to re-calculate state root");
        let mut status = status_of_pause;
        loop {
            status = match status {
                StatusOfFSM::Finalize(_) => {
                    assert_eq!(
                        format!("{}", status),
                        format!("{}", StatusOfFSM::Finalize(executed_block)),
                    );
                    break;
                }
                _ => transit(&mut executor, status),
            };
        }
    }
}
