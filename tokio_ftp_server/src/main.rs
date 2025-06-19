use std::collections::HashMap;
use std::sync::Arc;
use std::pin::Pin;
use std::future::Future;
use std::process::Command;
use tokio::fs::File;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::net::{TcpListener,TcpStream};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
/*
struct Credentials{
    username: String,
    password: String,
}
*/
enum Handler {
    Ls(fn(String, i8, Arc<RwLock<HashMap<i8, (String, String)>>>) -> Pin<Box<dyn Future<Output = String> + Send>>),
    Help(fn() -> String),
    Cd(fn(String, i8, String, Arc<RwLock<HashMap::<i8, (String, String)>>>)-> Pin<Box<dyn Future<Output = String> + Send>>), 
    Pwd(fn(i8, Arc<RwLock<HashMap::<i8, (String, String)>>>)-> Pin<Box<dyn Future<Output = String> + Send>>), 
    File(fn(String, Arc<Mutex<TcpStream>>, String, i8, Arc<RwLock<HashMap<i8, (String, String)>>>) -> Pin<Box<dyn Future<Output = String> + Send>>),
}
const ROOT_DIRECTORY: &str = "ftp_root";

async fn handle_ls(path: String, id: i8, working_directory_map: Arc<RwLock<HashMap::<i8, (String, String)>>>) -> String {
    // maybe i will add the option later to be able to do ls directory
    // path is always "" for now but the functionality is almost ready
    let map_tuple = {
        let map = working_directory_map.read().await;
        map.get(&id).cloned()
    };
    if let Some((_peer_address, current_directory)) = map_tuple {
        let output = {
            if path != "" {
                Command::new("ls")
                    .arg("-F")
                    .arg(current_directory)
                    .arg(&path)
                    .output()
                    .unwrap()
            } else {
                Command::new("ls")
                    .arg("-F")
                    .arg(current_directory)
                    .output()
                    .unwrap()
            }
        };

        if output.status.success(){
            let stdout = String::from_utf8_lossy(&output.stdout).into();
            return stdout;
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).into();
            return stderr;
        }
    } else {
        println!("this id {} doesn't exist", id);
        return String::from("Error in ls")
    }

}
//cd .. takes the Hashmap and removes the last / and does ls. Only if it is allowed. (you can go
//back up to a point not further.
//cd directory -> if it is not a directory return message not a directory, if it doesn't exist
//return doesn't exist, if it exists add in the hashmap /directory
//for simplicity reasons

async fn handle_cd(command: String, id: i8, peer: String, working_directory_map: Arc<RwLock<HashMap::<i8, (String, String)>>>) -> String {

    let parts = command.split(" ");
    let collection = parts.collect::<Vec<&str>>();
    
    let my_str = {
        let map = working_directory_map.read().await;
        map.get(&id).cloned()
    };

    if let Some((_peer_address, old_directory)) = my_str {
        let new_directory:String;
        if collection[1].to_string() == String::from(".."){
            if let Some(pos) = old_directory.rfind('/') {
                new_directory = old_directory[..pos].to_string();
            }

            else{
                return String::from("Can't go more up")
            }
        } else {
            new_directory = format!("{}/{}", old_directory, collection[1].to_string());
        }
        //println!("old directory {}", old_directory);
        //println!("new directory {}", new_directory);
        let mut map = working_directory_map.write().await;
        map.insert(id, (peer, new_directory.clone()));
        //temp
        return new_directory
    } else {
        println!("this id {} doesn't exist", id);
        return String::from("Error in cd")
    }
}


async fn handle_pwd(id: i8, working_directory_map: Arc<RwLock<HashMap::<i8, (String, String)>>>) -> String{
    //not really executing pwd command we just want the user to know his relevant directory path

    let my_str = {
        let map = working_directory_map.read().await;
        map.get(&id).cloned()
    };
    if let Some((_peer_address, current_directory)) = my_str {
        return current_directory.clone()
    } else {
        println!("this id {} doesn't exist", id);
        return String::from("Error in pwd")
    }
}
    

fn handle_help() -> String {
    return String::from("###available commands###\n\nls - show files in current directory\ncd .. - move up one directory\ncd directory - move down one specified directory\npwd - show current directory\n\n");
}

async fn handle_file(command: String, socket: Arc<Mutex<TcpStream>>, path: String, id: i8, working_directory_map: Arc<RwLock<HashMap::<i8, (String, String)>>>) -> String {
    let parts = command.split(" ");
    let collection = parts.collect::<Vec<&str>>();
    let available_files:String = handle_ls(path, id, Arc::clone(&working_directory_map)).await;
    
    let map_tuple = {
        let map = working_directory_map.read().await;
        map.get(&id).cloned()
    };
    if let Some((_peer_address, current_directory)) = map_tuple {
        let file_path = format!("{}/{}", current_directory, collection[1].to_string() );
        if available_files.contains(collection[1]){
            match send_file(socket, file_path).await {
                Ok(_) => String::from("File sent ok\n"),
                Err(_) => String::from("Error sending file\n"),
            }
        } else {
            let mut socket = socket.lock().await;
            match socket.write_all(String::from("File doesn't exist\n").as_bytes()).await {
                Ok(_) => String::from("Message sent ok\n"),
                Err(_) => String::from("Message wasn't sent\n"),
            }
        }
    } else {
        println!("this id {} doesn't exist", id);
        return String::from("Error in ls")
    }
    // this is not correct still because file1 might include the file string but its different to
    // ask for file and file1. i should split the reply from spaces and new lines and then check.
    // also i might add a check for directory or file(this is a maybe)
}


