use std::borrow::Cow;
use std::io::Cursor;
use std::ops::Deref;
use std::sync::Arc;

use bytes::{Buf, BytesMut};
use dashmap::DashMap;
use dxr::{
    DxrError, Fault, FaultResponse, MethodCall, MethodResponse, TryFromValue, TryToParams, Value,
};

use tachyonix::Sender;
use thiserror::Error;
use tm_server_types::event::Event;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, BufWriter, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
//use tokio::sync::mpsc::Sender;
use tokio::sync::{broadcast, oneshot};

#[derive(Debug)]
struct GbxPacket {
    handler: u32,
    body: String,
}

impl GbxPacket {
    fn parse(buf: &mut Cursor<&[u8]>) -> Result<GbxPacket, ClientError> {
        if buf.remaining() < 8 {
            return Err(ClientError::Incomplete);
        }
        let size = buf.get_u32_le() as usize;
        let handler = buf.get_u32_le();
        if buf.remaining() < size {
            return Err(ClientError::Incomplete);
        }

        let body = String::from_utf8_lossy(&buf.chunk()[..size]).into_owned();

        // Advance the buffer to body size. (Header Methods calls of u32 do this automatically)
        buf.advance(size);

        Ok(GbxPacket { handler, body })
    }

    fn is_method_response(&self) -> bool {
        self.handler > 0x80000000u32
    }
}

#[derive(Debug)]
enum GbxMethodCall {
    MethodCall {
        message: String,
        responder: oneshot::Sender<MethodResponse>,
    },
    /* Callback {
        message: String,
    }, */
}

/// Associates all events to a channel.
#[derive(Clone)]
struct RegisiteredCallbacks(
    #[allow(clippy::type_complexity)]
    Arc<
        DashMap<
            String,
            (
                broadcast::Receiver<Arc<Event>>,
                broadcast::Sender<Arc<Event>>,
            ),
        >,
    >,
);
//Arc<DashMap<String, Vec<Box<dyn Fn(&str) + Send + Sync>>>>

impl RegisiteredCallbacks {
    fn new() -> Self {
        RegisiteredCallbacks(Arc::new(DashMap::new()))
    }

    fn get(&self, key: &str) -> broadcast::Receiver<Arc<Event>> {
        if let Some(entry) = self.0.get(key) {
            entry.1.subscribe()
        } else {
            let new_channel = broadcast::channel::<Arc<Event>>(8);
            let ret = new_channel.0.subscribe();

            self.0
                .insert(key.to_owned(), (new_channel.1, new_channel.0));

            ret
        }
    }

    fn send(&self, key: &str, event: Arc<Event>) {
        if let Some(entry) = self.0.get(key) {
            _ = entry.1.send(event);
        }
    }
}

/// Interact with a server through xml-rpc.
/// Implemented with separate read and write threads.
/// Events will also execute as separate tokio tasks.
/// Interaction should be fully typed and can be achieved by importing trait from the types module.
pub struct TrackmaniaServer {
    /// Handler to reach the write thread and pass a message to the server.
    message_sender: Sender<GbxMethodCall>,

    /// Associates a handler value with a oneshot channel to correctly receive the response.
    response_mapping: Arc<DashMap<u32, oneshot::Sender<MethodResponse>>>,

    /// Subscriptions to the global_callback receive every event raised on the server.
    global_callback: broadcast::Receiver<Arc<Event>>,

    /// Allows to subscribe to specific callbacks.
    registered_callbacks: RegisiteredCallbacks,
}

impl TrackmaniaServer {
    pub async fn new(url: impl Into<String>) -> Self {
        let stream = BufWriter::new(TcpStream::connect(url.into()).await.unwrap());

        let (mut reader, writer) = io::split(stream);

        // Expect the "GbxRemote 2" handshake message.
        let mut buf = vec![0; 15];
        let _ = reader.read(&mut buf).await;

        let size = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let call = String::from_utf8(buf[4..((size + 4) as usize)].to_vec()).unwrap();

        println!("Connected to: {call}");

        let (sender, rx) = tachyonix::channel::<GbxMethodCall>(32);

        // With many players trackmania can dump a shitload of events so the capacity is very large to prevent overflows.
        let (global_callback_sender, global_callback) = broadcast::channel(222);

        let client = Self {
            global_callback,
            message_sender: sender,
            response_mapping: Arc::new(DashMap::new()),

            registered_callbacks: RegisiteredCallbacks::new(),
        };

        let writer_response = client.response_mapping.clone();
        Self::setup_write_loop(rx, writer, writer_response);

        let reader_response = client.response_mapping.clone();
        let registered_callbacks = client.registered_callbacks.clone();
        Self::setup_read_loop(
            reader_response,
            registered_callbacks,
            global_callback_sender,
            reader,
        );
        client
    }

