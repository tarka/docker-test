use nix::unistd::Uid;

fn main() {
    let uid = Uid::current();
    println!("Hello, docker! My UID is [{}]", uid);
}
