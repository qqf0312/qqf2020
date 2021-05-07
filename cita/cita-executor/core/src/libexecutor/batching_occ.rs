//pub mod common_bag;
//pub mod common_bag;
pub use petgraph::Graph;
pub use petgraph::algo;
pub use petgraph::Direction;
pub use petgraph::prelude::NodeIndex;
pub use super::_transaction::TransactionInfo;
pub use std::collections::HashMap;
pub use std::*;

//test fn construct_conflict_graph : PASS
//test fn has_conflict : PASS
//test fn commit_transaction_info : PASS
//test fn greedy_select_vertex : PASS
//test fn find_abort_transaction_info_set : PASS

#[derive(Clone)]
pub struct BatchingOcc{
    pub n_validation: i32,
    pub n_commit: i32,
    pub tdg_toposort: Vec<usize>,
    pub cg: Graph<TransactionInfo,i32>,
    pub tdg: Graph<TransactionInfo,i32>,
    
}
impl BatchingOcc{
    pub fn new()->BatchingOcc{
        let b = BatchingOcc{
            n_validation: 0,
            n_commit:0,
            tdg_toposort:Vec::new(),
            cg:Graph::new(),
            tdg:Graph::new(),
        };
        b
    }
    //要判断Vec是否为空，在进行判断
    //在进入iter时候，一定要unwrap（）之后进行（且分开两行写）
    //None不能被unwrap，会报错，所以之前应该用if语句判断之后执行
    pub fn construct_conflict_graph(&mut self,tx_list: Vec<TransactionInfo>)->Graph< TransactionInfo,i32>{
        let mut cg = Graph::new();
        let mut hashmap = HashMap::new();
        let vec_tran = tx_list.clone();
        let iter = vec_tran.into_iter();
        //加节点进入图中
        for n in iter{
            let tx = n.clone();
            //let x:i32 = y 是将y的值复制一份给x绑定
            //let mut tx : TransactionInfo= n;
            //加上 & 则只是引用传递，不会改变所有权（也就是tx）在add之后不会消失
            let node_index = cg.add_node(tx.clone());
            hashmap.insert(tx.tran_id,node_index);
        }
        //
        let vec_tran = tx_list.clone();
        let iter = vec_tran.into_iter();
        for n in iter{
            let tx = n;
            //获取tx写集合
            //let _writeset = tx.write_set.as_ref();
            let _writeset = tx.write_set.as_ref();
            let iter_other = tx_list.clone().into_iter();
            let mut  bool_:bool;
            for n in iter_other{
                let othertx = n;
                //判断两个交易是否相同，cita中可以换成交易的hash值
                if othertx.tran_id == tx.tran_id{
                    continue;
                }
                //获取other读集合，看看other的读集是否和tx的写集冲突
                if _writeset == None || othertx.read_set == None{
                    bool_= false;
                }
                else{
                    let w = _writeset.unwrap();
                    let r = othertx.read_set.as_ref().unwrap();
                    bool_ = self.has_conflict(&w,&r);
                }
                if  bool_ == true{
                    //通过哈希表查找tx对应的nodeindex
                    //利用从hashmap中‘复制’一份NodeIndex给a和b
                    //这样hashmap中的数据应该还会存在
                    let a : petgraph::prelude::NodeIndex = *hashmap.get(&tx.tran_id).unwrap();
                    let b : petgraph::prelude::NodeIndex = *hashmap.get(&othertx.tran_id).unwrap();
                    //let b = hashmap.get(&othertx.tran_id).unwrap();
                    //1为临时数据，以后根据实际tx修改
                    //冲突图从b->a
                    cg.add_edge(b, a, 1);
                }
            }
        }
        
        //self.cg=cg;
        //测试数据是否已经存入gragh中
        //计算node数量
        //let num = cg.node_count();
        //计算edge数量
        //let num_ =cg.edge_count();
        //打印出该图的节点和边的属性
        //打印节点的index以及tx的tranId（cita中是其唯一的哈希值）
        let nodes_iter=cg.node_indices().clone();
        for n in nodes_iter{
            //let tx = hashmap.get()
            let node = cg.node_weight(n).unwrap();
            
            println!("{:?}  tranId:{}",n,node.tran_id);
        }
        //打印边的index 
        let edge_iter = cg.edge_indices().clone();
        for n in edge_iter{
            let from_to = cg.edge_endpoints(n).unwrap();
            println!("edge:{:?} from_to {:?}",n,from_to);
        }
        self.cg = cg.clone();
        return cg;
        
    }
    pub fn has_conflict(&self,writeset:&Vec<String>,read_set:&Vec<String>)->bool{
        let mut conflict = false;
        let iter_w = writeset;
        for n in iter_w{
            let string_temp = n;
            let iter_r = read_set;
            for n in iter_r{
                if string_temp == n{
                    conflict = true;
                    break;
                }
            }
            if conflict == true{
                break;
            }
        }
        conflict
    }
    pub fn greedy_select_vertex(&self,mut component: Graph< TransactionInfo,i32>)
    ->Option<Vec<TransactionInfo>>{
        //let v :Option<Vec< TransactionInfo>>;
        if component.node_count() == 1 {
            return None;
        }
        //判断是否有环，无环则退出循环
        let cycle_bool = algo::is_cyclic_directed(&component);
        if cycle_bool == false {
            return None;
        }
        let mut v = Vec::new();
        //取一个无限大的数据，这里暂时取1000
        //let min = 1000;
        let mut max = 0;
        //初始化min_vid节点，该节点记录在component下的下标节点
        //11.21在选取节点的时候出错了 得修改
        let mut min_vid :petgraph::prelude::NodeIndex = NodeIndex::new(0);
        let mut min_vid_in_cg :petgraph::prelude::NodeIndex = NodeIndex::new(0);
        //component.remove_node(min_vid);
        let iter = component.node_indices();
        for n in iter{
            //选择出入度最大的节点进行删除
            //将要删除节点和其相关的边
            //let num = component.neighbors_undirected(n).count();
            //11.22 更改至cg总图的出入度 n是component的下标转移到 cg会出错
            let cg_indices = self.cg.node_indices().clone();
            //cg_index接受cg中n.tranid的下标
            let mut cg_index : petgraph::prelude::NodeIndex = NodeIndex::new(0);
            //遍历cg，查找与n的id对应的id在cg中的下标
            for index in cg_indices{
                let component_id = component.node_weight(n).unwrap().tran_id;
                let cg_id = self.cg.node_weight(index).unwrap().tran_id;
                //找到了以后
                if component_id == cg_id{
                    cg_index = index;
                    break;
                }
            }
            let num = self.cg.neighbors_undirected(cg_index).count();
            //println!("{:?} num={}",n,num);
            if num > max {
                min_vid = n;
                min_vid_in_cg = cg_index;
                max = num.clone();
                //println!("num={} max={} tran id:{}",num,max,component.node_weight(n).unwrap().tran_id);
            }
            //两个节点的出入度相同,则判断谁的出度更小
            //出度更小的删除
            else if num == max {
                let dir = Direction::Outgoing;
                //计算min_vid和n的出度，取较小的那一个
                //let min_vid_io_number =component.neighbors_directed(min_vid, dir).count();
                //let n_io_number = component.neighbors_directed(n, dir).count();
                //11.22 更改至获取cg总图的出入度 还是上个问题
                let min_vid_io_number =self.cg.neighbors_directed(min_vid_in_cg, dir).count();
                let n_io_number = self.cg.neighbors_directed(cg_index, dir).count();
                if min_vid_io_number > n_io_number{
                    min_vid = n;
                    min_vid_in_cg = cg_index;
                    max = num.clone();
                    //println!("num={} max={} tran id:{}",num,max,component.node_weight(n).unwrap().tran_id);
                }
            }
        }
        //开始从component移除min_vid的边
        //移除出度边
        let dir = Direction::Outgoing;
        //就离谱，不clone拿出来的component不能改，这你能信？
        let component_clone =component.clone();
        //let iter_out = component.neighbors_directed(min_vid,dir);
        let iter_out = component_clone.neighbors_directed(min_vid,dir);
        for n in iter_out{
            let edge_index = component.find_edge(min_vid, n);
            //一定找得到min_vid->n的边
            //println!("删除的出度边是{:?}",edge_index.unwrap());
            component.remove_edge(edge_index.unwrap());
        }
        //开始移除入度边
        let dir = Direction::Incoming;
        let component_clone =component.clone();
        //let iter_out = component.neighbors_directed(min_vid,dir);
        let iter_in = component_clone.neighbors_directed(min_vid,dir);
        for n in iter_in{
            let edge_index = component.find_edge(n, min_vid);
            //一定能找到n->min_vid的边
            //println!("删除的入度边是{:?}",edge_index.unwrap());
            component.remove_edge(edge_index.unwrap());
        }
        //开始从component移除min_vid的节点
        let tx = component.node_weight(min_vid).unwrap().clone();
        //println!("删除的节点是{:?}",min_vid);
        component.remove_node(min_vid);
        //把已经删除得节点tx加入中止节点中
        v.push(tx);
        println!("______________");
        let is_null = self.greedy_select_vertex(component);
        let is_null_clone = is_null.clone();
        let empty_vec : Vec<TransactionInfo> = Vec::new();
        let bool_ = is_null_clone.unwrap_or(empty_vec).is_empty();
        //vec不为空，则就加入V中止集合
        if bool_ == false{
            let mut vec_ = is_null.unwrap();

            v.append(&mut vec_);
        }
        //v.append();
        //重新包装一下v，让他成为option
        let return_v : Option<Vec<TransactionInfo>> = Some(v);
        return return_v;
    }

