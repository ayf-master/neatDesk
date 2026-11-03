# Rust 速成指南 — 针对 NeatDesk (cleanApp)

> 目标：让你在最短时间内看懂 cleanApp 的所有 Rust 代码，并能动手修改、增删功能。
> 假设：零 Rust 基础，有任意一门编程语言经验（JS/Python/Java 都行）。

---

## 第一课：Rust 到底长什么样？

光说不练没用，直接看一段 cleanApp 的代码。下面这个函数**扫描桌面文件**：

```rust
// scanner.rs
pub fn scan_desktop(desktop_path: &str) -> Result<Vec<FileInfo>, String> {
    let dir = Path::new(desktop_path);
    if !dir.exists() {
        return Err(format!("桌面路径不存在: {}", desktop_path));
    }

    let entries = std::fs::read_dir(dir).map_err(|e| format!("读取桌面失败: {}", e))?;

    let mut files = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();
        if path.is_dir() { continue; }

        let name = entry.file_name().to_string_lossy().to_string();
        // ... 更多处理
        files.push(FileInfo { name, /* ... */ });
    }

    Ok(files)
}
```

**你能立刻看懂的东西：**
- `pub fn` = 公开函数
- `-> Result<Vec<FileInfo>, String>` = 返回"要么成功得到文件列表，要么失败给个错误消息"
- `let` = 声明变量（不可变）
- `let mut` = 可变变量
- `for x in y` = 遍历
- `match` = switch/case
- `if` = if
- `//` = 注释

**跟 JS/Python 明显不同的地方：**
- `Path::new(desktop_path)` — 双冒号 `::` 表示"静态方法调用"或"命名空间下的东西"
- `.map_err(|e| format!(...))?` — 问号 `?` 表示"如果出错就提前返回错误"
- `Entry::FileInfo { name, ... }` — 结构体花括号构造
- `String` 和 `&str` — 前者是拥有所有权的字符串，后者是字符串的"引用/借用"

看不懂上面的细节没关系——这篇文档就是用来解释它们的。

---

## 第二课：安装 Rust 开发环境

```bash
# 1. 安装 rustup（Rust 版本管理器）
# 去 https://rustup.rs/ 下载，或者：
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. 验证安装
rustc --version   # 编译器
cargo --version   # 包管理器（类似 npm）

# 3. 项目已经在用 Tauri，需要额外工具
# Windows: 装好 Visual Studio 的 "Desktop development with C++" 工作负载
# 或者装 Microsoft C++ Build Tools: https://visualstudio.microsoft.com/visual-cpp-build-tools/
```

**在 cleanApp 项目里编译运行：**
```bash
cd D:\vibeCoding\cleanApp

# 安装前端依赖
npm install

# 启动开发模式（Vite + Tauri 同时运行）
npm run tauri dev

# 仅编译 Rust 部分（不启动 UI）
cd src-tauri && cargo build
```

**Cargo 基本命令（类比 npm）：**
```bash
cargo build           # npm run build
cargo run             # npm start（运行 Rust 项目）
cargo check           # 只检查语法，不生成二进制，比 build 快
cargo test            # 跑测试
cargo add 包名         # npm install 包名（添加依赖）
cargo doc --open      # 生成文档并打开
```

---

## 第三课：变量与类型系统

### 变量声明

```rust
let x = 5;              // 不可变（类似 const）
let mut y = 10;         // 可变（类似 let）
y = 20;                 // OK

// 类型标注（Rust 通常能自动推断，但你可以显式写）：
let name: String = String::from("张三");
let count: u32 = 42;    // u32 = 无符号 32 位整数
let is_ok: bool = true;
```

### 基本类型

| 类型 | 含义 | 示例 |
|------|------|------|
| `u8`, `u16`, `u32`, `u64` | 无符号整数 | `let x: u32 = 100;` |
| `i8`, ..., `i64` | 有符号整数 | `let y: i64 = -5;` |
| `f32`, `f64` | 浮点数 | `let z: f64 = 3.14;` |
| `bool` | 布尔 | `let b = true;` |
| `char` | Unicode 字符 | `let c = '中';` |
| `String` | 拥有所有权的字符串 | `let s = String::from("hello");` |
| `&str` | 字符串切片（引用） | `let s: &str = "hello";` |

