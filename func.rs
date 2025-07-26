fn main() {
    println!("Hello, world!");
    let haha = "haha";
    {
        {
            {
                a();
                b();
                {
                    a();
                }
                {
                    b();
                }
                {
                    a();
                }
                {
                    b();
                }
            }
        }
    }
}



fn a(){
    1+2;
}

fn b(){
    1+2;
}

