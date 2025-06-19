use std::io;
use std::io::{Read,Write};
use std::net::TcpStream;


fn main() -> io::Result<()> {
    println!("Welcome to my newbie ftp server i made just for you!\n\n");

    let mut stream = TcpStream::connect("127.0.0.1:8080")?;
    println!("connected to server");
    let mut socket_buff = [0; 8192];
    loop {

        let mut buffer = String::new();
        match io::stdin().read_line(&mut buffer){
            Ok(_n) => {},
            Err(_) => println!("failed to read input")
        }
        stream.write_all(buffer.as_bytes())?;
        let command_parts: Vec<String> = buffer.split_whitespace().map(|s| s.to_string()).collect();
        if command_parts[0] == String::from("file"){
            loop {
                let n = stream.read(&mut socket_buff)?;
                if n == 0 {
                    println!("Server closed the connection");
                    break;
                }
                println!("n is {}", n);
                let _received_data = String::from_utf8_lossy(& socket_buff[..n]);
                //println!("\n\n\n\n\n{}", received_data);
                
                /*
                //this didn't work because the data comes as a stream aka chunks or seperate
                //messages are combined
                if received_data == String::from("File sent ok\n"){
                    println!("this is correct");
                    break;
                }
                */
                if n < 8192{
                    println!("file received\n");
                    /*
                    let n = stream.read(&mut socket_buff)?;
                    println!("received ok");
                    let received_data = String::from_utf8_lossy(& socket_buff[..n]);
                    println!("\n\n\n\n\n{} {}", received_data, n);
                    */
                    break;
                }
            }
        } else {

            let n = stream.read(&mut socket_buff)?;
            if n == 0 {
                println!("Server closed the connection");
                break;
            }
            let received_data = String::from_utf8_lossy(& socket_buff[..n]);
            println!("{}", received_data);
        }
        
        
    }
    Ok(())
}
