# Radix Router Examples & Test Cases

这个目录包含了 router-radix 的完整示例和测试用例集合。

## 测试用例概览

### 1. `basic.rs` - 基础示例
**目的**: 展示路由器的基本功能和使用方法

**测试内容**:
- ✅ 精确路径匹配 (`/api/users`)
- ✅ 参数提取 (`:id`)
- ✅ 多参数路由 (`:id/posts`)
- ✅ 通配符匹配 (`*path`)
- ✅ 通配符主机名 (`*.api.example.com`)
- ✅ HTTP 方法验证
- ✅ 多方法支持 (`GET | PUT | DELETE`)

**运行方式**:
```bash
cargo run --example basic
```

### 2. `concurrency_test.rs` - 并发性能测试
**目的**: 验证路由器的线程安全性和并发性能

**测试内容**:
- ✅ 单线程性能基准
- ✅ 多线程并发访问（8线程）
- ✅ 无锁读取验证
- ✅ 不同路由类型的性能对比

**性能指标**:
- 精确匹配: ~1,000,000+ ops/sec
- 参数匹配: ~800,000+ ops/sec
- 通配符匹配: ~700,000+ ops/sec

**运行方式**:
```bash
cargo run --example concurrency_test
# 或使用 release 模式获得更高性能
cargo run --example concurrency_test --release
```

### 3. `edge_cases.rs` - 边界情况测试
**目的**: 测试各种边界条件和特殊情况

**测试内容**:
- ✅ 根路径和空路径处理
- ✅ 特殊字符支持 (`-`, `_`, `.`)
- ✅ 超长路径处理
- ✅ 相似路径区分 (`/api/user` vs `/api/users`)
- ✅ 多通配符优先级
- ✅ 特殊参数值 (UUID, 数字, 编码字符串)
- ✅ 尾部斜杠处理
- ✅ 大小写敏感性
- ✅ 空路由器行为
- ✅ 带端口的主机名
- ✅ 所有 HTTP 方法支持
- ✅ 深度嵌套参数

**运行方式**:
```bash
cargo run --example edge_cases
```

### 4. `benchmark.rs` - 性能基准测试
**目的**: 详细的性能分析和基准测试

**测试内容**:
- ✅ 精确路径匹配性能（哈希查找）
- ✅ 单参数提取性能
- ✅ 多参数提取性能
- ✅ 通配符匹配性能（短路径 vs 长路径）
- ✅ HTTP 方法匹配开销
- ✅ 主机名匹配开销（精确 vs 通配符）
- ✅ 优先级路由解析
- ✅ 大规模路由器性能（100+ 路由）
- ✅ 复杂真实场景性能

**性能分析**:
- 每个基准测试运行 1,000,000 次迭代
- 提供 ops/sec 和 ns/op 指标
- 对比不同场景的性能差异

**运行方式**:
```bash
cargo run --example benchmark --release
```

### 5. `stress_test.rs` - 压力测试
**目的**: 在极端负载下测试路由器的稳定性和性能

**测试内容**:
- ✅ 大规模路由创建（10,000 路由）
- ✅ 顺序查询性能（大路由表）
- ✅ 高并发压力测试（16 线程，每线程 100,000 查询）
- ✅ 动态路由管理（添加/删除 1,000 路由）
- ✅ 病态情况处理:
  - 深度嵌套（20 层）
  - 大量参数（10 个参数）
  - 超长路径（300+ 字符）

**性能指标**:
- 路由创建速度: ~5,000 routes/sec
- 并发查询吞吐: ~2,000,000 qps (16 线程)
- 平均延迟: <1 µs/query

**运行方式**:
```bash
cargo run --example stress_test --release
```

### 6. `integration.rs` - 真实场景集成测试
**目的**: 模拟真实 API 网关场景

**测试场景**:
1. **公共 API 访问**
   - 健康检查
   - 状态端点
   - API 文档

2. **用户服务请求**
   - CRUD 操作
   - 用户详情和配置文件

3. **订单服务 - 嵌套资源**
   - 订单列表
   - 订单项目
   - 支付处理

4. **多租户路由**
   - 基于主机名的租户隔离
   - 通配符主机匹配

5. **静态文件服务**
   - CSS、JS、图片、字体
   - 下载服务

6. **管理面板**
   - 优先级路由
   - 主机名限制

7. **WebSocket 连接**
   - 聊天服务
   - 通知服务
   - 直播流

8. **基于方法的路由**
   - 读写分离
   - 不同方法路由到不同上游

9. **搜索端点**
   - 参数化搜索类型