### 类型转换

```rust
// 数字类型转换用 as
let a: u32 = 42;
let b: f64 = a as f64;

// 数字→字符串
let s = 42.to_string();
let s = format!("文件数量: {}", count);

// 字符串→数字
let n: u32 = "42".parse().unwrap_or(0);
```

### → 在 cleanApp 中的体现

`config.rs` — 整篇几乎都在声明结构体类型和它们的字段类型：
```rust
pub struct AppConfig {
    pub version: u32,                    // 无符号整数
    pub categories: Vec<Category>,       // Vec = 动态数组
    pub schedule: Schedule,
    pub exclusions: Exclusions,
    pub settings: Settings,
}
```

---

## 第四课：函数定义

### 基本语法

```rust
// 无返回值
fn do_something() {
    // ...
}

// 有参数 + 有返回值
fn add(a: i32, b: i32) -> i32 {
    a + b    // 最后一个表达式就是返回值（不加分号！）
}

// 等价写法：
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}
```

**关键区别**：Rust 中**最后一个表达式就是返回值**，不需要 `return`。只有需要提前返回时才用 `return`。

### 范型函数

```rust
// T 代表"任意类型"
fn first<T>(list: Vec<T>) -> T {
    list.into_iter().next().unwrap()
}
```

### → 在 cleanApp 中的体现

几乎每个 `.rs` 文件都有函数。看 `config.rs` 的 `save_config`：

```rust
pub fn save_config(config_dir: &Path, config: &AppConfig) -> Result<(), String> {
    let config_file = config_dir.join("config.json");
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(&config_file, json).map_err(|e| e.to_string())?;
    Ok(())
}
```

分解这个函数：
- 参数 `config_dir: &Path` — 对 `Path` 类型的**引用**
- 参数 `config: &AppConfig` — 对 `AppConfig` 类型的引用
- 返回 `Result<(), String>` — 成功返回空 `()`，失败返回 `String`
- `?` — 如果 `serde_json::to_string_pretty` 或 `std::fs::write` 出错，就自动 return 错误

---

## 第五课：所有权 — Rust 最独特的概念

> **一句话总结"所有权"：每个数据只有一个"主人"，主人离开作用域时数据被自动释放。**

### 三条铁律

1. **每个值在 Rust 中都有一个所有者（owner）**
2. **同一时间只能有一个所有者**
3. **所有者离开作用域，值被自动释放**

### 看代码理解

```rust
let s1 = String::from("hello");
let s2 = s1;              // 所有权从 s1 "移动" 到了 s2
// println!("{}", s1);     // ❌ 编译错误！s1 已经"被移动"了

let s3 = s2.clone();      // ✅ 克隆一份，s2 仍然有效
println!("{}", s2);        // ✅ 可以
```

### 借用（引用）

不想转移所有权？用 `&` 借用：

```rust
fn compute_length(s: &String) -> usize {  // & 表示"借用"
    s.len()
}  // 这里 s 被释放（因为是借用，不释放底层数据）

let s = String::from("hello");
let len = compute_length(&s);  // & 创建引用
println!("{}", s);              // ✅ s 仍可用
```

### 可变引用

```rust
let mut s = String::from("hello");
fn change(s: &mut String) {    // &mut = 可变引用
    s.push_str(", world");
}
change(&mut s);
```

**重要规则**：同一时间只能有一个可变引用（或多个不可变引用）。这是 Rust 防止数据竞争的方式。

### → 在 cleanApp 中的体现

```rust
// 扫描器：&str = 借用字符串，不拿走所有权
pub fn scan_desktop(desktop_path: &str) -> Result<Vec<FileInfo>, String> {
    // desktop_path 在这里只读使用
}  // 这里不释放 desktop_path，调用者还能继续用

// 分类器：&[Category] = 借用 Category 数组切片
pub fn classify(files: Vec<FileInfo>, categories: &[Category]) -> HashMap<String, Vec<FileInfo>> {
    // files: Vec<FileInfo> — 所有权被拿走了！
    // categories: &[Category] — 只是借用，不影响调用方
}
```

问：为什么 `files` 拿走所有权，`categories` 只借用？
答：`classify` 要把 `files` 分配到不同类别中（会移动它们），所以需要所有权。`categories` 只需要读取分类规则，不需要修改，借就够了。