async fn match_command(command: &str, command_map: &HashMap<String, Handler>, socket: Arc<Mutex<TcpStream>>, path: String, id: i8, peer: String, working_directory_map: Arc<RwLock<HashMap::<i8, (String, String)>>>) -> Option<String> {
    let parts: Vec<&str> = command.split(' ').collect();
    //first word is the command second word is files etc...
    match command_map.get(parts[0]){
        Some(Handler::Ls(handler)) => {
            Some(handler(path, id, working_directory_map).await)
        }
        Some(Handler::Help(handler)) => {
            Some(handler())
        }
        Some(Handler::Cd(handler)) => {
            Some(handler(command.to_string(), id, peer, working_directory_map).await)
        }
        Some(Handler::Pwd(handler)) => {
            Some(handler(id, working_directory_map).await)
        }
        Some(Handler::File(handler)) => {
            Some(handler(command.to_string(), socket, path, id, working_directory_map).await)
        }
        None => None
    }
}



async fn send_file(socket: Arc<Mutex<TcpStream>>, file_path: String) -> io::Result<()> {
    let mut file = File::open(file_path).await?;
    let mut buffer = [0; 8192];
    loop{
        let n = file.read(&mut buffer).await?;
        if n == 0{
            break;
        }
        let mut socket = socket.lock().await;
        socket.write_all(&buffer[..n]).await?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /*
    let first_person = Credentials {
        username: String::from("panos"),
        password: String::from("panoswd"),
    };
    */
    let mut command_map = HashMap::new();
    {
        command_map.insert(String::from("ls"), Handler::Ls(|path, id, working_directory_map| {
            Box::pin(handle_ls(path, id, working_directory_map))
        }));
        command_map.insert(String::from("help"), Handler::Help(handle_help));
        command_map.insert(String::from("cd"), Handler::Cd(|command, id, peer, working_directory_map| {
            Box::pin(handle_cd(command, id, peer, working_directory_map))
        }));

        command_map.insert(String::from("pwd"), Handler::Pwd(|id, working_directory_map| {
            Box::pin(handle_pwd(id, working_directory_map))
        }));
            
        command_map.insert(String::from("file"), Handler::File(|command, socket, path, id, working_directory_map|{
            Box::pin(handle_file(command, socket, path, id, working_directory_map))
        }));
    }
    let command_map = Arc::new(command_map);

    let directory_map = Arc::new(RwLock::new(HashMap::<i8, (String, String)>::new()));
    
    let id = 1; 

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Running a tcp server at port 8080");
    loop {
        let (socket, _) = listener.accept().await?;
        let directory_map = Arc::clone(&directory_map);
        {
            let mut map = directory_map.write().await;
            let peer = socket.peer_addr()?;
            let peer = peer.to_string();
            map.insert(id, (peer, ROOT_DIRECTORY.to_string()));
        }

        {
            let map = directory_map.read().await;
            let my_str = map.get(&id);
            if let Some((peer_address, _directory)) = my_str {
                println!("{}", peer_address);
            } else {
                
            }
        }
        let command_map_clone = Arc::clone(&command_map);
        // i dont think thie socket should really have Mutex
        let socket = Arc::new(Mutex::new(socket));

        tokio::spawn(async move {
            let mut buf = [0; 1024];
            let socket_clone = Arc::clone(&socket);
            // In a loop, read data from the socket and write the data back.
            let directory_map = Arc::clone(&directory_map);
            loop {
                let n = {
                    let mut socket = socket_clone.lock().await;
                    match socket.read(&mut buf).await {
                        // socket closed
                        Ok(0) => return,
                        Ok(n) => n,
                        Err(e) => {
                            eprintln!("failed to read from socket; err = {:?}", e);
                            return;
                        }
                    }
                };
                let peer = {
                    let socket = socket_clone.lock().await;
                    match socket.peer_addr(){
                        Ok(peer) => peer.to_string(),
                        Err(e) => {
                            eprintln!("failed to get peer address; err = {:?}", e);
                            return;
                        }
                    }
                };
                let peer = peer.to_string();
                
                let request = String::from_utf8_lossy(&buf[0..n]);
                let request = request.trim();
                //client sent file so we send him the poem
                println!("user {}  requested {}",&peer, &request);
                //println!("ray bytes {:?}", &request);
                //let path = ROOT_DIRECTORY.to_string();
                let path = String::from("");
                //temporary
                let id = 1;
                match match_command(&request, &command_map_clone, Arc::clone(&socket_clone), path, id, peer, Arc::clone(&directory_map)).await{
                    Some(response) => {
                        let parts: Vec<&str> = request.split(' ').collect();
                        //temp solution. i should put it on the None too
                        {
                            let socket_locked = socket_clone.lock().await; 
                            let _peer = match socket_locked.peer_addr(){
                                Ok(peer) => peer.to_string(),
                                Err(e) => {
                                    eprintln!("failed to get peer address; err = {:?}", e);
                                    return;
                                }
                            };
                        }

                        //println!("the response is{}\n", &response);
                        if parts[0] != String::from("file"){
                            if let Err(e) = socket_clone.lock().await.write_all(response.as_bytes()).await {
                                eprintln!("failed to send file to socket; err = {:?}", e);
                                return;
                            }
                        }
                    }
                    None => {
                        println!("Error: unknown command");
                        if let Err(e) = socket_clone.lock().await.write_all(String::from("Error unknown command\n").as_bytes()).await {
                            eprintln!("failed to write to socket; err = {:?}", e);
                            return;
                        }
                        
                    }
                }
            }
        });
    }
}