**运行方式**:
```bash
cargo run --example integration
```

### 7. `vars_filter_test.rs` - 变量表达式和过滤函数测试
**目的**: 测试高级路由功能

**测试内容**:
1. **基础变量表达式**
   - `Expr::Eq` - 等值匹配
   
2. **正则表达式匹配**
   - `Expr::Regex` - 模式匹配
   - User-Agent 检测示例

3. **多变量表达式**
   - AND 逻辑组合
   - 多条件验证

4. **自定义过滤函数**
   - 营业时间检查
   - 动态业务逻辑

5. **速率限制过滤**
   - 基于请求计数的限流

6. **IP 访问控制**
   - 内网 IP 白名单

7. **A/B 测试**
   - 基于用户 ID 的流量分配

8. **组合过滤**
   - 表达式 + 过滤函数组合

**应用场景**:
- 环境隔离（生产/开发）
- 移动端适配
- 高级权限控制
- 流量管理
- 安全策略

**运行方式**:
```bash
cargo run --example vars_filter_test
```

## 运行所有测试

### 编译所有示例
```bash
cargo build --examples
```

### 运行所有示例（按顺序）
```bash
# 基础功能
cargo run --example basic

# 边界情况
cargo run --example edge_cases

# 集成测试
cargo run --example integration

# 变量和过滤器
cargo run --example vars_filter_test

# 性能测试（建议使用 release 模式）
cargo run --example benchmark --release
cargo run --example concurrency_test --release
cargo run --example stress_test --release
```

### 批量运行脚本
```bash
#!/bin/bash
# run_all_tests.sh

echo "=== Running All Radix Router Tests ==="
echo ""

tests=(
    "basic"
    "edge_cases"
    "integration"
    "vars_filter_test"
)

perf_tests=(
    "benchmark"
    "concurrency_test"
    "stress_test"
)

echo "Running functional tests..."
for test in "${tests[@]}"; do
    echo ">>> Running: $test"
    cargo run --example "$test" || exit 1
    echo ""
done

echo "Running performance tests (release mode)..."
for test in "${perf_tests[@]}"; do
    echo ">>> Running: $test"
    cargo run --example "$test" --release || exit 1
    echo ""
done

echo "=== All Tests Completed Successfully! ==="
```

## 测试覆盖范围

### 核心功能
- ✅ 路径匹配（精确、前缀、通配符）
- ✅ 参数提取（单个、多个、嵌套）
- ✅ HTTP 方法匹配
- ✅ 主机名匹配（精确、通配符）
- ✅ 优先级路由
- ✅ 变量表达式
- ✅ 自定义过滤函数

### 性能特性
- ✅ 哈希优化的精确匹配
- ✅ 预编译的正则表达式
- ✅ 无锁并发读取
- ✅ 线程安全
- ✅ Async 安全

### 边界情况
- ✅ 空路由器
- ✅ 大量路由
- ✅ 深度嵌套
- ✅ 超长路径
- ✅ 特殊字符
- ✅ 路由冲突

### 真实场景
- ✅ API 网关
- ✅ 微服务路由
- ✅ 静态文件服务
- ✅ WebSocket 路由
- ✅ 多租户系统
- ✅ A/B 测试
- ✅ 访问控制

## 性能参考

### 开发模式 (Debug)
- 精确匹配: ~1M ops/sec
- 参数匹配: ~800K ops/sec
- 通配符匹配: ~700K ops/sec

### 发布模式 (Release)
- 精确匹配: ~5M+ ops/sec
- 参数匹配: ~3M+ ops/sec
- 通配符匹配: ~2M+ ops/sec
- 并发吞吐: ~10M+ qps (16 线程)

## 贡献新的测试

如果你想添加新的测试用例，请遵循以下结构：

```rust
/// 测试描述
/// 这个示例测试...

use router-radix::{RadixHttpMethod, RadixMatchOpts, RadixRouter, RadixNode};

fn main() -> anyhow::Result<()> {
    println!("=== Your Test Name ===\n");
    
    // 创建路由
    let routes = vec![
        // ... 你的路由定义
    ];
    
    let mut router = RadixRouter::new()?;
    router.add_routes(routes)?;
    
    // 测试场景
    println!("Test 1: Description");
    {
        // 测试逻辑
        let opts = RadixMatchOpts::default();
        let result = router.match_route("/path", &opts)?;
        assert!(result.is_some());
        println!("  ✓ Test passed");
    }
    
    println!("\n=== All Tests Passed ✓ ===");
    Ok(())
}
```

## 许可证

与主项目保持一致。

