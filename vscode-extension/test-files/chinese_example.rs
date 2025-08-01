// 这是一个包含中文注释的测试文件
// 用来测试智能语言检测功能

fn main() {
    // 无意义的变量名
    let data = "hello world";  // 这个变量名太模糊了
    let temp = 42;             // 临时变量名不好
    let foo = vec![1, 2, 3];   // foo 是什么意思？
    
    // 匈牙利命名法（不推荐）
    let strName = "张三";      // 应该用 name 而不是 strName
    let intAge = 25;           // 应该用 age 而不是 intAge
    
    // 过度缩写
    let mgr = "经理";          // manager 比 mgr 更清楚
    let usr = "用户";          // user 比 usr 更清楚
    
    // 调试打印（应该用日志）
    println!("调试信息: data = {}", data);
    println!("调试信息: temp = {}", temp);
    
    // 不安全的 unwrap 使用
    let result = Some("测试");
    let value = result.unwrap();  // 应该用 match 或 if let
    
    // 不必要的克隆
    let original = String::from("原始字符串");
    let copy = original.clone();        // 可能不需要克隆
    let another_copy = copy.clone();    // 又一个不必要的克隆
    
    // 深层嵌套（代码可读性差）
    if true {
        if true {
            if true {
                if true {
                    println!("嵌套太深了！");
                }
            }
        }
    }
    
    // 魔法数字（应该定义常量）
    let magic = 42;           // 这个 42 是什么意思？
    let pi = 3.14159;         // 应该用 std::f64::consts::PI
    
    // 单字母变量滥用
    let a = 1;  // 变量名太短
    let b = 2;  // 没有意义
    let c = 3;  // 不知道用途
    
    process_data(data, temp, foo, value, copy, another_copy, magic, pi, a, b, c);
}

// 上帝函数 - 参数太多，功能太复杂
fn process_data(
    data: &str,      // 数据
    temp: i32,       // 临时值
    foo: Vec<i32>,   // 列表
    value: &str,     // 值
    copy: String,    // 拷贝1
    another_copy: String,  // 拷贝2
    magic: i32,      // 魔法数字
    pi: f64,         // 圆周率
    a: i32,          // 未知变量A
    b: i32,          // 未知变量B
    c: i32,          // 未知变量C
) {
    println!("处理所有数据...");
    
    // 更多问题代码
    let numbers = vec![1, 2, 3, 4, 5];
    for i in 0..numbers.len() {  // 应该用迭代器
        println!("{}", numbers[i]);
    }
    
    // 复杂的 match，应该用 if let
    match Some(42) {
        Some(x) => println!("得到 {}", x),
        None => {},  // 空分支
    }
}

/* 
这是一段被注释掉的代码
fn old_function() {
    let old_data = "旧数据";
    println!("这个函数不再使用了");
    // 更多旧代码
    let unused = 42;
}
*/