---

## 第六课：结构体 — 自定义数据类型

### 定义和使用

```rust
// 定义（类似 JS 的 class / TS 的 interface）
struct FileInfo {
    name: String,
    extension: String,
    path: String,
    size: u64,
    modified: String,
    is_hidden: bool,
}

// 创建实例
let info = FileInfo {
    name: String::from("report.docx"),
    extension: String::from(".docx"),
    path: String::from("C:/Users/me/Desktop/report.docx"),
    size: 1024 * 1024,
    modified: String::from("2026-06-24 15:30:00"),
    is_hidden: false,
};

// 访问字段
println!("文件名: {}", info.name);
```

### 元组结构体

```rust
struct Color(u8, u8, u8);  // 没有字段名，只有类型
let red = Color(255, 0, 0);
println!("R: {}", red.0);
```

### → 在 cleanApp 中的体现

`config.rs` 定义了 6 个结构体，全部用 `#[derive(Debug, Serialize, Deserialize, Clone)]` 装饰：

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub extensions: Vec<String>,
    pub name_patterns: Vec<String>,
    pub target_path: String,
    pub enabled: bool,
}
```

**derive 上的属性是什么？**
- `Debug` — 允许用 `{:?}` 调试打印
- `Serialize, Deserialize` — 允许序列化为 JSON（serde 库）
- `Clone` — 允许显式克隆（`.clone()`）

**pub 关键字：**
- `pub struct` — 结构体对外可见
- `pub field: type` — 字段对外可见
  - Rust 默认是私有的（类似 `private`）

---

## 第七课：枚举与模式匹配

### 枚举定义

```rust
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

enum Message {
    Quit,
    Move { x: i32, y: i32 },   // 带命名字段
    Write(String),              // 带匿名数据
}
```

### 模式匹配

Rust 的 `match` 非常强大——必须覆盖所有分支：

```rust
let dir = Direction::Up;

match dir {
    Direction::Up => println!("上"),
    Direction::Down => println!("下"),
    Direction::Left => println!("左"),
    Direction::Right => println!("右"),
    // 不需要 default，因为 4 个分支已覆盖全部可能
}
```

### 最常用的枚举：Result 和 Option

**Result** — 可能成功或失败：
```rust
enum Result<T, E> {
    Ok(T),      // 成功，附带值
    Err(E),     // 失败，附带错误
}
```

**Option** — 可能有值，可能为空（没有 null！）：
```rust
enum Option<T> {
    Some(T),    // 有值
    None,       // 没值
}
```

```rust
// 使用 Result
fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("不能除以零".into())
    } else {
        Ok(a / b)
    }
}

match divide(10.0, 0.0) {
    Ok(result) => println!("结果: {}", result),
    Err(msg) => println!("错误: {}", msg),
}

// 使用 Option
let name = Some("张三");
let empty: Option<&str> = None;

match name {
    Some(n) => println!("你好, {}", n),
    None => println!("没有名字"),
}
```

### 问号运算符

`?` 是 Rust 最常用的语法糖——"如果出错就提前返回"：

```rust
fn read_file(path: &str) -> Result<String, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("读文件失败: {}", e))?;  // 如果 Err，立即 return
    Ok(content)
}

// 等价于：
fn read_file(path: &str) -> Result<String, String> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => return Err(format!("读文件失败: {}", e)),
    };
    Ok(content)
}
```

### → 在 cleanApp 中的体现

cleanApp 几乎所有函数都返回 `Result<T, String>`。用 `?` 链式处理错误：

```rust
// commands.rs —— 全篇都是 Result<..., String>
pub fn organize_desktop(state: State<'_, Mutex<AppState>>) -> Result<serde_json::Value, String> {
    let app_state = state.lock().map_err(|e| e.to_string())?;
    let config = config::load_config(&app_state.config_dir);
    let desktop = crate::scanner::get_desktop_path()?;
    let files = crate::scanner::scan_desktop(&desktop)?;
    let files = crate::exclusions::filter(files, &config.exclusions);
    let classified = crate::classifier::classify(files, &config.categories);
    let result = crate::organizer::organize(&desktop, classified, &config.categories)?;
    // ... 继续
    Ok(serde_json::json!({...}))
}
```

每步都可能失败（路径不存在、读取失败、移动失败），到处用 `?` 传播错误。

---

## 第八课：Vec（动态数组）与迭代器

### 基本操作

```rust
// 创建
let mut v: Vec<i32> = Vec::new();
let mut v = vec![1, 2, 3];  // 宏

