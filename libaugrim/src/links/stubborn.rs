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
use crate::message::Message;
use crate::process::Process;

use super::{FairLossLink, Receiver, Sender};

pub trait StubbornLink {}

impl<T> FairLossLink for T where T: StubbornLink {}

// p36
pub struct StubbornSender<P, M, S>
where
    P: Process,
    M: Message,
    S: Sender<P, M> + FairLossLink,
{
    inner: S,
    process_phantom: std::marker::PhantomData<P>,
    message_phantom: std::marker::PhantomData<M>,
}

impl<P, M, S> StubbornSender<P, M, S>
where
    P: Process,
    M: Message,
    S: Sender<P, M> + FairLossLink,
{
    fn new(sender: S) -> Self {
        StubbornSender {
            inner: sender,
            process_phantom: std::marker::PhantomData,
            message_phantom: std::marker::PhantomData,
        }
    }
}

impl<P, M, S> Sender<P, M> for StubbornSender<P, M, S>
where
    P: Process,
    M: Message,
    S: Sender<P, M> + FairLossLink,
{
    fn send(to_process: &P, message: M) -> Result<(), InternalError> {
        unimplemented!()
    }
}

impl<P, M, S> StubbornLink for StubbornSender<P, M, S>
where
    P: Process,
    M: Message,
    S: Sender<P, M> + FairLossLink,
{
}

pub struct StubbornReceiver<P, M, R>
where
    P: Process,
    M: Message,
    R: Receiver<P, M> + FairLossLink,
{
    inner: R,
    process_phantom: std::marker::PhantomData<P>,
    message_phantom: std::marker::PhantomData<M>,
}

impl<P, M, R> StubbornReceiver<P, M, R>
where
    P: Process,
    M: Message,
    R: Receiver<P, M> + FairLossLink,
{
    fn new(receiver: R) -> Self {
        StubbornReceiver {
            inner: receiver,
            process_phantom: std::marker::PhantomData,
            message_phantom: std::marker::PhantomData,
        }
    }
}

impl<P, M, R> Receiver<P, M> for StubbornReceiver<P, M, R>
where
    P: Process,
    M: Message,
    R: Receiver<P, M> + FairLossLink,
{
    fn recv(from_process: &P, message: M) -> Result<(), InternalError> {
        unimplemented!()
    }
}

impl<P, M, R> StubbornLink for StubbornReceiver<P, M, R>
where
    P: Process,
    M: Message,
    R: Receiver<P, M> + FairLossLink,
{
}
