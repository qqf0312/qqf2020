
pub use crate::BatchingOCC;
//pub use crate::Threadpool;
pub use petgraph::Graph;
pub use petgraph::algo;
pub use petgraph::Direction;
pub use petgraph::prelude::NodeIndex;
pub use crate::common_bag::_transaction;
pub use std::collections::{HashMap,HashSet,BTreeMap};
pub use stackvec::StackVec;
pub use petgraph::visit::Visitable;
pub use fixedbitset::FixedBitSet;
//pub use crate::Simple_db;
//多线程的初始化代码
use std::sync::mpsc::{channel,Sender,Receiver};
use std::sync::{Arc,Mutex,Condvar};
use std::sync::atomic::{AtomicUsize,Ordering};
//test executeParallel: PASS
#[derive(Clone)]
pub struct miner{
    pub bocc: BatchingOCC::batchingOcc,
    //线程池(尚未编写)
    pub commitRatio : f32,
    pub k: i32,
    pub batch : HashMap<i32,_transaction::transaction>,
    pub db: Simple_db::db,
    pub tdg_toposort: Vec<i32>,
}
impl miner{
    pub fn new()->miner{
        let miner_ = miner{
            bocc: BatchingOCC::batchingOcc::new(),
            //pool
            //commitRatio: 0.8,
            commitRatio: 1.0,
            k : 1,
            batch : HashMap::new(),
            db: Simple_db::db::new(),
            tdg_toposort: Vec::new(),
        };
        return  miner_;
    }
    pub fn set_db(&mut self,db: Simple_db::db){
        self.db = db;
    }
    pub fn new_param(/*ExcutorSeveice pool*/cr: f32,k:i32)->miner{
        let miner_ = miner{
            bocc: BatchingOCC::batchingOcc::new(),
            //pool
            commitRatio: cr,
            k : k,
            batch : HashMap::new(),
            db: Simple_db::db::new(),
            tdg_toposort: Vec::new(),
        };
        return  miner_;
    }
    pub fn input_batch(&mut self,map: HashMap<i32,_transaction::transaction>){
        self.batch = map;
    }
    pub fn executeParallel(&self,txs: HashMap<i32,_transaction::transaction>){
        let iter = txs.clone().into_iter();
        let mut guards :Vec<std::thread::JoinHandle<()>> = Vec::new();
        let db_arc = Arc::new(self.db.clone());
        for n in iter{
            let (k,tran)=n.clone();
            //执行交易，目前是 x2 操作
            //模拟交易
            
            let guard = std::thread::spawn(move || {
                //executed tx
                /*
                let temp = tran.readSet_.clone().unwrap();
                let r = temp[0].clone();
                let temp = tran.readSet_.clone().unwrap();
                let map = db_arc.kv_db.clone();
                let w= temp[0].clone();
                let temp = tran.readValues_.clone().unwrap();
                let wv = temp[0].clone();
                */
                let tranid = tran.tranId_;
                println!("executing tran id : {} ",tranid);
            });
            guards.push(guard);
        }
        for guard in guards{
            guard.join().unwrap();
        }
    }
    pub fn txCommit(&mut self,tx: _transaction::transaction){
        let tran = tx;
        let temp = tran.readSet_.clone().unwrap();
        let r = temp[0].clone();
        let rv = self.db.kv_db.get(&r).unwrap().clone();
        println!("r={:?} rv={}",r,rv);
        let temp = tran.writeSet_.clone().unwrap();
        let w= temp[0].clone();
        let wv = rv * 2;
        println!("w={:?} wv={}",w,wv);
        self.db.db_insert(w, wv);
    }
    //并发处理批交易
    pub fn concurrentMining(&mut self,
        //g1: Graph<_transaction::transaction,i32>,
        /*g2: Graph<_transaction::transaction,i32>*/){//测试的时候临时加上的2个参数g1,g2
        //创建线程池
        //let pool = Threadpool::ThreadPool::new(8);
        //i32 = tranid
        let mut txs = self.batch.clone();
        let hash_len = self.batch.len() as f32;
        let number :f32 = self.commitRatio*hash_len;
        let number = number as i32;
        println!("when executing transactions' number is :{}, the loop will end.",number);
        let mut count = 0;
        while self.bocc.getnCommit() < number /*&& count < 4*/{
            //并发执行交易
            count = count + 1;
            println!("nCommit:{}",self.bocc.getnCommit());
            self.executeParallel(txs.clone());
            //cg冲突图形成
            //哈希表转换成vec,~.cloned().collect()
            let vec = txs.values().cloned().collect();
            let mut cg = self.bocc.constructConflictGraph(vec);
            //获得中止节点集合（以cg的顶点index为vec的值）
            let abort = self.bocc.findAbortTransactionSet(cg.clone());
            //打印中止节点
            println!("---中止节点---");
            for n in abort.clone(){
                println!("abort_tranId:{}",n.tranId_);
            }
            println!("-----------");
            //let abort = self.bocc.findAbortTransactionSet(cg.clone());
            //从冲突图cg中删除中止的交易abort
            let iter = abort.clone().into_iter();
            for node in iter{
                //从冲突图中依据weight来寻找index
                let iter = cg.node_indices().clone();
                let mut Vid : petgraph::prelude::NodeIndex = NodeIndex::new(0);
                for index in iter{
                    let x = cg.node_weight(index).unwrap().clone();
                    if x.tranId_ == node.tranId_ {
                        Vid = index;
                        break;
                    }
                }
                //开始从component移除minVid的边
                //移除出度边
                let dir = Direction::Outgoing;
                //就离谱，不clone拿出来的component不能改，这你能信？
                let component_clone =cg.clone();
                //let iter_out = component.neighbors_directed(minVid,dir);
                let iter_out = component_clone.neighbors_directed(Vid,dir);
                for n in iter_out{
                    let edge_index = cg.find_edge(Vid, n);
                    //一定找得到minVid->n的边
                    cg.remove_edge(edge_index.unwrap());
                }
                //开始移除入度边
                let dir = Direction::Incoming;
                let component_clone =cg.clone();
                let iter_in = component_clone.neighbors_directed(Vid,dir);
                for n in iter_in{
                    let edge_index = cg.find_edge(n, Vid);
                    //一定能找到n->minVid的边
                    cg.remove_edge(edge_index.unwrap());
                }
                //开始从component移除minVid的节点
                let tx = cg.node_weight(Vid).unwrap().clone();
                cg.remove_node(Vid);
            }
            //完成删除
            //得到拓扑排序 sorted里面是cg拓扑的结果，以cg的节点标号为序
            let mut sorted = algo::toposort(&cg,None).unwrap();
            if !sorted.is_empty(){
                let mut s = sorted.clone();
                let iter = s.into_iter();
                for n in iter{
                    let x = cg.node_weight(n).unwrap().clone();
                    self.tdg_toposort.push(x.tranId_);
                }
                //self.tdg_toposort.append(&mut s);
            }
            //Stack topology =cg.getTopologicalSort
            println!("---topo---");
            for node in sorted.clone(){
                let w = cg.node_weight(node).unwrap().clone();
                print!("{} ",w.tranId_);
            }
            println!("");
            println!("----------");
            for node in sorted{
                //node代表的交易tx
                let commitTx = cg.node_weight(node).unwrap().clone();
                //提交程序
                self.txCommit(commitTx.clone());
                //提交至TDG图中
                self.bocc.commitTransaction(commitTx.clone(), 1);
                //删除hashmap（txs）中的comiitTx
                txs.remove(&commitTx.tranId_);
            }
            //txs剩下的都是没有执行过的交易
            //正常交易中在前方excuted已经执行过交易了
            //这时候则需要回滚中止交易txs（提交过的交易已经被remove出去了）
        }
        //移除中止节点
        //留在hashmap的就是中止节点
        //好奇怪啊为什么会有ratio这种奇怪的数字，难道不处理完的batch就直接删除了嘛
        let vec = txs.values();
        for n in vec{
            self.batch.remove(&n.tranId_);
        }
    }
    //暂时用不到
    pub fn kWayPartition(&self,tdg : Graph<_transaction::transaction,i32>)
    ->Vec<Graph<_transaction::transaction,i32>>{
        let mut partition : Vec<Graph<_transaction::transaction,i32>>= Vec::new();
        //计算总体节点权重 
        let mut totalCost = 0;
        let iter = tdg.node_indices().clone();
        for node in iter{
            let tx = tdg.node_weight(node).unwrap().clone();
            //获取每个节点的权重
            let w = tx.weight_;
            totalCost = totalCost + w;
        }
        let upperBound = totalCost/self.k;
        //print!("upperBound:{}",upperBound);
        let mut index = 0;
        let mut cost = 0;
        for _ in 0..self.k{
            let mut part : Graph<_transaction::transaction,i32>= Graph::new();
            partition.push(part);
        }
        let weightlist = tdg.raw_edges();
        //对边集进行排序
        let iter = weightlist.iter();
        let mut edges_vec = Vec::new();
        //let sorted_vec = Vec::new();
        for n in iter{
            edges_vec.push(n.clone());
        }
        let number = edges_vec.len();
        //把edge按照从大到小排序
        for i in 0..number{
            for j in 0..number{
                //符号有待商榷
                if edges_vec[i].weight > edges_vec[j].weight{
                    let temp = edges_vec[i].clone();
                    edges_vec[i] = edges_vec[j].clone();
                    edges_vec[j] = temp;
                }
            }
        }
        //edges_vec成为有序向量
        let iter = tdg.node_indices();
        let mut visitmap = HashMap::new();
        for n in iter{
            let bool_ = false;
            visitmap.insert(n,bool_);
        }
        let iter = edges_vec.iter();
        for e in iter{
            //出节点
            let start = e.source();
            //入节点
            let end = e.target();
            let s_bool = visitmap.get(&start).unwrap();
            let e_bool = visitmap.get(&end).unwrap();
            if !s_bool && !e_bool {
                let mut tmp = cost.clone();
                tmp = tmp + tdg.node_weight(start).unwrap().weight_.clone();
                tmp = tmp + tdg.node_weight(end).unwrap().weight_.clone();
                if tmp >= upperBound {
                    cost = 0;
                    if index + 1 < self.k{
                        index = index + 1;
                    }
                }
                //更新visit已读状态
                visitmap.insert(start,true);
                visitmap.insert(end,true);
                let add_start =tdg.node_weight(start).unwrap().clone();
                let add_end =tdg.node_weight(end).unwrap().clone();
                let x = index as usize;
                partition[x].add_node(add_start.clone());
                partition[x].add_node(add_end.clone());
                cost = cost + add_start.weight_;
                cost = cost + add_end.weight_;
            }
            //判断起始节点被遍历过了而终止节点没被遍历的情况
            else if !s_bool && e_bool.clone() {
                let mut tmp = cost.clone();
                let add_start =tdg.node_weight(start).unwrap().clone();
                tmp = tmp + add_start.weight_;
                if tmp >= upperBound {
                    cost = 0;
                    if index + 1 < self.k {
                        index = index + 1;
                    }
                }
                visitmap.insert(start,true);
                let x = index as usize;
                partition[x].add_node(add_start.clone());
                cost = cost + add_start.weight_;
            }
            else if s_bool.clone() && !e_bool {
                let mut tmp = cost.clone();
                let add_end =tdg.node_weight(end).unwrap().clone();
                tmp = tmp + add_end.weight_;
                if tmp >= upperBound {
                    cost = 0;
                    if index + 1 < self.k {
                        index = index + 1;
                    }
                }
                visitmap.insert(end,true);
                let x = index as usize;
                partition[x].add_node(add_end.clone());
                cost = cost + add_end.weight_;
            }
        }
        let iter = tdg.node_indices();
        for vertex in iter{
            let v_bool = visitmap.get(&vertex).unwrap();
            if !v_bool {
                let mut tmp = cost.clone();
                let w = tdg.node_weight(vertex).unwrap().clone();
                tmp = tmp + w.weight_;
                if tmp > upperBound {
                    cost = 0;
                    if index + 1 < self.k{
                        index = index + 1;
                    }
                }
                visitmap.insert(vertex,true);
                let x = index as usize;
                let add_v =tdg.node_weight(vertex).unwrap().clone();
                partition[x].add_node(add_v.clone());
                cost = cost + add_v.weight_;
            }
        }
        return partition;
    }
}