# CITA Interfaces

CITA RPC specifications and tests.

## 目的

主要是为了测试 API 接口的所有类型能正常工作，
对于 EVM 内不同情况的但是 RPC 接口一致的情况，不属于本测试覆盖范围，部分 EVM 指令的测试需要EVM内部构造测试用例。

## 分类

- blockNumber

  正确参数：[正确结果[包含交易内容，不包含交易内容]，错误结果]

  错误参数：[个数错误，类型错误]

- sendRawTransaction 暂时不处理

- getBlockByHash

  正确参数：

  - 参数：[hash, latest, earliest, genesis]
  - 返回：[有结果[有交易，无交易]，无结果]

  错误参数：...

- getBlockByNumber

  正确参数：...

  错误参数：...

- getTransactionReceipt

  正确参数:

  - 有结果
    - 交易正常处理
    - 交易处理失败： ReceiptError 的类型参见 ReceiptError，对于权限类别的验证，这里不覆盖。
  - 无结果

  错误参数：

- getLogs

  正确参数：几种组合...

  错误参数：...

- call

  正确参数: [...]

  错误参数: ...

- getTransaction...

- getTransactionCount ...

- getCode ...

- getAbi...

- newFilter ...

- newBlockFilter 暂时不测试

- uninstallFilter 暂时不测试

- getFilterChanges 暂时不测试

- getFilterLogs 暂时不测试

- getTransactionProof 暂时不测试

## 原则

根据以上测试数据构造数据和测试用例
