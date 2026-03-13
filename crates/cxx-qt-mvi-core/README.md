# cxx-qt-mvi-core

MVI (Model-View-Intent) 架构核心框架库，用于 cxx-qt 应用。

## 功能特性

- **MVI Traits**: Intent、State、Reducer 核心抽象
- **类型安全**: 强类型的状态管理
- **可测试性**: 纯函数式状态转换

## 核心概念

- **Intent**: 用户意图（枚举类型）
- **State**: 应用状态（不可变数据）
- **Reducer**: 状态转换器（纯函数）

## 使用示例

```rust
use cxx_qt_mvi_core::prelude::*;

#[derive(Debug, Clone)]
pub enum MyIntent {
    Increment,
    Decrement,
}

impl Intent for MyIntent {}

#[derive(Debug, Clone, Default)]
pub struct MyState {
    pub count: i32,
}

impl State for MyState {}

pub struct MyReducer;

impl Reducer<MyState, MyIntent> for MyReducer {
    fn reduce(&self, state: MyState, intent: MyIntent) -> MyState {
        match intent {
            MyIntent::Increment => MyState { count: state.count + 1 },
            MyIntent::Decrement => MyState { count: state.count - 1 },
        }
    }
}
```

## 架构文档

详细架构设计请参考: `.kiro/steering/mvi-architecture.md`