// 增删改
v.push(4);           // 追加
v.pop();             // 弹出末尾
v.remove(0);         // 删除索引 0
v[0] = 100;          // 修改

// 遍历
for item in &v {
    println!("{}", item);
}
```

### 迭代器链式操作

Rust 的迭代器跟 JS 的 `array.map().filter()` 类似：

```rust
// JS:  numbers.filter(n => n > 0).map(n => n * 2)
// Rust:
numbers.iter()
    .filter(|n| **n > 0)
    .map(|n| n * 2)
    .collect::<Vec<_>>();

// 常用迭代器方法：
.iter()          // 遍历不可变引用
.into_iter()     // 遍历并拿走所有权
.filter(|x| ...) // 过滤
.map(|x| ...)    // 转换
.any(|x| ...)    // 任一匹配
.all(|x| ...)    // 全部匹配
.find(|x| ...)   // 找第一个
.count()         // 计数
.collect()       // 收集为集合
```

### → 在 cleanApp 中的体现

```rust
// exclusions.rs —— filter 的经典用法
pub fn filter(files: Vec<FileInfo>, exclusions: &Exclusions) -> Vec<FileInfo> {
    files.into_iter()               // 拿走文件的所有权开始遍历
        .filter(|f| {
            if f.name.starts_with('.') { return false; }
            // ... 更多过滤条件
            true
        })
        .collect()                  // 收集过滤结果
}
```

```rust
// classifier.rs —— 用 any() 检查是否匹配
if cat.extensions.iter().any(|ext| ext.eq_ignore_ascii_case(&file.extension)) {
    result.get_mut(&cat.id).unwrap().push(file.clone());
    classified = true;
    break;
}
```

---

## 第九课：impl — 给类型添加方法

### 基本语法

```rust
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    // 方法（第一个参数是 &self）
    fn area(&self) -> u32 {
        self.width * self.height
    }

    // 关联函数（类似静态方法，没有 &self）
    fn square(size: u32) -> Rectangle {
        Rectangle { width: size, height: size }
    }
}

// 使用
let rect = Rectangle { width: 10, height: 20 };
println!("面积: {}", rect.area());           // 方法调用

let sq = Rectangle::square(5);               // 关联函数调用（双冒号）
```

### → 在 cleanApp 中的体现

```rust
// config.rs —— 给 AppConfig 实现默认值
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: 1,
            categories: vec![
                Category { id: "images".into(), name: "图片".into(), /* ... */ },
                // ...
            ],
            // ...
        }
    }
}
```

```rust
// scheduler.rs —— 给 Scheduler 实现方法
impl Scheduler {
    pub fn new() -> Self {
        Self { running: Arc::new(AtomicBool::new(false)) }
    }

