use tokio::{
    io::{AsyncWriteExt, BufReader, AsyncBufReadExt,},
    net::TcpListener,
    sync::broadcast,
};

#[tokio::main]
async fn main() 
{
  // IMPORTANT: .unwrap() is used here for learning. Use of .unwrap() throws a panic.
  // Error handling should use ? operator (ex: .await? ). 
  // Use of ? requires a Result<()> to be returned
  // useful answer here: https://stackoverflow.com/questions/42917566/what-is-this-question-mark-operator-about

  // setup a tokio socket listener
  let listener = TcpListener::bind("localhost:8080").await.unwrap();

  // setup a broadcast channel to allow producers and consumers to share between each other
  let (tx, _rx) = broadcast::channel::<String>(10);

  loop 
  {
    // assign the listener to a socket and address. _ <- underscore ignores warnings, errors, for now (like in Go)
    let (mut socket, _addr) = listener.accept().await.unwrap();

    let tx = tx.clone();
    let mut rx = tx.subscribe();

    // tokio::spawn creates a seperate concurent task for each connection
    // read tokio docs if you wish to know more 
    tokio::spawn(async move 
    {
      // here we split the io read/write portions of the stream
      // otherwise, reader:BufReader would have ownership of the entire socket
      let (reader, mut writer) = socket.split();

      // assign reader:ReadHalf to a mutable BufReader
      let mut reader = BufReader::new(reader);
      // create a String for storing the stream of bytes
      // tokio BufReader can read text line by line from a stream
      let mut line = String::new();

      loop 
      {
          tokio::select! 
          {
            result = reader.read_line(&mut line) => 
            {
              if result.unwrap() == 0 
              { // if we have 0 bytes incoming we have reached the end of the file 
                break;
              }

              tx.send(line.clone()).unwrap();
              // line.clear() clears variable 'line'. read_line appends the next stream line
              // to whatever has comes through. We must clear the string outselves.
              // There is a practical reason to keep this (ex: reading in a file line by line)
              line.clear();
            }
            result = rx.recv() => 
            {
              let msg = result.unwrap();
              writer.write_all(msg.as_bytes()).await.unwrap();
            }
          }
      }
    });
  }
}