    //pub fn tarjan_scc<G>(g: G) -> Vec<Vec<G::NodeId>>
    pub fn find_abort_transaction_info_set(&self,cg: Graph< TransactionInfo,i32>)->Vec<TransactionInfo>{
        let mut tx_set = Vec::new();
        //let mut scc =Vec::new();
        let copycg = cg.clone();
        let scc = algo::tarjan_scc(&copycg);
        //输出tarjan后的结果
        
        println!("tarjan:");
        let scc_copy = scc.clone();
        let i = scc_copy.iter();
        let mut num = 0;
        for n in i{
            let i = n.iter();
            for n_ in i{
                print!("{:?}",n_);
            }
            num=num+1;
            println!(" 第{}组",num);
        }
        println!("---tarjan_end---");
        //Scc的副本
        let mut copy_scc = Vec::new();
        let iter = scc.into_iter();
        //取出一组强连通分量
        let mut scc_count = 1;
        for component in iter{
            //创建一个hashmap来储存cg中的节点号（before）和copy_component新节点号的一一对应
            let mut map : HashMap<petgraph::prelude::NodeIndex,petgraph::prelude::NodeIndex> 
            = collections::HashMap::new();
            //强连通分量的图
            let mut copy_component = Graph::new();
            let iter_node = component.clone().into_iter();
            //添加节点到对应的强连通分量
            //#这时的copy_component是没有边的
            for _nodeindex in iter_node{
                let addnode = cg.node_weight(_nodeindex).unwrap().clone();
                let after_node = copy_component.add_node(addnode);
                //before 和 after 一一对应
                map.insert(_nodeindex.clone(),after_node.clone());
            }
            //节点和节点之间互相创建新的边
            let iter = component.clone().into_iter();
            for from in iter{
                let iter_ = component.clone().into_iter();
                for to in iter_{
                    //println!("该趟是处理:from:{:?} to:{:?}",from,to);
                    if from == to {
                        continue;
                    }
                    let edgeindex = cg.find_edge(from,to);
                    //println!("findout!");
                    if edgeindex != None{
                        //取得原有的edge weight
                        let w = cg.edge_weight(edgeindex.unwrap()).unwrap().clone();
                        //from to 的index是cg的节点，要转化成强连通子图的index
                        let real_from : petgraph::prelude::NodeIndex
                        = map.get(&from.clone()).unwrap().clone();
                        let real_to : petgraph::prelude::NodeIndex
                        = map.get(&to.clone()).unwrap().clone();
                        //println!("from: {:?}  to: {:?}",from.clone(),to.clone());
                        copy_component.add_edge(real_from, real_to, w);
                    }
                }
            }
            //打印生成的强连通子图
            println!("----强连通子图{}:----",scc_count);
            scc_count = scc_count + 1;
            let nodes_iter=copy_component.node_indices().clone();
            for n in nodes_iter{
            //let tx = hashmap.get()
            let node = copy_component.node_weight(n).unwrap();
            println!("{:?}  tranId:{}",n,node.tran_id);
            }   
            //打印边的index 
            let edge_iter = copy_component.edge_indices().clone();
            for n in edge_iter{
            let from_to = copy_component.edge_endpoints(n).unwrap();
            println!("edge:{:?} from_to {:?}",n,from_to);
            }
            println!("----结束----");
            copy_scc.push(copy_component);
        }
        //运行greedySelectComponent函数
        //在获得的copy_scc强连通分量集合时，再一个一个计算中止节点
        
        let copy_ = copy_scc.clone();
        let iter = copy_.into_iter();
        let empty_vec : Vec<TransactionInfo> = Vec::new();
        //let test_tx = TransactionInfo::new(0,None,None);
        //component是一个强连通子图
        for component in iter{
            //let x = component.unwrap();
            //将所有中止节点打包成一个集合
            let mut tx_package = self.greedy_select_vertex(component).unwrap_or(empty_vec.clone());
            if !tx_package.is_empty(){
                tx_set.append(&mut tx_package);
            }
            //tx_set.append(&mut tx_package);
        }
        return tx_set;
    }
    /*
    pub fn txCommit(&mut self,tx: TransactionInfo){
        let tran = tx;
        let temp = tran.read_set.clone().unwrap();
        let r = temp[0].clone();
        let rv = self.db.kv_db.get(&r).unwrap().clone();
        let temp = tran.read_set.clone().unwrap();
        let w= temp[0].clone();
        let wv = rv * 2;
        self.db.db_insert(w, wv);
    }
    */
    pub fn commit_transaction_info(&mut self,tx: TransactionInfo,w:i32){
        //tx commit
        self.n_commit=self.n_commit + 1;
        //加新的节点到tdg依赖图中
        //记录nodeindex
        let node_index = self.tdg.add_node(tx.clone());
        //更新边
        //即交易依赖图中所有的入边都是由冲突图中该交易的出边转换而来，入边不会被包含到交易依赖图中。
        let read_set = tx.clone().read_set;
        let mut bool_ = false;
        if read_set != None{
            let iter = read_set.unwrap().clone().into_iter();
            for tx_read in iter{
                let tdg_clone = self.tdg.clone();
                let iter = tdg_clone.node_indices();
                //寻找TDG其他节点的写集
                for othernode_index in iter{
                    let othernode = self.tdg.node_weight(othernode_index.clone());
                    let othernode = othernode.unwrap().clone();
                    //遍历TDG时候判断是否为相同节点
                    if othernode.tran_id != tx.tran_id{
                        let write_set = othernode.write_set;
                        //判断写集是否为空，不为空则继续进行查找读写集
                        if write_set != None{
                            let iter = write_set.clone().unwrap().into_iter();
                            //遍历other节点写集
                            for tx_write in iter{
                                if tx_write == tx_read {
                                    bool_ = true;
                                    break;
                                }
                            }
                        }
                    }
                //存在读写冲突,加一条边，成依赖图
                if bool_ == true{
                    self.tdg.add_edge(othernode_index,node_index, w);
                    bool_ = false;
                }

                }
            }
        }
    }
    pub fn get_n_commit(&self)->i32{
        self.n_commit
    }
    pub fn get_tdg(&self){
        //打印出该图的节点和边的属性
        //打印节点的index以及tx的tranId（cita中是其唯一的哈希值）
        let g = &self.tdg;
        println!("-----tdg-----");
        let nodes_iter=g.node_indices().clone();
        for n in nodes_iter{
        //let tx = hashmap.get()
        let node = g.node_weight(n).unwrap();     
        println!("{:?}  tranId:{}",n,node.tran_id);
        }
        //打印边的index 
        let edge_iter = g.edge_indices().clone();
        for n in edge_iter{
        let from_to = g.edge_endpoints(n).unwrap();
        println!("edge:{:?} from_to {:?}",n,from_to);
        }
        println!("---tdg:end----");
    }
    pub fn get_tdg_transactioninfo(&self){
        //打印出该图的节点和边的属性
        //打印节点的index以及tx的tranId（cita中是其唯一的哈希值）
        let g = &self.tdg;
        info!("-----tdg-----");
        let nodes_iter=g.node_indices().clone();
        for n in nodes_iter{
            let node = g.node_weight(n).unwrap();     
            info!("tran_Id:{}",node.tran_id);
            let iter = node.read_set.clone().unwrap().into_iter();
            let mut index: usize = 0;
            info!("read读集:");
            for n_r in iter{
                let temp = node.read_values.clone().unwrap();
                info!("key:{} value:{:?}",n_r.clone(),temp[index].clone());
                index = index + 1;
            }
            let iter = node.write_set.clone().unwrap().into_iter();
            let mut index: usize = 0;
            info!("write写集:");
            for n_w in iter{
                let temp = node.write_values.clone().unwrap();
                info!("key:{:?} value:{:?}",n_w.clone(),temp[index].clone());
                index = index + 1;
            }
        }
        //打印边的index 
        //let edge_iter = g.edge_indices().clone();
        //for n in edge_iter{
        //let from_to = g.edge_endpoints(n).unwrap();
        //println!("edge:{:?} from_to {:?}",n,from_to);
        //}
        info!("---tdg:end----");
    }
    pub fn get_graph_in_vec(&self) -> (Vec<TransactionInfo>, Vec<(usize, usize)>){
        let tdg = self.tdg.clone();
        let (vec_nodes, vec_edges) = tdg.into_nodes_edges();
        let iter = vec_nodes.clone().into_iter();
        let mut vec_transactioninfo = Vec::new();
        for node in iter{
            let tran_info = node.weight.clone();
            vec_transactioninfo.push(tran_info);
        }
        let iter = vec_edges.iter();
        let mut vec_usize = Vec::new();
        for edge in iter{
            let from_index = edge.source().clone();
            let to_index = edge.target().clone();
            let from = self.tdg.node_weight(from_index).clone().unwrap();
            let from = from.tran_id;
            let to = self.tdg.node_weight(to_index).clone().unwrap();
            let to = to.tran_id;
            let temp = (from,to);
            vec_usize.push(temp.clone());
        }
        return (vec_transactioninfo, vec_usize);
    }
    pub fn set_toposort(&mut self, sort: Vec<usize>){
        self.tdg_toposort = sort;
    }
}
