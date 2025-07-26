use std::collections::HashMap;

// 这是一个故意写得很糟糕的代码示例，用来测试代码质量检测工具

fn main() {
    let x = 1;
    let y = 2;
    let z = 3;
    let a = 4;
    let b = 5;
    let c = 6;
    let d = 7;
    let e = 8;
    let f = 9;
    let g = 10;
    
    // 深度嵌套的噩梦
    if x > 0 {
        if y > 0 {
            if z > 0 {
                if a > 0 {
                    if b > 0 {
                        if c > 0 {
                            if d > 0 {
                                if e > 0 {
                                    println!("太深了！");
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // 重复的代码块
    let mut data1 = Vec::new();
    data1.push(1);
    data1.push(2);
    data1.push(3);
    let sum1 = data1.iter().sum::<i32>();
    println!("Sum1: {}", sum1);
    
    let mut data2 = Vec::new();
    data2.push(4);
    data2.push(5);
    data2.push(6);
    let sum2 = data2.iter().sum::<i32>();
    println!("Sum2: {}", sum2);
    
    let mut data3 = Vec::new();
    data3.push(7);
    data3.push(8);
    data3.push(9);
    let sum3 = data3.iter().sum::<i32>();
    println!("Sum3: {}", sum3);
    
    // 调用糟糕的函数
    let result1 = bad_function_1(x, y);
    let result2 = bad_function_2(a, b);
    let result3 = bad_function_1(c, d); // 重复调用
    let result4 = bad_function_2(e, f); // 重复调用
    
    // 更多无意义的变量
    let temp = result1;
    let tmp = result2;
    let val = result3;
    let res = result4;
    
    // 复杂的处理逻辑
    process_data(temp, tmp, val, res);
    
    // 未使用的变量
    let unused1 = 42;
    let unused2 = "hello";
    let unused3 = vec![1, 2, 3];
}

// 糟糕的函数1 - 名字无意义，逻辑复杂
fn bad_function_1(x: i32, y: i32) -> i32 {
    let mut result = 0;
    
    // 无意义的计算
    let temp1 = x + y;
    let temp2 = x - y;
    let temp3 = x * y;
    let temp4 = x / (y + 1);
    
    // 深度嵌套的条件判断
    if temp1 > 0 {
        if temp2 > 0 {
            if temp3 > 0 {
                if temp4 > 0 {
                    result = temp1 + temp2;
                } else {
                    result = temp1 - temp2;
                }
            } else {
                result = temp1 * temp2;
            }
        } else {
            result = temp1 / (temp2 + 1);
        }
    } else {
        result = 0;
    }
    
    // 重复的代码块
    let mut data = Vec::new();
    data.push(result);
    data.push(result + 1);
    data.push(result + 2);
    let sum = data.iter().sum::<i32>();
    
    sum
}

// 糟糕的函数2 - 几乎和函数1一样
fn bad_function_2(a: i32, b: i32) -> i32 {
    let mut result = 0;
    
    // 几乎相同的计算
    let temp1 = a + b;
    let temp2 = a - b;
    let temp3 = a * b;
    let temp4 = a / (b + 1);
    
    // 相同的嵌套结构
    if temp1 > 0 {
        if temp2 > 0 {
            if temp3 > 0 {
                if temp4 > 0 {
                    result = temp1 + temp2;
                } else {
                    result = temp1 - temp2;
                }
            } else {
                result = temp1 * temp2;
            }
        } else {
            result = temp1 / (temp2 + 1);
        }
    } else {
        result = 0;
    }
    
    // 重复的代码块
    let mut data = Vec::new();
    data.push(result);
    data.push(result + 1);
    data.push(result + 2);
    let sum = data.iter().sum::<i32>();
    
    sum
}

// 超级复杂的处理函数
fn process_data(w: i32, x: i32, y: i32, z: i32) {
    // 创建一个HashMap但用糟糕的方式
    let mut map = HashMap::new();
    
    // 深度嵌套的循环
    for i in 0..3 {
        for j in 0..3 {
            for k in 0..3 {
                for l in 0..3 {
                    for m in 0..3 {
                        for n in 0..3 {
                            let key = format!("{}-{}-{}-{}-{}-{}", i, j, k, l, m, n);
                            let value = i + j + k + l + m + n;
                            map.insert(key, value);
                        }
                    }
                }
            }
        }
    }
    
    // 重复的处理逻辑
    let mut results = Vec::new();
    for (key, value) in &map {
        if value % 2 == 0 {
            results.push(format!("Even: {} = {}", key, value));
        }
    }
    
    let mut results2 = Vec::new();
    for (key, value) in &map {
        if value % 3 == 0 {
            results2.push(format!("Divisible by 3: {} = {}", key, value));
        }
    }
    
    let mut results3 = Vec::new();
    for (key, value) in &map {
        if value % 5 == 0 {
            results3.push(format!("Divisible by 5: {} = {}", key, value));
        }
    }
    
    // 打印结果（但方式很糟糕）
    println!("Results count: {}", results.len());
    println!("Results2 count: {}", results2.len());
    println!("Results3 count: {}", results3.len());
}

// 更多糟糕的函数
fn a() -> i32 { 1 }
fn b() -> i32 { 2 }
fn c() -> i32 { 3 }
fn d() -> i32 { 4 }
fn e() -> i32 { 5 }

// 单字母结构体
struct A {
    x: i32,
    y: i32,
}

struct B {
    a: i32,
    b: i32,
}

// 糟糕的实现
impl A {
    fn new(x: i32, y: i32) -> Self {
        // 无意义的嵌套
        if x > 0 {
            if y > 0 {
                if x + y > 0 {
                    A { x, y }
                } else {
                    A { x: 0, y: 0 }
                }
            } else {
                A { x: 0, y: 0 }
            }
        } else {
            A { x: 0, y: 0 }
        }
    }
    
    fn calculate(&self) -> i32 {
        // 重复的计算逻辑
        let temp1 = self.x + self.y;
        let temp2 = self.x - self.y;
        let temp3 = self.x * self.y;
        
        if temp1 > temp2 {
            if temp2 > temp3 {
                temp1
            } else {
                temp2
            }
        } else {
            temp3
        }
    }
}