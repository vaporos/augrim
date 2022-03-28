// Copyright 2021-2022 Cargill Incorporated
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

#[cfg(feature = "algorithm-flooding")]
pub mod flooding;

pub trait Action {}
pub trait Context {}
pub trait Value: Clone {}

pub trait Algorithm<P>
where
    P: Process,
{
    type Event;
    type Action;
    type Context;

    fn event(
        &self,
        event: Self::Event,
        context: Self::Context,
    ) -> Result<Vec<Self::Action>, InternalError>;
}