    /// Internal helper to setup the thread which is responsible for sending messages to the server.
    fn setup_write_loop(
        mut write_request: tachyonix::Receiver<GbxMethodCall>,
        mut writer: WriteHalf<BufWriter<TcpStream>>,
        writer_response: Arc<DashMap<u32, oneshot::Sender<MethodResponse>>>,
    ) {
        tokio::spawn(async move {
            let mut handler = 0x80000000u32;

            // Start receiving messages and only stop when all senders get out of scope.
            while let Ok(cmd) = write_request.recv().await {
                match cmd {
                    GbxMethodCall::MethodCall { message, responder } => {
                        // Increment the handler before each method call
                        handler += 1;

                        // Since GbxRemote packets expect little endian write them out.
                        writer.write_u32_le(message.len() as u32).await.unwrap();
                        writer.write_u32_le(handler).await.unwrap();

                        // The body of the packet.
                        writer.write_all(message.as_bytes()).await.unwrap();

                        let _ = writer.flush().await;

                        writer_response.insert(handler, responder);
                    }
                }
            }
        });
    }

    /// Internal helper to setup the thread which is responsible for receiving messages (method call responses and events) from the server.
    fn setup_read_loop(
        reader_response: Arc<DashMap<u32, oneshot::Sender<MethodResponse>>>,
        registered_callbacks: RegisiteredCallbacks,
        global_callback_sender: broadcast::Sender<Arc<Event>>,
        mut reader: ReadHalf<BufWriter<TcpStream>>,
    ) {
        tokio::spawn(async move {
            let mut buffer: BytesMut = BytesMut::with_capacity(1024);

            /// Ensures only complete messages get passed as a valid packet and keeps the buffer in check.
            fn parse_packet(buffer: &mut BytesMut) -> Option<GbxPacket> {
                let mut cursor = Cursor::new(&buffer[..]);

                if let Ok(packet) = GbxPacket::parse(&mut cursor) {
                    buffer.advance(cursor.position() as usize);

                    // Return the frame to the caller.
                    Some(packet)
                } else {
                    None
                }
            }

            // The main reading loop continously receiveing messages from the server.
            loop {
                // When we succeed parsing a packet...
                while let Some(packet) = parse_packet(&mut buffer) {
                    //check if this is a method response we are waiting for.
                    if packet.is_method_response() {
                        let (_, response) = reader_response.remove(&packet.handler).unwrap();
                        _ = response.send(body_to_response(&packet.body).unwrap());
                    } else {
                        // if its not a method response it must be an event.
                        let callback = dxr::deserialize_xml::<MethodCall>(&packet.body).unwrap();
                        // Event from the ModeScript extension which is the newer counterpart to the legacy events.
                        if callback.name() == "ManiaPlanet.ModeScriptCallbackArray" {
                            let params = callback.params();
                            let modescript_callback_name =
                                String::try_from_value(&params[0]).unwrap();

                            let value = Vec::<Value>::try_from_value(&params[1]).unwrap();
                            let modescript_callback_body =
                                String::try_from_value(&value[0]).unwrap();

                            println!(
                                "Name: {modescript_callback_name}, JSON: {modescript_callback_body:?}"
                            );

                            // Parse the event to make it fully typed.
                            let event = Event::new(
                                modescript_callback_name.clone(),
                                modescript_callback_body,
                            );

                            // Send the parsed event to all subscribed event handlers.
                            let event = Arc::new(event);
                            if let Err(error) = global_callback_sender.send(event.clone()) {
                                println!("Global Events Listener failed: {:?}", error);
                            }
                            registered_callbacks.send(&modescript_callback_name, event);
                        } else {
                            println!("Old callback: {:?}", callback);
                        }
                    }
                }

                // If we failed parsing a full packet we must have a partial packet.
                // That means wewait for the another activity on the tcp socket.
                // If the tcp socket returns 0 it has closed and we disconnect.
                // Otherwise we loop around and try to parse a full packet again above.
                if 0 == reader.read_buf(&mut buffer).await.unwrap() {
                    // The remote closed the connection. For this to be a clean
                    // shutdown, there should be no data in the read buffer. If
                    // there is, this means that the peer closed the socket while
                    // sending a frame.
                    if buffer.is_empty() {
                        println!("The Trackmania server ended the connection.");
                        break;
                    } else {
                        panic!("connection reset by peer");
                    }
                }
            }
        });
    }

