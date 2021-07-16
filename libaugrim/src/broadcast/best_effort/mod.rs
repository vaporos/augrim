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

/// p75
use crate::error::InternalError;
use crate::message::Message;
use crate::network::NetworkSender;
use crate::process::Process;

pub struct BestEffortBroadcastSender<P, M, N>
where
    P: Process,
    M: Message,
    N: NetworkSender<P, M>,
{
    network_sender: N,
    processes: Vec<P>,
    message_phantom: std::marker::PhantomData<M>,
}

impl<P, M, N> BestEffortBroadcastSender<P, M, N>
where
    P: Process,
    M: Message,
    N: NetworkSender<P, M>,
{
    pub fn new(network_sender: N, processes: Vec<P>) -> Self {
        BestEffortBroadcastSender {
            network_sender,
            processes,
            message_phantom: std::marker::PhantomData,
        }
    }

    pub fn broadcast(&self, message: &M) -> Result<(), InternalError> {
        for process in self.processes.clone() {
            self.network_sender.send(&process, message.clone())?;
        }
        Ok(())
    }
}

pub trait BestEffortBroadcastReceiver<P, M> {
    fn deliver(process: P, message: M) -> Result<(), InternalError>;
}
