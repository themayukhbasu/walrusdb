fn main() {
    println!("============= [u8; 4] to u32 ==============");
    let buf = [0x2a, 0x2a, 0x2a, 0x2a];
    let n = u32::from_le_bytes(buf);

    println!("{}", n);
    println!("{}", buf[0]);
    println!("{}", buf[1]);

    println!("============= Big Endian ==============");
    let buf = [0x00, 0x00, 0x00, 0x2a]; // [0x2a, 0x00, 0x00, 0x00]
    let n: u32 = u32::from_be_bytes(buf);
    println!("{}", n);

    println!("============= u32 to [u8; 4] ==============");
    let n: u32 = 42;
    let buf: [u8; 4] = n.to_le_bytes();
    println!("{:?}", buf);

    println!("============= size of var ==============");

    let slot0 = [b'A'; 32];
    println!("size of b'A' :{}", std::mem::size_of_val(&slot0));
    let byte = b'A';
    println!("b'A' = {}", byte);
    println!("get back b'A' from 65 = {}", byte as char);
    // size of type
    println!("{}", std::mem::size_of::<u32>());

    let flag = false;
    println!("{}", std::mem::size_of_val(&flag));
}
