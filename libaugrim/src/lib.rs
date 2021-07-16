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

/// # Bibliography
///
/// An emphasis has been placed on implementing algorithms as described in textbooks, research
/// papers, and other sources. The following is a complete list of the sources cited elsewhere.
///
/// 1. Cachin, Christian, Rachid Guerraoui, and Louis Rodrigues. Introduction to Reliable and Secure
///    Distributed Programming, 2nd ed. (Heidelberg: Springer, 2011).

#[allow(unused_imports)]
#[macro_use]
extern crate log;

#[cfg(feature = "algorithm")]
pub mod algorithm;
#[cfg(feature = "broadcast")]
pub mod broadcast;
#[cfg(feature = "communication")]
pub mod communication;
pub mod error;
#[cfg(feature = "links")]
pub mod links;
#[cfg(feature = "message")]
pub mod message;
#[cfg(feature = "network")]
pub mod network;
#[cfg(feature = "process")]
pub mod process;
