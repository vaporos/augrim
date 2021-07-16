// Copyright 2021 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::hash::Hash;
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::error::InternalError;
use crate::links::{FairLossReceiver, FairLossSender};
use crate::message::Message;
use crate::process::Process;

#[derive(Debug)]
pub struct IntraProcessNetworkError;

enum ControlMessage<P, M> {
    Message(P, M),
    Shutdown,
}

pub struct IntraProcessNetwork<P, M, R>
where
    P: Process,
    M: Message,
    R: IntraProcessNetworkReceiver<M>,
{
    join_handle: thread::JoinHandle<()>,
    sender: Sender<ControlMessage<P, M>>,
    process_to_receiver: Arc<Mutex<HashMap<P, R>>>,
}

impl<P, M, R> IntraProcessNetwork<P, M, R>
where
    P: Process + 'static + Send + Hash,
    M: Message + 'static + Send,
    R: IntraProcessNetworkReceiver<M> + Send + 'static,
{
    pub fn new() -> Result<Self, InternalError> {
        let (sender, receiver) = mpsc::channel();

        let process_to_receiver: Arc<Mutex<HashMap<P, R>>> = Arc::new(Mutex::new(HashMap::new()));
        let p_t_r = process_to_receiver.clone();

        let join_handle = thread::Builder::new().name("IntraProcessNetwork".to_string()).spawn(move || {

            loop {
                match receiver.recv() {
                    Ok(ControlMessage::Message(process, message)) => {
                        let mut map = p_t_r.lock().unwrap();
                        let mut receiver = map.get_mut(&process).unwrap();
                        receiver.deliver(message).unwrap();
                    }
                    Ok(ControlMessage::Shutdown) => break,
                    Err(e) => {
                        error!("{}", e);
                        break;
                    }
                }
            }
        }).map_err(|e| InternalError::from_source(Box::new(e)))?;

        Ok(IntraProcessNetwork {
            join_handle,
            sender,
            process_to_receiver,
        })
    }

    pub fn add_process(&mut self, process: P, receiver: R) {
        self.process_to_receiver.lock().unwrap().insert(process, receiver);
    }

    pub fn sender(&self) -> IntraProcessNetworkSender<P, M> {
        IntraProcessNetworkSender {
            inner: self.sender.clone()
        }
    }

    pub fn shutdown(self) -> Result<(), InternalError> {
        self.sender.send(ControlMessage::Shutdown).map_err(|e| InternalError::from_source(Box::new(e)))?;
        self.join_handle.join();
        Ok(())
    }
}

pub struct IntraProcessNetworkSender<P, M>
where
    P: Process,
    M: Message,
{
    inner: Sender<ControlMessage<P, M>>,
}

impl<P, M> IntraProcessNetworkSender<P, M>
where
    P: Process,
    M: Message,
{
    pub fn send(&self, process: P, message: M) -> Result<(), IntraProcessNetworkError> {
        Ok(self.inner.send(ControlMessage::Message(process, message)).unwrap()) // TODO unwrap
    }
}

pub trait IntraProcessNetworkReceiver<M> {
    fn deliver(&mut self, message: M) -> Result<(), IntraProcessNetworkError>;
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    struct TestProcess {
        id: u64,
    }

    impl Process for TestProcess {}

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestMessage {
        msg: String,
    }

    impl TestMessage {
        fn new<'a>(msg: &'a dyn ToString) -> Self {
            TestMessage {
                msg: msg.to_string()
            }
        }
    }

    impl Message for TestMessage {}

    #[derive(Clone)]
    struct Receiver {
        delivered: Arc<Mutex<Vec<TestMessage>>>,
    }

    impl Receiver {
        fn new() -> Self {
            Receiver {
                delivered: Arc::new(Mutex::new(vec![]))
            }
        }

        fn pop(&mut self) -> Option<TestMessage> {
            self.delivered.lock().unwrap().pop()
        }
    }

    impl IntraProcessNetworkReceiver<TestMessage> for Receiver {
        fn deliver(&mut self, message: TestMessage) -> Result<(), IntraProcessNetworkError> {
            self.delivered.lock().unwrap().push(message);
            Ok(())
        }
    }

    #[test]
    fn test_send_receive() {
        let mut network: IntraProcessNetwork<TestProcess, TestMessage, Receiver> = IntraProcessNetwork::new().unwrap();

        let process1 = TestProcess { id: 1 };
        let process2 = TestProcess { id: 2 };

        let mut receiver1 = Receiver::new();
        let mut receiver2 = Receiver::new();

        network.add_process(process1, receiver1.clone());
        network.add_process(process2, receiver2.clone());

        let sender = network.sender();
        sender.send(process1, TestMessage::new(&"Message 1"));
        sender.send(process2, TestMessage::new(&"Message 2"));

        network.shutdown().unwrap();

        assert_eq!(receiver1.pop(), Some(TestMessage { msg: "Message 1".to_string() }));
        assert_eq!(receiver2.pop(), Some(TestMessage { msg: "Message 2".to_string() }));
    }
}
