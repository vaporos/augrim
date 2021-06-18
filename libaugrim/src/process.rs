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

//! Process trait

/// An identifier for an entity participating in consensus.
///
/// Multiple distributed processes coordinate to execute a consensus algorithm by sending messages
/// across a network that connects processes together[^note].
///
/// [^note]: For a full explanation of processes and the relation to other components, see Cachin,
/// Guerraoui, and Rodrigues, Reliable and Secure Distributed Programming, 2nd ed., 2.1.1.
pub trait Process: Copy + Eq + PartialEq {}