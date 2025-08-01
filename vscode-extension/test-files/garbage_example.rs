// 这是一个故意写得很"垃圾"的 Rust 文件，用来测试插件功能

fn main() {
    // 无意义的变量名
    let data = "hello world";
    let temp = 42;
    let foo = vec![1, 2, 3];
    let bar = String::new();
    
    // 匈牙利命名法
    let strName = "John";
    let intAge = 25;
    let bIsValid = true;
    
    // 过度缩写
    let mgr = "manager";
    let ctrl = "controller";
    let usr = "user";
    
    // Printf 调试
    println!("Debug: data = {}", data);
    println!("Debug: temp = {}", temp);
    
    // Unwrap 滥用
    let result = Some("test");
    let value = result.unwrap();
    
    // 不必要的 clone
    let original = String::from("hello");
    let copy = original.clone();
    let another_copy = copy.clone();
    
    // 深层嵌套
    if true {
        if true {
            if true {
                if true {
                    println!("Too deep!");
                }
            }
        }
    }
    
    // 魔法数字
    let magic = 42;
    let another_magic = 3.14159;
    
    // TODO 注释过多
    // TODO: fix this
    // TODO: optimize this
    // TODO: refactor this
    
    // 长函数（这个函数做了太多事情）
    process_everything(data, temp, foo, bar, strName, intAge, bIsValid, mgr, ctrl, usr, value, copy, another_copy, magic, another_magic);
}

// 上帝函数 - 做太多事情
fn process_everything(
    data: &str,
    temp: i32, 
    foo: Vec<i32>,
    bar: String,
    strName: &str,
    intAge: i32,
    bIsValid: bool,
    mgr: &str,
    ctrl: &str,
    usr: &str,
    value: &str,
    copy: String,
    another_copy: String,
    magic: i32,
    another_magic: f64
) {
    // 这个函数参数太多了
    println!("Processing everything...");
    
    // 更多 unwrap 滥用
    let result = Some(42);
    let num = result.unwrap();
    
    // 不必要的 Vec 分配
    let numbers = vec![1, 2, 3, 4, 5];
    for i in 0..numbers.len() {
        println!("{}", numbers[i]);
    }
    
    // 应该用迭代器的地方用了循环
    let mut sum = 0;
    for num in numbers {
        sum += num;
    }
    
    // 复杂的 match，应该用 if let
    match Some(42) {
        Some(x) => println!("Got {}", x),
        None => {},
    }
}

// 单字母变量滥用
fn single_letters() {
    let a = 1;
    let b = 2; 
    let c = 3;
    let d = 4;
    let e = 5;
    
    for i in 0..10 {
        for j in 0..10 {
            for k in 0..10 {
                // 太多嵌套循环
                println!("{} {} {}", i, j, k);
            }
        }
    }
}

// 被注释掉的代码块
/*
fn old_function() {
    let old_data = "this is old";
    println!("This function is no longer used");
    // More old code here
    let unused = 42;
}
*/