    pub fn start(&self, app: AppHandle, state: Arc<Mutex<AppState>>) {
        // ...
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}
```

注意区分两种语法：
- `impl Default for AppConfig` — 为 `AppConfig` **实现** `Default` trait（类似"实现接口"）
- `impl Scheduler` — 为 `Scheduler` **添加**方法

---

## 第十课：use、mod 与模块系统

### 模块声明

```rust
// lib.rs —— 声明所有子模块
pub mod commands;       // 声明 commands.rs 为公开模块
pub mod config;
pub mod scanner;
mod scheduler;          // 没有 pub，模块只在 crate 内可见
mod compact;
```

### use 导入

```rust
// 绝对路径（从 crate 根开始）
use crate::config::AppConfig;

// 当前模块相对路径
use super::config;      // 上一级目录的 config
use self::helper;       // 同级模块

// 简化写法
use std::collections::HashMap;
use std::path::Path;
```

### 路径分隔符：双冒号 `::`

```rust
std::fs::read_to_string(path)  // 标准库 → fs 模块 → read_to_string 函数
crate::scanner::scan_desktop   // 当前 crate → scanner 模块 → scan_desktop 函数
```

### → 在 cleanApp 中的体现

```rust
// commands.rs
use tauri::State;
use crate::config;              // 引入 config 模块
use std::sync::Mutex;

// lib.rs——相当于项目根
pub mod commands;               // 声明子模块
pub mod config;
pub mod scanner;
// 然后 commands.rs 里就能用 crate::config::load_config()
```

**包的结构（类比 package.json）：**
```
Cargo.toml      → package.json（项目配置、依赖）
lib.rs          → 库入口（类似 index.ts）
main.rs         → 可执行文件入口
```

`main.rs` 调用 `desktop_clean_lib::run()`，在 `Cargo.toml` 中定义为：
```toml
[lib]
name = "desktop_clean_lib"
crate-type = ["lib"]
```

---

## 第十一课：常用标准库速查

### 文件系统 — `std::fs`

```rust
use std::path::Path;

// 读文件
std::fs::read_to_string("config.json")   // → Result<String>
std::fs::read("file.bin")                // → Result<Vec<u8>>

// 写文件
std::fs::write("config.json", content)   // → Result<()>

// 目录操作
std::fs::create_dir_all("a/b/c")         // 递归创建
std::fs::read_dir(".")                   // 读取目录 → 迭代器
std::fs::remove_dir("dir")               // 删空目录
std::fs::rename("old.txt", "new.txt")    // 移动/重命名

// Path 常用方法
Path::new("C:/Users/file.txt")
    .exists()           // 是否存在
    .is_dir()           // 是否目录
    .is_file()          // 是否文件
    .parent()           // 父目录 → Option<&Path>
    .join("sub")        // 拼接路径
    .extension()        // 扩展名 → Option<&OsStr>
    .file_stem()        // 文件名（不含扩展名）
    .starts_with(other) // 是否以 other 开头
```

### 环境变量

```rust
std::env::var("USERPROFILE")    // → Result<String>
std::env::current_exe()         // → Result<PathBuf>（当前程序路径）
```

### 集合

```rust
// Vec — 动态数组
let mut v = vec![1, 2, 3];
v.push(4);
v.len();           // 长度
v.is_empty();      // 是否为空

// HashMap — 哈希表
use std::collections::HashMap;
let mut map = HashMap::new();
map.insert("key".to_string(), value);
map.get("key");                       // → Option<&V>
map.entry("key".to_string()).or_default();  // 不存在就插入默认值
```

---

## 第十二课：serde + JSON — 序列化/反序列化

serde 是 Rust 事实上的序列化框架。在 cleanApp 中，所有配置文件和 Tauri 命令的数据交换都靠它。

### 派生子

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Person {
    name: String,
    age: u8,
    #[serde(rename = "type")]     // 字段别名（JSON 中叫 "type"）
    person_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,         // 可选字段
}
```

### 核心函数

```rust
// 结构体 → JSON 字符串
let json = serde_json::to_string(&person)?;           // 紧凑
let json = serde_json::to_string_pretty(&person)?;    // 带缩进

// JSON 字符串 → 结构体
let person: Person = serde_json::from_str(&json)?;

// 动态 JSON（不定义结构体时）
use serde_json::{json, Value};
let v: Value = json!({
    "name": "张三",
    "age": 20,
    "items": [1, 2, 3]
});
v["name"].as_str();      // → Option<&str>
v["items"].as_array();   // → Option<&Vec<Value>>
```

### → 在 cleanApp 中的体现

```rust
// config.rs —— 全套配置用 serde JSON 序列化
pub fn save_config(config_dir: &Path, config: &AppConfig) -> Result<(), String> {
    let config_file = config_dir.join("config.json");
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(&config_file, json).map_err(|e| e.to_string())?;
    Ok(())
}
```

Tauri 命令返回给前端的也是 JSON：
```rust
Ok(serde_json::json!({
    "total_files": result.total_files,
    "success_count": result.success_count,
    "categories": result.categories,
    "errors": result.errors,
}))
```

---

## 第十三课：Tauri 命令 — Rust 和前端怎么通信

这是你改代码最需要理解的部分。

### 后端（Rust）定义一个命令

```rust
// commands.rs
#[tauri::command]
pub fn get_config(state: State<'_, Mutex<AppState>>) -> Result<AppConfig, String> {
    let app_state = state.lock().map_err(|e| e.to_string())?;
    Ok(config::load_config(&app_state.config_dir))
}
```

关键点：
- `#[tauri::command]` 标记这是一个 Tauri 命令
- 参数可以是 `State`（从 Tauri 获取共享状态）、基本类型 (`String`, `bool`)、或任何实现了 `Deserialize` 的类型
- 返回值可以是 `Result<T, String>`（成功/失败）或直接 `T`

### 注册命令

```rust
// lib.rs
.invoke_handler(tauri::generate_handler![
    commands::get_config,       // 注册命令
    commands::update_config,
    commands::organize_desktop,
    // ...
])
```

### 前端（Vue/TS）调用

```typescript
// api.ts
import { invoke } from "@tauri-apps/api/core";

// 调用无参数命令
const config = await invoke("get_config");

// 调用带参数命令
await invoke("update_config", { newConfig: config });
//              ↑命令名            ↑参数（camelCase 自动转 snake_case）

// 调用带 i64 参数的命令（JS 数字）
await invoke("undo_organize", { recordId: 123 });
```

**命名转换规则**：Rust 的 `snake_case`（`delete_history_record`）在前端调用时也是 `snake_case`，但参数名 Rust 是 `snake_case`，前端传 `camelCase`（Tauri 自动转换）。

### AppState — 共享状态

```rust
// commands.rs 定义状态
pub struct AppState {
    pub config_dir: PathBuf,     // 配置目录路径
    pub data_dir: PathBuf,       // 数据目录路径
}

// lib.rs 初始化并注册
let app_state = AppState {
    config_dir: config_path,
    data_dir: data_path,
};
app.manage(Mutex::new(app_state));

// 命令中获取
#[tauri::command]
pub fn get_config(state: State<'_, Mutex<AppState>>) -> ... {
    let app_state = state.lock().map_err(|e| e.to_string())?;
    // 用 app_state.config_dir / app_state.data_dir
}
```

---

## 第十四课：Mutex、Arc 与原子类型 — Rust 并发

### Mutex — 互斥锁

多个代码块不能同时访问同一数据时，用 Mutex 包起来：

```rust
use std::sync::Mutex;

let counter = Mutex::new(0);

{
    let mut num = counter.lock().unwrap();  // 获取锁
    *num += 1;                               // 修改
}  // 锁在这里自动释放
```

在 cleanApp 里，因为 Tauri 命令可能被多个线程调用，`AppState` 被包在 `Mutex` 里：
```rust
// 在 lib.rs
app.manage(Mutex::new(app_state));

// 在 commands.rs
#[tauri::command]
pub fn get_config(state: State<'_, Mutex<AppState>>) -> Result<AppConfig, String> {
    let app_state = state.lock().map_err(|e| e.to_string())?;
    //                 ↑ 获取锁，其他线程必须等在这里
    Ok(config::load_config(&app_state.config_dir))
}  // app_state 离开作用域，自动释放锁
```

### Arc — 原子引用计数

`Arc` 让同一份数据可以在多个线程间共享所有权：

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

let shared_flag = Arc::new(AtomicBool::new(true));

// 克隆 Arc（不克隆底层数据，只增加引用计数）
let flag_clone = shared_flag.clone();

// 在另一个线程中使用
std::thread::spawn(move || {
    if flag_clone.load(Ordering::SeqCst) {
        // ...
    }
});
```

### → 在 cleanApp 中的体现

```rust
// scheduler.rs
pub struct Scheduler {
    running: Arc<AtomicBool>,  // 跨线程共享的运行标志
}

impl Scheduler {
    pub fn start(&self, app: AppHandle, state: Arc<Mutex<AppState>>) {
        let running = self.running.clone();   // Arc 克隆
        tauri::async_runtime::spawn(async move {
            run_scheduler(app, state, running).await;
        });
    }
}
```

---

## 第十五课：闭包

Rust 的闭包类似 JS 的箭头函数：

```javascript
// JS
const add = (a, b) => a + b;
const filtered = arr.filter(x => x > 0);
```

```rust
// Rust
let add = |a: i32, b: i32| a + b;      // 参数在 || 里
let filtered: Vec<_> = arr.into_iter()
    .filter(|x| *x > 0)                  // 闭包
    .collect();
```

闭包可以捕获外部变量：
```rust
let threshold = 10;
let is_big = |x: &i32| *x > threshold;  // 捕获 threshold
println!("{}", is_big(&20));              // true
```

闭包有三种捕获方式：
- `Fn` — 不可变借用（只读）
- `FnMut` — 可变借用（可修改）
- `FnOnce` — 拿走所有权（只能调用一次）

**Thread spawn 要求闭包是 `FnOnce` + `Send`**（可以跨线程移动），所以需要 `move` 关键字：
```rust
std::thread::spawn(move || {
    // 这里的变量所有权被移入闭包
});
```

---

## 第十六课：条件编译 — `#[cfg()]`

根据平台条件编译不同代码：

```rust
#[cfg(windows)]
fn windows_only() {
    println!("只有 Windows 才编译这个函数");
}

#[cfg(not(windows))]
fn windows_only() {
    println!("非 Windows 平台用这个替代");
}
```

### → 在 cleanApp 中的体现

```rust
// compact.rs —— 整个文件只在 Windows 编译
#![cfg(windows)]

// scanner.rs —— 平台特定的隐藏文件检测
#[cfg(windows)]
fn is_windows_hidden(metadata: &std::fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
    let attrs = metadata.file_attributes();
    (attrs & FILE_ATTRIBUTE_HIDDEN) != 0
}

#[cfg(not(windows))]
fn is_windows_hidden(_metadata: &std::fs::Metadata) -> bool {
    false  // 非 Windows 平台，没有隐藏属性概念
}
```

---

## 第十七课：unsafe — 调用 C 接口（FFI）

Rust 通常保证内存安全，但调用 Windows API 时必须用 `unsafe`：

```rust
extern "system" {
    fn FindWindowW(lpClassName: LPCWSTR, lpWindowName: LPCWSTR) -> HWND;
    fn SetWindowLongW(hWnd: HWND, nIndex: i32, dwNewLong: isize) -> isize;
}

fn do_something() {
    unsafe {
        let hwnd = FindWindowW(ptr::null(), ptr::null());
        SetWindowLongW(hwnd, -16, 0x0100);
    }
}
```

- `extern "system"` — 声明外部函数（来自 Windows DLL）
- `unsafe { }` 块 — 告诉编译器"我知道可能有风险，相信我"
- 类型别名如 `type HWND = isize;` — Windows API 的 C 类型映射

→ 见 `compact.rs` 和 `lib.rs` 中的单例检测。

---

## 第十八课：tokio 异步 — async/await

虽然 cleanApp 的 Tauri 部分主要用同步代码，但 `scheduler.rs` 用了 tokio：

```rust
use tokio::time::{self, Duration};

async fn run_scheduler(...) {
    let mut interval = time::interval(Duration::from_secs(30));
    while running.load(Ordering::SeqCst) {
        interval.tick().await;          // 等待 30 秒
        // ... 执行检查
    }
}

// 用 spawn 启动异步任务
tauri::async_runtime::spawn(async move {
    run_scheduler(app, state, running).await;
});
```

**`async` 函数**不是直接在当前线程执行，而是返回一个"未来值"。需要用 `.await` 或 `spawn` 来驱动它执行。

---

## 第十九课：常用语法糖与惯用法

### `.into()` / `.to_string()`

`into()` 是 Rust 最常用的转换语法糖——"转换成目标类型"：

```rust
let s: String = "hello".into();                    // &str → String
let v: Vec<String> = vec!["a".into(), "b".into()]; // 批量转换

// 等价于：
let s = String::from("hello");
```

`to_string()` 是另一个常用的转换：
```rust
let num = 42;
let s = num.to_string();       // "42"
let json = serde_json::to_string(&config)?;
```

### `unwrap_or_default()` / `.unwrap_or()` / `.unwrap()`

```rust
let val = option_value.unwrap_or(42);               // 有值取之，无值用 42
let val = result_value.unwrap_or_default();          // 有值取之，无值用类型的默认值
let val = option_value.unwrap();                     // 有值取之，无值 panic!
let val = option_value.unwrap_or_else(|| compute()); // 有值取之，无值执行闭包
```

**生产代码中 `unwrap()` 要少用**（会 panic 崩溃）。cleanApp 里只在确定不会失败的地方用（比如 `Default` 实现中）。

### `eprintln!` vs `println!`

```rust
println!("输出到 stdout（标准输出）");  // 前端通常看不到
eprintln!("输出到 stderr（标准错误）");   // 调试信息
```

### `format!` 宏

类似 JS 的模板字符串或 Python 的 f-string：

```rust
let name = "张三";
let age = 20;
let s = format!("我叫{}，今年{}岁", name, age);
// → "我叫张三，今年20岁"
```

### `.as_str()` / `.to_string_lossy()`

```rust
// Path → &str
let path = Path::new("C:/test.txt");
let s: &str = path.to_str().unwrap();     // 可能失败（非 UTF-8 路径）
let s = path.to_string_lossy();           // 不会失败，非法字符替换为 �

// String → &str
let s = String::from("hello");
let r: &str = s.as_str();
```

---

## 第二十课：rust-analyzer — 开发利器

在 VS Code 中装 `rust-analyzer` 扩展，它能：

- **悬停类型** — 鼠标悬停在变量上看它的类型
- **自动补全** — 代码提示
- **实时错误** — 输入时就能看到编译错误
- **跳转定义** — 点击函数名跳转到它的定义
- **类型标注内联** — 自动显示函数返回类型、变量类型

**强烈建议**：写 Rust 时开着 rust-analyzer，它比任何教程都有用。

---

## 附录：cleanApp 各文件速查

| 文件 | 作用 | 关键概念 |
|------|------|----------|
| `main.rs` | 程序入口，处理 `--quick-organize` 参数 | CLI args, 模块调用 |
| `lib.rs` | Tauri 应用组装，注册所有命令 | Builder 模式, 插件, `Mutex` |
| `commands.rs` | 18 个 Tauri 命令，前后端桥梁 | `#[tauri::command]`, `State`, `serde_json::json!` |
| `config.rs` | 配置结构体 + JSON 读写 | `struct`, `derive`, serde, `Default` trait |
| `scanner.rs` | 扫描桌面文件 | `std::fs::read_dir`, 平台特定 `#[cfg]` |
| `classifier.rs` | 按关键词/扩展名分类 | `HashMap`, 迭代器链式操作 |
| `organizer.rs` | 移动文件到分类文件夹 | `std::fs::rename`, 错误码处理 |
| `exclusions.rs` | 按排除列表过滤 | 迭代器 `.filter()`, `.any()` |
| `undo.rs` | SQLite 记录 + 撤销 | `rusqlite`, serde JSON 序列化 |
| `scheduler.rs` | 定时自动整理 | tokio async/await, `Arc<AtomicBool>` |
| `tray.rs` | 系统托盘 + 右键菜单 | Tauri Menu/Tray API, 图片生成 |
| `compact.rs` | 桌面图标紧凑排列 | Win32 FFI, `unsafe`, `extern "system"` |

---

## 动手练习：自己改改试试

学完这些，试试改这些简单的东西来练手：

1. **给配置加个新字段**：在 `Settings` 结构体里加一个 `auto_organize: bool`，前端配置页加个开关
2. **加个新 Tauri 命令**：`src-tauri/src/commands.rs` 加一个 `greet(name: String) -> String`，在 `lib.rs` 的 `generate_handler!` 里注册，前端调用
3. **换个分类逻辑**：在 `classifier.rs` 里改一下匹配优先级
4. **加个扫描排除项**：在 `scanner.rs` 的 `scan_desktop` 里加一个新文件跳过条件

**调试小技巧：**
- 编译错误看不懂？把错误复制进 https://chatgpt.com 或问 AI
- 不确定类型？装 `rust-analyzer` 悬停查看
- `cargo check` 比 `cargo build` 快得多，先 check 再 build
- 忘记某个语法？在 cleanApp 代码库里搜关键词找例子

---

> 这篇文档不追求全覆盖——只讲 cleanApp 里用到的 Rust 特性。当你开始自己写新功能时，最常用的几个操作就是：定义结构体（`struct`）、读写 JSON（serde）、操作文件（`std::fs`）、加 Tauri 命令（`#[tauri::command]`）。上手之后，Rust 的类型系统反而会成为你的保护网——编译通过 = 大概率没 bug。