    /// Returns a handle that receives every message of the selected event.
    pub fn subscribe<'a>(&'a self, event: impl Into<&'a str>) -> broadcast::Receiver<Arc<Event>> {
        self.registered_callbacks.get(event.into())
    }

    /// Executes the specified function whenever the specified event is triggered.
    pub fn on<'b, T, F>(&self, event: impl Into<&'b str>, execute: F)
    where
        for<'a> &'a T: From<&'a Event>,
        F: Fn(&T),
        F: Send + Sync + 'static,
    {
        let mut receiver = self.registered_callbacks.get(event.into());

        tokio::spawn(async move {
            while let Ok(event) = receiver.recv().await {
                execute(Into::<&T>::into(event.deref()));
            }
        });
    }

    /// The specified handler function gets called on every event that the server sends to us.
    pub fn event(&self, handle: impl Fn(&Event) + Send + Sync + 'static) {
        let mut receiver = self.global_callback.resubscribe();

        tokio::spawn(async move {
            while let Ok(event) = receiver.recv().await {
                handle(&event);
            }
        });
    }

    /// Allows to call a method on the server.
    /// Needs to be awaited in order to be executed and receive the response.
    pub async fn call<P: TryToParams, R: TryFromValue>(
        &self,
        method: &str,
        args: P,
    ) -> Result<R, ClientError> {
        let params = args.try_to_params()?;
        let result = self.call_inner(Cow::Borrowed(method), params).await?;

        // extract return value
        Ok(R::try_from_value(&result)?)
    }

    // Internal helper to get correct method call response.
    async fn call_inner(
        &self,
        method: Cow<'_, str>,
        params: Vec<Value>,
    ) -> Result<Value, ClientError> {
        // serialize XML-RPC method call
        let request = MethodCall::new(method, params);
        let xml = dxr::serialize_xml(&request)
            .map_err(|error| DxrError::invalid_data(error.to_string()))?;

        // concatenate the body with the xml header.
        let body = [r#"<?xml version="1.0"?>"#, xml.as_str()].join("");

        // Obtain the way to send a message to the server.
        let message_sender = self.message_sender.clone();

        let response = tokio::spawn(async move {
            // Responsible to notify us when the method response is there.
            let (send_me_response, waiting) = oneshot::channel();
            message_sender
                .send(GbxMethodCall::MethodCall {
                    message: body,
                    responder: send_me_response,
                })
                .await
                .unwrap();

            // Wait till we receive the response for the method call.
            waiting.await.unwrap()
        })
        .await
        .unwrap();

        Ok(response.inner())
    }
}

fn body_to_response(contents: &str) -> Result<MethodResponse, ClientError> {
    // need to check for FaultResponse first:
    // - a missing <params> tag is ambiguous (can be either an empty response, or a fault response)
    // - a present <fault> tag is unambiguous
    let error2 = match dxr::deserialize_xml(contents) {
        Ok(fault) => {
            let response: FaultResponse = fault;
            return match Fault::try_from(response) {
                // server fault: return Fault
                Ok(fault) => Err(fault.into()),
                // malformed server fault: return DxrError
                Err(error) => Err(error.into()),
            };
        }
        Err(error) => error.to_string(),
    };

    let error1 = match dxr::deserialize_xml(contents) {
        Ok(response) => return Ok(response),
        Err(error) => error.to_string(),
    };

    // log errors if the contents could not be deserialized as either response or fault
    println!("Failed to deserialize response as either value or fault.");
    println!("Response failed with: {error1}; Fault failed with: {error2}");

    // malformed response: return DxrError::InvalidData
    Err(DxrError::invalid_data(contents.to_owned()).into())
}

#[derive(Debug, Error)]
pub enum ClientError {
    /// Error variant for XML-RPC server faults.
    #[error("{}", fault)]
    Fault {
        /// Fault returned by the server.
        #[from]
        fault: Fault,
    },
    /// Error variant for XML-RPC errors.
    #[error("{}", error)]
    RPC {
        /// XML-RPC parsing error.
        #[from]
        error: DxrError,
    },
    #[error("request incomplete")]
    Incomplete,
}

#[allow(unused)]
impl ClientError {
    fn fault(fault: Fault) -> Self {
        ClientError::Fault { fault }
    }

    fn rpc(error: DxrError) -> Self {
        ClientError::RPC { error }
    }
}
