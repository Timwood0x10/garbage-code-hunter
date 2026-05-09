// This is a deliberately "garbage" Rust file for testing plugin functionality

fn main() {
    // Meaningless variable names
    let data = "hello world";
    let temp = 42;
    let foo = vec![1, 2, 3];
    let bar = String::new();
    
    // Hungarian notation (not recommended)
    let strName = "John";
    let intAge = 25;
    let bIsValid = true;
    
    // Excessive abbreviations
    let mgr = "manager";
    let ctrl = "controller";
    let usr = "user";
    
    // Printf debugging
    println!("Debug: data = {}", data);
    println!("Debug: temp = {}", temp);
    
    // Unwrap abuse
    let result = Some("test");
    let value = result.unwrap();
    
    // Unnecessary clones
    let original = String::from("hello");
    let copy = original.clone();
    let another_copy = copy.clone();
    
    // Deep nesting
    if true {
        if true {
            if true {
                if true {
                    println!("Too deep!");
                }
            }
        }
    }
    
    // Magic numbers
    let magic = 42;
    let another_magic = 3.14159;
    
    // Too many TODO comments
    // TODO: fix this
    // TODO: optimize this
    // TODO: refactor this
    
    // Long function (this function does too much)
    process_everything(data, temp, foo, bar, strName, intAge, bIsValid, mgr, ctrl, usr, value, copy, another_copy, magic, another_magic);
}

// God function - does too much
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
    // This function has too many parameters
    println!("Processing everything...");
    
    // More unwrap abuse
    let result = Some(42);
    let num = result.unwrap();
    
    // Unnecessary Vec allocation
    let numbers = vec![1, 2, 3, 4, 5];
    for i in 0..numbers.len() {
        println!("{}", numbers[i]);
    }
    
    // Should use iterator instead of loop
    let mut sum = 0;
    for num in numbers {
        sum += num;
    }
    
    // Complex match, should use if let
    match Some(42) {
        Some(x) => println!("Got {}", x),
        None => {},
    }
}

// Single letter variable abuse
fn single_letters() {
    let a = 1;
    let b = 2; 
    let c = 3;
    let d = 4;
    let e = 5;
    
    for i in 0..10 {
        for j in 0..10 {
            for k in 0..10 {
                // Too many nested loops
                println!("{} {} {}", i, j, k);
            }
        }
    }
}

// Commented out code blocks
/*
fn old_function() {
    let old_data = "this is old";
    println!("This function is no longer used");
    // More old code here
    let unused = 42;
}
*/