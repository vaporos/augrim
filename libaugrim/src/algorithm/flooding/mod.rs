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

use crate::broadcast::BestEffortBroadcastSender;
use crate::error::InternalError;
use crate::message::Message;
use crate::network::NetworkSender;
use crate::process::Process;

// p50
pub trait PerfectFailureDetectorReceiver<P> {
    fn crash(&mut self, process: P) -> Result<(), InternalError>;
}

// p205
trait ConsensusProposer<V> {
    fn propose(value: V) -> Result<(), InternalError>;
}

// p205
trait Consensus<V> {
    fn propose(value: V) -> Result<(), InternalError>;
}

// p206
#[derive(Clone)]
pub enum FloodingConsensusMessage<V: Clone, PROPOSAL: Clone> {
    Decided {
        value: V,
    },
    Proposal {
        round: u64,
        proposals: Vec<PROPOSAL>,
    },
}

impl<V: Clone, PROPOSAL: Clone> Message for FloodingConsensusMessage<V, PROPOSAL> {}

// p206
pub struct FloodingConsensus<D, P, V, PROPOSAL, N>
where
    D: PerfectFailureDetectorReceiver<P>,
    P: Process,
    V: Clone,
    PROPOSAL: Clone,
    N: NetworkSender<P, FloodingConsensusMessage<V, PROPOSAL>>,
{
    correct: Vec<P>,
    round: u64,
    decision: Option<V>,
    received_from: Vec<Vec<P>>,
    proposals: Vec<Vec<PROPOSAL>>,

    detector: D,
    sender: BestEffortBroadcastSender<P, FloodingConsensusMessage<V, PROPOSAL>, N>,
    process_phantom: std::marker::PhantomData<P>,
    value_phantom: std::marker::PhantomData<V>,
    proposal_phantom: std::marker::PhantomData<PROPOSAL>,
}

impl<D, P, V, PROPOSAL, N> FloodingConsensus<D, P, V, PROPOSAL, N>
where
    D: PerfectFailureDetectorReceiver<P>,
    P: Process + Clone,
    V: Clone,
    PROPOSAL: Clone,
    N: NetworkSender<P, FloodingConsensusMessage<V, PROPOSAL>>,
{
    pub fn new(
        processes: Vec<P>,
        detector: D,
        sender: BestEffortBroadcastSender<P, FloodingConsensusMessage<V, PROPOSAL>, N>,
    ) -> Self {
        FloodingConsensus {
            correct: processes.clone(),
            round: 1,
            decision: None,
            received_from: vec![processes],
            proposals: vec![],
            detector,
            sender,
            process_phantom: std::marker::PhantomData,
            value_phantom: std::marker::PhantomData,
            proposal_phantom: std::marker::PhantomData,
        }
    }
}

impl<D, P, V, PROPOSAL, N> Consensus<V> for FloodingConsensus<D, P, V, PROPOSAL, N>
where
    D: PerfectFailureDetectorReceiver<P>,
    P: Process,
    V: Clone,
    PROPOSAL: Clone,
    N: NetworkSender<P, FloodingConsensusMessage<V, PROPOSAL>>,
{
    fn propose(value: V) -> Result<(), InternalError> {
        unimplemented!()
    }
}

impl<D, P, V, PROPOSAL, N> PerfectFailureDetectorReceiver<P>
    for FloodingConsensus<D, P, V, PROPOSAL, N>
where
    D: PerfectFailureDetectorReceiver<P>,
    P: Process,
    V: Clone,
    PROPOSAL: Clone,
    N: NetworkSender<P, FloodingConsensusMessage<V, PROPOSAL>>,
{
    fn crash(&mut self, process: P) -> Result<(), InternalError> {
        match self.correct.iter().position(|p| *p == process) {
            Some(index) => {
                self.correct.remove(index);
            }
            None => (),
        };
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::sync::{Arc, Mutex};

    use crate::communication::internal::{
        IntraProcessNetwork, IntraProcessNetworkError, IntraProcessNetworkReceiver,
    };
    use crate::message::Message;

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
                msg: msg.to_string(),
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
                delivered: Arc::new(Mutex::new(vec![])),
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
        let mut network: IntraProcessNetwork<TestProcess, TestMessage, Receiver> =
            IntraProcessNetwork::new().unwrap();

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

        assert_eq!(
            receiver1.pop(),
            Some(TestMessage {
                msg: "Message 1".to_string()
            })
        );
        assert_eq!(
            receiver2.pop(),
            Some(TestMessage {
                msg: "Message 2".to_string()
            })
        );
    }
}
