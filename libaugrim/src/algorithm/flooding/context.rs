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

use crate::algorithm::Value;
use crate::process::Process;

use super::Round;

pub struct FloodingContext<P, V>
where
    P: Process,
    V: Value,
{
    correct: Vec<P>,
    round: Round,
    decision: Option<V>,
    received_from: Vec<Vec<P>>,
    proposals: Vec<Vec<V>>,
}

impl<P, V> FloodingContext<P, V>
where
    P: Process,
    V: Value,
{
    pub fn new(processes: Vec<P>) -> Self {
        let processes_len = processes.len();

        let received_from = (0..processes_len)
            .map(|n| match n {
                0 => processes.clone(),
                _ => Vec::new(),
            })
            .collect();

        FloodingContext {
            correct: processes,
            round: 1,
            decision: None,
            received_from,
            proposals: vec![Vec::new(); processes_len],
        }
    }

    pub fn correct(&self) -> &Vec<P> {
        &self.correct
    }

    pub fn correct_mut(&mut self) -> &mut Vec<P> {
        &mut self.correct
    }

    pub fn round(&self) -> Round {
        self.round
    }

    pub fn set_round(&mut self, round: Round) {
        self.round = round
    }

    pub fn decision(&self) -> &Option<V> {
        &self.decision
    }

    pub fn set_decision(&mut self, decision: Option<V>) {
        self.decision = decision
    }

    pub fn received_from(&self) -> &Vec<Vec<P>> {
        &self.received_from
    }

    pub fn received_from_mut(&mut self) -> &mut Vec<Vec<P>> {
        &mut self.received_from
    }

    pub fn proposals(&self) -> &Vec<Vec<V>> {
        &self.proposals
    }

    pub fn proposals_mut(&mut self) -> &mut Vec<Vec<V>> {
        &mut self.proposals
    }
}
