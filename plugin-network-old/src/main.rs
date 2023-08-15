mod nmcli;

fn main() {
    let networks = nmcli::scan().unwrap();
    for network in networks {
        println!("{network:?}");
    }
}
