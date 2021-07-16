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

use crate::error::InternalError;
use crate::process::Process;
use crate::message::Message;

use super::{FairLossSender, FairLossReceiver};

pub trait StubbornSender<P, M>
where
    P: Process,
    M: Message,
{
    fn send(to_process: &P, message: M) -> Result<(), InternalError>;
}

pub trait StubbornReceiver<P, M>
where
    P: Process,
    M: Message,
{
    fn recv(from_process: &P, message: M) -> Result<(), InternalError>;
}

// p36
struct DefaultStubbornSender<P, M, S>
where
    P: Process,
    M: Message,
    S: FairLossSender<P, M>,
{
    fair_loss_sender: S,
    process_phantom: std::marker::PhantomData<P>,
    message_phantom: std::marker::PhantomData<M>,
}

impl<P, M, S> DefaultStubbornSender<P, M, S>
where
    P: Process,
    M: Message,
    S: FairLossSender<P, M>,
{
    fn new(fair_loss_sender: S) -> Self {
        DefaultStubbornSender {
            fair_loss_sender,
            process_phantom: std::marker::PhantomData,
            message_phantom: std::marker::PhantomData,
        }
    }
}

impl<P, M, S> StubbornSender<P, M> for DefaultStubbornSender<P, M, S>
where
    P: Process,
    M: Message,
    S: FairLossSender<P, M>,
{
    fn send(to_process: &P, message: M) -> Result<(), InternalError> {
        unimplemented!()
    }
}

//impl<P, M> FairLossReceiver for StubbornReceiver<P, M> {
//
//}
