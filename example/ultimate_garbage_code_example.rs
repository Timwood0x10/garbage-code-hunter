// 史诗级垃圾代码示例 - 集合所有可检测的问题
// 这个文件故意写得很烂，用来测试 Garbage Code Hunter 的所有功能

use std::collections::HashMap;

// TODO: 重构这个文件
// FIXME: 修复所有问题
// XXX: 这里有严重问题
// HACK: 临时解决方案
// NOTE: 记住要优化
// BUG: 已知bug
// TODO: 添加错误处理
// FIXME: 性能问题

// 无意义命名 + 匈牙利命名法 + 过度缩写
static mut g_data: i32 = 42424242; // 全局变量 + 魔法数字
static strUserName: String = String::new(); // 匈牙利命名法
const MAGIC_NUM: i32 = 999999; // 魔法数字

// 被注释掉的代码块
// fn old_function() {
//     let old_var = "this is old";
//     if old_var.len() > 0 {
//         println!("Old logic");
//         for i in 0..10 {
//             println!("Loop: {}", i);
//         }
//     }
//     let another_old = 123;
//     match another_old {
//         1 => println!("one"),
//         _ => println!("other"),
//     }
// }

// 上帝函数 - 做太多事情，参数太多，复杂度极高
fn ultimate_god_function(
    strName: String,        // 匈牙利命名法 + String 滥用
    intAge: i32,           // 匈牙利命名法
    data: Vec<String>,     // 无意义命名 + Vec 滥用
    flag: bool,            // 无意义命名
    mgr: String,           // 过度缩写
    ctrl: i32,             // 过度缩写
    usr: String,           // 过度缩写
    pwd: String,           // 过度缩写
    cfg: HashMap<String, String>, // 过度缩写
) -> Result<String, String> {
    println!("Debug: entering function"); // println 调试
    println!("Debug: strName = {}", strName); // println 调试
    println!("Debug: intAge = {}", intAge); // println 调试
    
    // 魔法数字滥用
    if intAge > 150 {
        panic!("Age too high!"); // panic 滥用
    }
    
    if intAge < 0 {
        panic!("Negative age!"); // 又一个 panic
    }
    
    // String 滥用
    let temp = String::new(); // 无意义命名 + String 滥用
    let foo = String::from("hello"); // 无意义命名 + String 滥用
    let bar = "world".to_string(); // 无意义命名 + String 滥用
    let baz = format!("{}{}", foo.to_string(), bar.to_string()); // 无意义命名 + 过度 to_string
    
    println!("Debug: temp = {}", temp); // println 调试
    println!("Debug: foo = {}", foo); // println 调试
    
    // Vec 滥用
    let mut items = Vec::new(); // Vec 滥用
    let mut stuff = Vec::new(); // 无意义命名 + Vec 滥用
    let mut things = Vec::new(); // 无意义命名 + Vec 滥用
    let mut manager = Vec::new(); // 无意义命名 + Vec 滥用
    
    // 深度嵌套 + 复杂逻辑
    if flag {
        for i in 0..100 { // 魔法数字
            println!("Debug: i = {}", i); // println 调试
            if i % 7 == 0 { // 魔法数字
                if i > 50 { // 魔法数字
                    for item in &data {
                        println!("Debug: processing item"); // println 调试
                        if item.len() > 5 { // 魔法数字
                            for j in 0..20 { // 魔法数字
                                if j % 3 == 0 { // 魔法数字
                                    if j > 10 { // 魔法数字
                                        for k in 0..15 { // 魔法数字
                                            println!("Debug: deep nesting k = {}", k); // println 调试
                                            if k % 2 == 0 {
                                                if k > 8 { // 魔法数字
                                                    items.push(format!("item_{}", k));
                                                    if items.len() > 100 { // 魔法数字
                                                        panic!("Too many items!"); // panic 滥用
                                                        println!("This will never execute"); // 死代码
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Match 滥用 - 可以用 if let
    let maybe_value = Some(42); // 魔法数字
    match maybe_value {
        Some(x) => {
            println!("Debug: got value {}", x); // println 调试
        }
        None => {
            println!("Debug: no value"); // println 调试
        }
    }
    
    // 另一个 Match 滥用
    let result: Result<i32, &str> = Ok(123); // 魔法数字
    match result {
        Ok(value) => {
            println!("Debug: success {}", value); // println 调试
        }
        Err(error) => {
            println!("Debug: error {}", error); // println 调试
            panic!("Result error!"); // panic 滥用
        }
    }
    
    // 迭代器滥用 - 用循环代替迭代器
    let numbers = vec![1, 2, 3, 4, 5];
    let mut sum = 0;
    for num in &numbers {
        println!("Debug: adding {}", num); // println 调试
        sum += num;
    }
    
    // 另一个简单循环
    let mut evens = Vec::new(); // Vec 滥用
    for num in &numbers {
        if num % 2 == 0 {
            evens.push(*num);
        }
    }
    
    // 更多 String 滥用
    let processed = process_string_badly(strName.to_string());
    let final_result = format!("{}{}", processed.to_string(), baz.to_string());
    
    println!("Debug: final result = {}", final_result); // println 调试
    
    // 死代码
    if true {
        return Ok(final_result);
        println!("This is dead code"); // 死代码
        let dead_var = "never used"; // 死代码
    }
    
    unreachable!(); // 死代码
    println!("More dead code"); // 死代码
}

// String 参数滥用
fn process_string_badly(input: String) -> String { // 应该用 &str
    let temp = String::new(); // String 滥用
    let result = String::from("processed: "); // String 滥用
    format!("{}{}", result.to_string(), input.to_string()) // 过度 to_string
}

// 更多无意义命名和缩写
fn handle_usr_data(
    usr: String,     // 缩写
    pwd: String,     // 缩写
    mgr: String,     // 缩写
    ctrl: i32,       // 缩写
    btn: bool,       // 缩写
) {
    let foo = usr.to_string(); // 无意义命名 + String 滥用
    let bar = pwd.to_string(); // 无意义命名 + String 滥用
    let baz = mgr.to_string(); // 无意义命名 + String 滥用
    
    println!("Debug: foo = {}", foo); // println 调试
    println!("Debug: bar = {}", bar); // println 调试
    println!("Debug: baz = {}", baz); // println 调试
    
    if ctrl > 9999 { // 魔法数字
        panic!("Control value too high!"); // panic 滥用
    }
    
    // 简单的 match，可以用 if let
    let opt = Some(ctrl);
    match opt {
        Some(val) => println!("Control: {}", val),
        None => println!("No control"),
    }
}

// 主函数也很烂
fn main() {
    println!("Debug: starting main"); // println 调试
    
    // 无意义变量名
    let data = vec!["hello".to_string(), "world".to_string()]; // 无意义命名 + String 滥用
    let temp = String::from("test"); // 无意义命名 + String 滥用
    let stuff = 12345; // 无意义命名 + 魔法数字
    let thing = true; // 无意义命名
    let manager = String::new(); // 无意义命名 + String 滥用
    
    println!("Debug: data = {:?}", data); // println 调试
    println!("Debug: temp = {}", temp); // println 调试
    
    // 调用上帝函数
    let result = ultimate_god_function(
        temp.to_string(), // String 滥用
        stuff,
        data,
        thing,
        manager.to_string(), // String 滥用
        999, // 魔法数字
        "user123".to_string(), // String 滥用
        "password".to_string(), // String 滥用
        HashMap::new(),
    );
    
    // 简单的 match，可以用 if let
    match result {
        Ok(value) => {
            println!("Debug: success: {}", value); // println 调试
        }
        Err(error) => {
            println!("Debug: error: {}", error); // println 调试
            panic!("Main function failed!"); // panic 滥用
        }
    }
    
    // 更多垃圾代码
    handle_usr_data(
        "admin".to_string(), // String 滥用
        "secret".to_string(), // String 滥用
        "boss".to_string(), // String 滥用
        88888, // 魔法数字
        false,
    );
    
    println!("Debug: ending main"); // println 调试
    
    // unimplemented!(); // 如果取消注释会被检测到
    // todo!(); // 如果取消注释会被检测到
}