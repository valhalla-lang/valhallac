mod syntax;

fn main() {
    println!("\nTill Valhalla!\n");
    
    syntax::parse_file("./test.vh");
}
